use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::broadcast;

use crate::managers::common::matchers::path_glob_matcher::PathGlobMatcher;
use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::agent_run::processors::architecture_lint::architecture_lint_processor::ArchitectureLintProcessor;
use crate::managers::engines::agent_run::processors::patch::patch_processor::PatchProcessor;
use crate::managers::engines::agent_run::processors::project_map::project_map_processor::ProjectMapProcessor;
use crate::managers::engines::agent_run::processors::prompt::prompt_processor::PromptProcessor;
use crate::managers::engines::agent_run::processors::scaffold::scaffold_processor::ScaffoldProcessor;
use crate::managers::engines::agent_run::processors::tool::tool_processor::ToolProcessor;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::exceptions::workspace_gateway_exception::WorkspaceGatewayException;
use crate::managers::gateways::command_runner::command_runner_gateway::CommandRunnerGateway;
use crate::managers::gateways::openai_responses::openai_responses_gateway::OpenAiResponsesGateway;
use crate::managers::gateways::workspace::workspace_gateway::WorkspaceGateway;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::check_result::CheckResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::patch_operation::PatchOperation;
use crate::managers::models::patch_operation_kind::PatchOperationKind;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::run_context::RunContext;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::scaffold_plan::ScaffoldPlan;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_lint::StackPackLint;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::stack_pack_scaffold::StackPackScaffold;
use crate::managers::models::tool_call::ToolCall;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::models::user_request::UserRequest;
use crate::managers::models::workspace_file::WorkspaceFile;

const APPLY_PATCH: &str = "apply_patch";
// The check the harness runs itself, before the pack's commands.
const ARCHITECTURE_CHECK: &str = "architecture";

// Runs one agent on one request, to completion: the only component that talks to the model. Each turn sends the
// session's whole conversation (the API keeps nothing), does what the model asked (reads, searches, patches, checks)
// and sends what came of it back, until the model answers without asking for anything. Every item is appended to the
// session store as it happens, so the conversation survives the run.
pub struct AgentRunEngine {
    openai_responses_gateway: Arc<OpenAiResponsesGateway>,
    workspace_gateway: Arc<WorkspaceGateway>,
    command_runner_gateway: Arc<CommandRunnerGateway>,
    prompt_processor: Arc<PromptProcessor>,
    tool_processor: Arc<ToolProcessor>,
    patch_processor: Arc<PatchProcessor>,
    project_map_processor: Arc<ProjectMapProcessor>,
    architecture_lint_processor: Arc<ArchitectureLintProcessor>,
    scaffold_processor: Arc<ScaffoldProcessor>,
    max_turns: i64,
    check_timeout: Duration,
}

impl AgentRunEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        openai_responses_gateway: Arc<OpenAiResponsesGateway>,
        workspace_gateway: Arc<WorkspaceGateway>,
        command_runner_gateway: Arc<CommandRunnerGateway>,
        prompt_processor: Arc<PromptProcessor>,
        tool_processor: Arc<ToolProcessor>,
        patch_processor: Arc<PatchProcessor>,
        project_map_processor: Arc<ProjectMapProcessor>,
        architecture_lint_processor: Arc<ArchitectureLintProcessor>,
        scaffold_processor: Arc<ScaffoldProcessor>,
        max_turns: i64,
        check_timeout: Duration,
    ) -> Self {
        AgentRunEngine {
            openai_responses_gateway,
            workspace_gateway,
            command_runner_gateway,
            prompt_processor,
            tool_processor,
            patch_processor,
            project_map_processor,
            architecture_lint_processor,
            scaffold_processor,
            max_turns,
            check_timeout,
        }
    }

    /// Reports what happens on `events`; the run's start and end are the manager's to report. The engine is a shared
    /// singleton, which the ServiceLocator builds without the DataFacade, so the manager passes it in.
    pub async fn run(
        &self,
        session: &Session,
        user_request: &UserRequest,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<RunSummary> {
        let run_context: RunContext = self.prepare_run(session, data_facade).await?;

        let mut conversation_items: Vec<ConversationItem> = data_facade.get_conversation_items(&session.id).await?;
        // A run cancelled midway can leave tool calls unanswered, which the API refuses: they are answered first.
        let unfinished_tool_call_outputs: Vec<ConversationItem> =
            self.tool_processor.answer_unfinished_tool_calls(&conversation_items);
        if !unfinished_tool_call_outputs.is_empty() {
            data_facade
                .append_conversation_items(&session.id, &unfinished_tool_call_outputs)
                .await?;
            conversation_items.extend(unfinished_tool_call_outputs);
        }
        let mut new_items: Vec<ConversationItem> = Vec::with_capacity(2);
        if let Some(project_map_note) = run_context.project_map_note.as_ref() {
            new_items.push(project_map_note.clone());
        }
        new_items.push(ConversationItem::UserMessage {
            text: user_request.text.clone(),
        });
        data_facade.append_conversation_items(&session.id, &new_items).await?;
        conversation_items.extend(new_items);

        let mut usage: ModelUsage = ModelUsage::default();
        for turn in 1..=self.max_turns {
            let model_response: ModelResponse = self
                .openai_responses_gateway
                .create_response(&ModelRequest {
                    lane_id: session.id.clone(),
                    prompt_layers: run_context.prompt_layers.clone(),
                    items: conversation_items.clone(),
                    tools: run_context.tools.clone(),
                    service_tier: session.service_tier,
                })
                .await?;
            usage = usage.plus(model_response.usage);
            data_facade
                .append_usage_record(&AgentRunEngine::to_usage_record(session, &model_response))
                .await?;
            data_facade
                .append_conversation_items(&session.id, &model_response.output_items)
                .await?;
            conversation_items.extend(model_response.output_items.iter().cloned());

            if let Some(text) = model_response.text.as_ref() {
                events.send(SessionEvent::AssistantReplied { text: text.clone() }).ok();
            }
            if model_response.tool_calls.is_empty() {
                return Ok(RunSummary {
                    turns: turn,
                    usage,
                    service_tier: model_response.service_tier,
                });
            }

            let mut tool_outputs: Vec<ConversationItem> = Vec::with_capacity(model_response.tool_calls.len());
            for tool_call in &model_response.tool_calls {
                tool_outputs.push(self.use_tool(session, &run_context, tool_call, events).await?);
            }
            data_facade
                .append_conversation_items(&session.id, &tool_outputs)
                .await?;
            conversation_items.extend(tool_outputs);
        }

        Err(ApplicationBaseException::new(
            AgentRunEngineException::MaxTurnsReached,
            format!("The run stopped after {} model turns without finishing", self.max_turns),
            ContextualData::new()
                .with("session_id", session.id.as_str())
                .with("max_turns", self.max_turns),
        ))
    }

    // The workspace's stack (the first pack whose files it has), and the prompt layers and tools every turn sends.
    async fn prepare_run(&self, session: &Session, data_facade: &DataFacade) -> ApplicationResult<RunContext> {
        let mut workspace_stack_pack: Option<StackPack> = None;
        for stack_pack in data_facade.get_stack_packs().await? {
            if self
                .has_files(&session.workspace_directory, &stack_pack.detect_files)
                .await?
            {
                workspace_stack_pack = Some(stack_pack);
                break;
            }
        }
        let rules: Option<String> = match workspace_stack_pack.as_ref() {
            Some(stack_pack) => Some(data_facade.get_stack_pack_rules(stack_pack).await?),
            None => None,
        };
        let (project_map, project_map_note): (Option<String>, Option<ConversationItem>) = match workspace_stack_pack
            .as_ref()
            .and_then(|stack_pack: &StackPack| stack_pack.language.as_deref().zip(stack_pack.project_map.as_ref()))
        {
            Some((language_name, stack_pack_project_map)) => {
                let current_map: String = self
                    .build_project_map(&session.workspace_directory, language_name, stack_pack_project_map)
                    .await?;
                let (layer_map, project_map_note): (String, Option<ConversationItem>) =
                    self.choose_project_map(session, current_map, data_facade).await?;
                (Some(layer_map), project_map_note)
            }
            None => (None, None),
        };

        Ok(RunContext {
            prompt_layers: self.prompt_processor.build_prompt_layers(
                &session.workspace_directory,
                workspace_stack_pack.as_ref(),
                rules.as_deref(),
                project_map.as_deref(),
            ),
            tools: self.tool_processor.get_tool_definitions(),
            stack_pack: workspace_stack_pack,
            project_map_note,
        })
    }

    // The map the requests open with, and a note of what changed since the last run, if anything did. The session
    // keeps its map from run to run so the requests' prefix (and the lane) carries over; changes are told in notes,
    // until there are too many and the map is rebuilt.
    async fn choose_project_map(
        &self,
        session: &Session,
        current_map: String,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(String, Option<ConversationItem>)> {
        let session_project_maps: SessionProjectMaps = data_facade.get_session_project_maps(&session.id).await?;
        let Some(layer_map) = session_project_maps.layer_map else {
            data_facade
                .append_project_map(&session.id, &current_map, ProjectMapRole::Layer)
                .await?;
            return Ok((current_map, None));
        };
        let latest_map: String = session_project_maps.latest_map.unwrap_or_else(|| layer_map.clone());
        if latest_map == current_map {
            return Ok((layer_map, None));
        }
        if self.project_map_processor.should_rebuild(&layer_map, &current_map) {
            data_facade
                .append_project_map(&session.id, &current_map, ProjectMapRole::Layer)
                .await?;
            return Ok((current_map, None));
        }

        let changes: String = self.project_map_processor.describe_changes(&latest_map, &current_map);
        data_facade
            .append_project_map(&session.id, &current_map, ProjectMapRole::Seen)
            .await?;
        Ok((layer_map, Some(ConversationItem::HarnessNote { text: changes })))
    }

    // Every file of the workspace, and the public items of the files the pack's globs select.
    async fn build_project_map(
        &self,
        workspace_directory: &Path,
        language_name: &str,
        stack_pack_project_map: &StackPackProjectMap,
    ) -> ApplicationResult<String> {
        let (paths, source_files): (Vec<String>, Vec<WorkspaceFile>) = self
            .read_source_files(workspace_directory, &stack_pack_project_map.source_globs)
            .await?;
        self.project_map_processor
            .build_project_map(&paths, &source_files, language_name, stack_pack_project_map)
    }

    // Every file of the workspace, and those the globs select, read. A source file that can't be read as text is left
    // out.
    async fn read_source_files(
        &self,
        workspace_directory: &Path,
        source_globs: &[String],
    ) -> ApplicationResult<(Vec<String>, Vec<WorkspaceFile>)> {
        let paths: Vec<String> = self.workspace_gateway.list_files(workspace_directory, ".").await?;
        let source_paths: Vec<String> = PathGlobMatcher::select(&paths, source_globs)?;

        let mut source_files: Vec<WorkspaceFile> = Vec::with_capacity(source_paths.len());
        for source_path in &source_paths {
            match self.workspace_gateway.read_file(workspace_directory, source_path).await {
                Ok(source_file) => source_files.push(source_file),
                Err(exception)
                    if exception.kind() == ExceptionKind::WorkspaceGateway(WorkspaceGatewayException::NotTextFile) =>
                {
                    ApplicationLogger::log_exception(&exception, LogLevel::Warning);
                }
                Err(exception) => return Err(exception),
            }
        }
        Ok((paths, source_files))
    }

    // The architecture lint, as a check: passed when no file breaks a rule.
    async fn run_architecture_lint(
        &self,
        workspace_directory: &Path,
        language_name: &str,
        stack_pack_lint: &StackPackLint,
    ) -> ApplicationResult<CheckResult> {
        let (_, source_files): (Vec<String>, Vec<WorkspaceFile>) = self
            .read_source_files(workspace_directory, &stack_pack_lint.source_globs)
            .await?;
        let lint_violations: Vec<LintViolation> =
            self.architecture_lint_processor
                .lint(&source_files, language_name, stack_pack_lint)?;
        Ok(CheckResult {
            name: ARCHITECTURE_CHECK.to_string(),
            passed: lint_violations.is_empty(),
            exit_code: None,
            timed_out: false,
            output: self.tool_processor.format_lint_violations(&lint_violations),
        })
    }

    async fn has_files(&self, workspace_directory: &Path, paths: &[String]) -> ApplicationResult<bool> {
        for path in paths {
            if !self.workspace_gateway.file_exists(workspace_directory, path).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // Does what one tool call asks, reports it, and returns the output the model is sent for it.
    async fn use_tool(
        &self,
        session: &Session,
        run_context: &RunContext,
        tool_call: &ToolCall,
        events: &broadcast::Sender<SessionEvent>,
    ) -> ApplicationResult<ConversationItem> {
        match tool_call {
            ToolCall::Function {
                call_id,
                name,
                arguments,
            } => {
                let (target, output, succeeded): (String, String, bool) =
                    match self.tool_processor.to_tool_request(name, arguments) {
                        Ok(tool_request) => {
                            let (output, succeeded): (String, bool) =
                                self.use_function_tool(session, run_context, &tool_request).await?;
                            (self.tool_processor.describe_target(&tool_request), output, succeeded)
                        }
                        Err(exception) => (String::new(), self.to_tool_error(exception)?, false),
                    };
                events
                    .send(SessionEvent::ToolUsed {
                        tool_name: name.clone(),
                        target,
                        succeeded,
                    })
                    .ok();
                Ok(ConversationItem::FunctionCallOutput {
                    call_id: call_id.clone(),
                    output,
                })
            }
            ToolCall::ApplyPatch {
                call_id,
                patch_operation,
            } => {
                let (output, succeeded): (String, bool) =
                    match self.apply_patch(&session.workspace_directory, patch_operation).await {
                        Ok(output) => (output, true),
                        // The model knows this error format from the reference applier.
                        Err(exception) => (format!("Error: {}", self.to_tool_error_message(exception)?), false),
                    };
                events
                    .send(SessionEvent::ToolUsed {
                        tool_name: APPLY_PATCH.to_string(),
                        target: format!("{} {}", patch_operation.kind.name(), patch_operation.path),
                        succeeded,
                    })
                    .ok();
                Ok(ConversationItem::ApplyPatchCallOutput {
                    call_id: call_id.clone(),
                    succeeded,
                    output,
                })
            }
        }
    }

    // The tool's output and whether it did what was asked.
    async fn use_function_tool(
        &self,
        session: &Session,
        run_context: &RunContext,
        tool_request: &ToolRequest,
    ) -> ApplicationResult<(String, bool)> {
        let workspace_directory: &Path = &session.workspace_directory;
        match tool_request {
            ToolRequest::ReadFiles { paths } => {
                let mut sections: Vec<String> = Vec::with_capacity(paths.len());
                let mut succeeded: bool = true;
                for path in paths {
                    match self.workspace_gateway.read_file(workspace_directory, path).await {
                        Ok(workspace_file) => sections.push(self.tool_processor.format_file(&workspace_file)),
                        Err(exception) => {
                            succeeded = false;
                            let message: String = self.to_tool_error_message(exception)?;
                            sections.push(self.tool_processor.format_file_error(path, &message));
                        }
                    }
                }
                Ok((sections.join("\n\n"), succeeded))
            }
            ToolRequest::ListFiles { directory } => {
                match self.workspace_gateway.list_files(workspace_directory, directory).await {
                    Ok(paths) => Ok((self.tool_processor.format_file_list(&paths), true)),
                    Err(exception) => Ok((self.to_tool_error(exception)?, false)),
                }
            }
            ToolRequest::Search { pattern, path_glob } => {
                match self
                    .workspace_gateway
                    .search(workspace_directory, pattern, path_glob.as_deref())
                    .await
                {
                    Ok(search_matches) => Ok((self.tool_processor.format_search_matches(&search_matches), true)),
                    Err(exception) => Ok((self.to_tool_error(exception)?, false)),
                }
            }
            ToolRequest::Scaffold { kind, name, within } => {
                match run_context
                    .stack_pack
                    .as_ref()
                    .and_then(|stack_pack: &StackPack| stack_pack.scaffold.as_ref())
                {
                    None => Ok((
                        self.tool_processor
                            .format_error("The workspace's stack has no scaffolds; write the component by hand"),
                        false,
                    )),
                    Some(stack_pack_scaffold) => {
                        match self
                            .scaffold(workspace_directory, stack_pack_scaffold, kind, name, within.as_deref())
                            .await
                        {
                            Ok(output) => Ok((output, true)),
                            Err(exception) => Ok((self.to_tool_error(exception)?, false)),
                        }
                    }
                }
            }
            ToolRequest::RunChecks => match run_context.stack_pack.as_ref() {
                None => Ok((
                    self.tool_processor
                        .format_error("No stack pack matches the workspace, so it has no checks"),
                    false,
                )),
                Some(stack_pack) => {
                    let check_results: Vec<CheckResult> = self.run_checks(workspace_directory, stack_pack).await?;
                    let succeeded: bool = check_results
                        .iter()
                        .all(|check_result: &CheckResult| check_result.passed);
                    Ok((self.tool_processor.format_check_results(&check_results), succeeded))
                }
            },
        }
    }

    // Creates a component from the pack's scaffold kind and wires it in. Everything is checked first (the component it
    // lives within exists, no file it creates does, every anchor is found), so a scaffold applies whole or not at all;
    // then the pack's formatter sorts the lines it added in.
    async fn scaffold(
        &self,
        workspace_directory: &Path,
        stack_pack_scaffold: &StackPackScaffold,
        kind: &str,
        name: &str,
        within: Option<&str>,
    ) -> ApplicationResult<String> {
        let scaffold_plan: ScaffoldPlan = self.scaffold_processor.plan(stack_pack_scaffold, kind, name, within)?;

        if let Some(within_path) = scaffold_plan.within_path.as_deref()
            && !self
                .workspace_gateway
                .file_exists(workspace_directory, within_path)
                .await?
        {
            return Err(ApplicationBaseException::new(
                AgentRunEngineException::ScaffoldNotApplicable,
                format!(
                    "{within_path} doesn't exist: there is no component named {}",
                    within.unwrap_or_default()
                ),
                ContextualData::new().with("path", within_path),
            ));
        }
        for created_file in &scaffold_plan.created_files {
            if self
                .workspace_gateway
                .file_exists(workspace_directory, &created_file.path)
                .await?
            {
                return Err(ApplicationBaseException::new(
                    AgentRunEngineException::ScaffoldNotApplicable,
                    format!("{} already exists: the component is there already", created_file.path),
                    ContextualData::new().with("path", created_file.path.as_str()),
                ));
            }
        }

        // Each file wired into, with every insertion made, in the order they are first named.
        let mut updated_files: Vec<WorkspaceFile> = Vec::new();
        for scaffold_insertion in &scaffold_plan.insertions {
            let position: Option<usize> = updated_files
                .iter()
                .position(|updated_file: &WorkspaceFile| updated_file.path == scaffold_insertion.path);
            let current: WorkspaceFile = match position {
                Some(index) => updated_files.remove(index),
                None => {
                    self.workspace_gateway
                        .read_file(workspace_directory, &scaffold_insertion.path)
                        .await?
                }
            };
            let content: String = self.scaffold_processor.insert(&current.content, scaffold_insertion)?;
            updated_files.push(WorkspaceFile {
                path: current.path,
                content,
            });
        }

        for created_file in &scaffold_plan.created_files {
            self.workspace_gateway
                .write_file(workspace_directory, &created_file.path, &created_file.content, true)
                .await?;
        }
        for updated_file in &updated_files {
            self.workspace_gateway
                .write_file(workspace_directory, &updated_file.path, &updated_file.content, false)
                .await?;
        }

        let format_result: String = if stack_pack_scaffold.format_command.is_empty() {
            String::new()
        } else {
            match self
                .command_runner_gateway
                .run(
                    workspace_directory,
                    &stack_pack_scaffold.format_command,
                    self.check_timeout,
                )
                .await
            {
                Ok(command_result) if command_result.exit_code == Some(0) => "Formatted.".to_string(),
                Ok(command_result) => format!("Formatting failed:\n{}", command_result.output.trim_end()),
                Err(exception) => format!("Formatting failed: {}", self.to_tool_error_message(exception)?),
            }
        };

        // As they are after formatting, which the model patches against.
        let mut created_files: Vec<WorkspaceFile> = Vec::with_capacity(scaffold_plan.created_files.len());
        for created_file in &scaffold_plan.created_files {
            created_files.push(
                self.workspace_gateway
                    .read_file(workspace_directory, &created_file.path)
                    .await?,
            );
        }
        let updated_paths: Vec<String> = updated_files
            .iter()
            .map(|updated_file: &WorkspaceFile| updated_file.path.clone())
            .collect();
        Ok(self
            .tool_processor
            .format_scaffold(&created_files, &updated_paths, &format_result))
    }

    // Every check of the pack that has a command, in order; a check that can't be started fails.
    async fn run_checks(
        &self,
        workspace_directory: &Path,
        stack_pack: &StackPack,
    ) -> ApplicationResult<Vec<CheckResult>> {
        let mut check_results: Vec<CheckResult> = Vec::new();
        if let (Some(language_name), Some(stack_pack_lint)) = (stack_pack.language.as_deref(), stack_pack.lint.as_ref())
        {
            check_results.push(
                self.run_architecture_lint(workspace_directory, language_name, stack_pack_lint)
                    .await?,
            );
        }
        for check in &stack_pack.checks {
            if check.command.is_empty() {
                continue;
            }
            let check_result: CheckResult = match self
                .command_runner_gateway
                .run(workspace_directory, &check.command, self.check_timeout)
                .await
            {
                Ok(command_result) => CheckResult {
                    name: check.name.clone(),
                    passed: command_result.exit_code == Some(0),
                    exit_code: command_result.exit_code,
                    timed_out: command_result.timed_out,
                    output: command_result.output,
                },
                Err(exception) => CheckResult {
                    name: check.name.clone(),
                    passed: false,
                    exit_code: None,
                    timed_out: false,
                    output: self.to_tool_error_message(exception)?,
                },
            };
            check_results.push(check_result);
        }
        Ok(check_results)
    }

    // A patch's file operation; what it did, for the model.
    async fn apply_patch(
        &self,
        workspace_directory: &Path,
        patch_operation: &PatchOperation,
    ) -> ApplicationResult<String> {
        let diff: &str = patch_operation.diff.as_deref().unwrap_or_default();
        match patch_operation.kind {
            PatchOperationKind::CreateFile => {
                let content: String = self.patch_processor.create_file_content(diff)?;
                self.workspace_gateway
                    .write_file(workspace_directory, &patch_operation.path, &content, true)
                    .await?;
                Ok(format!("Created {}", patch_operation.path))
            }
            PatchOperationKind::UpdateFile => {
                let workspace_file: WorkspaceFile = self
                    .workspace_gateway
                    .read_file(workspace_directory, &patch_operation.path)
                    .await?;
                let content: String = self
                    .patch_processor
                    .update_file_content(&workspace_file.content, diff)?;
                self.workspace_gateway
                    .write_file(workspace_directory, &patch_operation.path, &content, false)
                    .await?;
                Ok(format!("Updated {}", patch_operation.path))
            }
            PatchOperationKind::DeleteFile => {
                self.workspace_gateway
                    .delete_file(workspace_directory, &patch_operation.path)
                    .await?;
                Ok(format!("Deleted {}", patch_operation.path))
            }
        }
    }

    fn to_tool_error(&self, exception: ApplicationBaseException) -> ApplicationResult<String> {
        Ok(self
            .tool_processor
            .format_error(&self.to_tool_error_message(exception)?))
    }

    // An exception a tool call ran into, told to the model so it can do something else: logged here, where it is
    // caught, as a warning. One the model can do nothing about ends the run.
    fn to_tool_error_message(&self, exception: ApplicationBaseException) -> ApplicationResult<String> {
        if !AgentRunEngine::is_told_to_model(exception.kind()) {
            return Err(exception);
        }
        ApplicationLogger::log_exception(&exception, LogLevel::Warning);
        Ok(exception.message().to_string())
    }

    /// Names every family, with no `_` arm, so a new family doesn't compile until it is decided here.
    fn is_told_to_model(exception_kind: ExceptionKind) -> bool {
        match exception_kind {
            ExceptionKind::AgentRunEngine(exception) => match exception {
                AgentRunEngineException::InvalidToolCall
                | AgentRunEngineException::InvalidPatch
                | AgentRunEngineException::ScaffoldNotApplicable => true,
                AgentRunEngineException::MaxTurnsReached => false,
            },
            ExceptionKind::WorkspaceGateway(_) | ExceptionKind::CommandRunnerGateway(_) => true,
            ExceptionKind::ConfigurationProvider(_)
            | ExceptionKind::OpenAiResponsesGateway(_)
            | ExceptionKind::OttoBotManager(_)
            | ExceptionKind::SessionStoreGateway(_)
            | ExceptionKind::StackPackStoreGateway(_)
            | ExceptionKind::InvalidUserRequest
            | ExceptionKind::InvalidStackPack => false,
        }
    }

    fn to_usage_record(session: &Session, model_response: &ModelResponse) -> UsageRecord {
        UsageRecord {
            session_id: session.id.clone(),
            response_id: model_response.id.clone(),
            model: model_response.model.clone(),
            requested_service_tier: session.service_tier,
            served_service_tier: model_response.service_tier,
            usage: model_response.usage,
            recorded_on: Utc::now(),
        }
    }
}
