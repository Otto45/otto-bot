use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::broadcast;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::agent_run::processors::patch::patch_processor::PatchProcessor;
use crate::managers::engines::agent_run::processors::project_map::project_map_processor::ProjectMapProcessor;
use crate::managers::engines::agent_run::processors::prompt::prompt_processor::PromptProcessor;
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
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::patch_operation::PatchOperation;
use crate::managers::models::patch_operation_kind::PatchOperationKind;
use crate::managers::models::run_context::RunContext;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::tool_call::ToolCall;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::models::user_request::UserRequest;
use crate::managers::models::workspace_file::WorkspaceFile;

const APPLY_PATCH: &str = "apply_patch";

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
        let user_message: ConversationItem = ConversationItem::UserMessage {
            text: user_request.text.clone(),
        };
        data_facade
            .append_conversation_items(&session.id, std::slice::from_ref(&user_message))
            .await?;
        conversation_items.push(user_message);

        let mut usage: ModelUsage = ModelUsage::default();
        for turn in 1..=self.max_turns {
            let model_response: ModelResponse = self
                .openai_responses_gateway
                .create_response(&ModelRequest {
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
        let project_map: Option<String> = match workspace_stack_pack
            .as_ref()
            .and_then(|stack_pack: &StackPack| stack_pack.project_map.as_ref())
        {
            Some(stack_pack_project_map) => Some(
                self.build_project_map(&session.workspace_directory, stack_pack_project_map)
                    .await?,
            ),
            None => None,
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
        })
    }

    // Every file of the workspace, and the public items of the files the pack's globs select. A source file that can't
    // be read as text is listed without its items.
    async fn build_project_map(
        &self,
        workspace_directory: &Path,
        stack_pack_project_map: &StackPackProjectMap,
    ) -> ApplicationResult<String> {
        let paths: Vec<String> = self.workspace_gateway.list_files(workspace_directory, ".").await?;
        let source_paths: Vec<String> = self
            .project_map_processor
            .select_source_paths(&paths, stack_pack_project_map)?;

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
        self.project_map_processor
            .build_project_map(&paths, &source_files, stack_pack_project_map)
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

    // Every check of the pack that has a command, in order; a check that can't be started fails.
    async fn run_checks(
        &self,
        workspace_directory: &Path,
        stack_pack: &StackPack,
    ) -> ApplicationResult<Vec<CheckResult>> {
        let mut check_results: Vec<CheckResult> = Vec::new();
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
                AgentRunEngineException::InvalidToolCall | AgentRunEngineException::InvalidPatch => true,
                AgentRunEngineException::MaxTurnsReached | AgentRunEngineException::InvalidProjectMap => false,
            },
            ExceptionKind::WorkspaceGateway(_) | ExceptionKind::CommandRunnerGateway(_) => true,
            ExceptionKind::ConfigurationProvider(_)
            | ExceptionKind::OpenAiResponsesGateway(_)
            | ExceptionKind::OttoBotManager(_)
            | ExceptionKind::SessionStoreGateway(_)
            | ExceptionKind::StackPackStoreGateway(_)
            | ExceptionKind::InvalidUserRequest => false,
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
