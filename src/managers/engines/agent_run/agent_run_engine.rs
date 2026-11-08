use std::collections::HashMap;
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
use crate::managers::engines::agent_run::processors::work_plan::work_plan_processor::WorkPlanProcessor;
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
use crate::managers::models::agent_assignment::AgentAssignment;
use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::agent_role::AgentRole;
use crate::managers::models::agent_run_outcome::AgentRunOutcome;
use crate::managers::models::check_result::CheckResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::format_result::FormatResult;
use crate::managers::models::implementation_result::ImplementationResult;
use crate::managers::models::lane_ending::LaneEnding;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::patch_operation::PatchOperation;
use crate::managers::models::patch_operation_kind::PatchOperationKind;
use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::prompt_layer::PromptLayer;
use crate::managers::models::report_status::ReportStatus;
use crate::managers::models::run_context::RunContext;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::scaffold_file::ScaffoldFile;
use crate::managers::models::scaffold_plan::ScaffoldPlan;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_format::StackPackFormat;
use crate::managers::models::stack_pack_lint::StackPackLint;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::stack_pack_scaffold::StackPackScaffold;
use crate::managers::models::tool_call::ToolCall;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::models::user_request::UserRequest;
use crate::managers::models::work_plan::WorkPlan;
use crate::managers::models::workspace_file::WorkspaceFile;

const APPLY_PATCH: &str = "apply_patch";
const SCAFFOLD: &str = "scaffold";
// How many times the implementor, once it reports a plan done, is sent the checks that fail, before the build ends as
// it is.
const IMPLEMENTOR_ROUNDS: i64 = 3;
// The lanes of the session's own agent, and of the implementor of its plans.
const ORCHESTRATOR_LANE_NAME: &str = "orchestrator";
const IMPLEMENTOR_LANE_NAME: &str = "implementor";
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
    work_plan_processor: Arc<WorkPlanProcessor>,
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
        work_plan_processor: Arc<WorkPlanProcessor>,
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
            work_plan_processor,
            max_turns,
            check_timeout,
        }
    }

    /// Runs the session's own agent, the orchestrator, on the person's request. Reports what happens on `events`; the
    /// run's start and end are the manager's to report. The engine is a shared singleton, which the ServiceLocator
    /// builds without the DataFacade, so the manager passes it in.
    pub async fn run(
        &self,
        session: &Session,
        user_request: &UserRequest,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<AgentRunOutcome> {
        let agent_assignment: AgentAssignment = AgentRunEngine::to_orchestrator_assignment(session);
        let (run_context, project_map_note): (RunContext, Option<ConversationItem>) = self
            .prepare_orchestrator(session, &agent_assignment, data_facade)
            .await?;

        let mut opening_items: Vec<ConversationItem> = Vec::with_capacity(2);
        opening_items.extend(project_map_note);
        opening_items.push(ConversationItem::UserMessage {
            text: user_request.text.clone(),
        });
        self.run_lane(
            session,
            &agent_assignment,
            &run_context,
            opening_items,
            events,
            data_facade,
        )
        .await
    }

    /// Builds the orchestrator's accepted plan: the harness scaffolds its components, then the implementor, on a lane of
    /// its own, builds the change. Once the implementor reports it done, the harness runs every check itself and sends
    /// it what fails, for a few rounds; a blocked report ends the build as it is.
    pub async fn implement_plan(
        &self,
        session: &Session,
        work_plan: &WorkPlan,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<ImplementationResult> {
        let agent_assignment: AgentAssignment = AgentAssignment {
            lane_name: IMPLEMENTOR_LANE_NAME.to_string(),
            role: AgentRole::Implementor,
            workspace_directory: session.workspace_directory.clone(),
        };
        let workspace_directory: &Path = &agent_assignment.workspace_directory;
        let (stack_pack, _): (Option<StackPack>, Option<String>) =
            self.detect_stack_pack(workspace_directory, data_facade).await?;
        let scaffold_outputs: Vec<String> = match stack_pack.as_ref() {
            Some(stack_pack) => {
                self.scaffold_components(workspace_directory, stack_pack, &work_plan.components, events)
                    .await?
            }
            None => Vec::new(),
        };

        // The map is built after scaffolding, so what the scaffolds added shows in the note.
        let (run_context, project_map_note): (RunContext, Option<ConversationItem>) = self
            .prepare_implementor(session, &agent_assignment, data_facade)
            .await?;
        let mut opening_items: Vec<ConversationItem> = Vec::with_capacity(2);
        opening_items.extend(project_map_note);
        opening_items.push(ConversationItem::UserMessage {
            text: self
                .tool_processor
                .format_implementor_brief(work_plan, &scaffold_outputs),
        });

        let mut run_summary: Option<RunSummary> = None;
        let mut report: Option<AgentReport> = None;
        let mut checks_passed: bool = false;
        for round in 1..=IMPLEMENTOR_ROUNDS {
            let agent_run_outcome: AgentRunOutcome = self
                .run_lane(
                    session,
                    &agent_assignment,
                    &run_context,
                    opening_items,
                    events,
                    data_facade,
                )
                .await?;
            run_summary = Some(match run_summary {
                Some(run_summary) => run_summary.plus(agent_run_outcome.run_summary),
                None => agent_run_outcome.run_summary,
            });
            report = match agent_run_outcome.lane_ending {
                Some(LaneEnding::Reported(agent_report)) => Some(agent_report),
                Some(LaneEnding::PlanAccepted(_)) | None => None,
            };

            let Some(stack_pack) = run_context.stack_pack.as_ref() else {
                checks_passed = true;
                break;
            };
            let (format_result, check_results): (FormatResult, Vec<CheckResult>) =
                self.format_and_check(workspace_directory, stack_pack).await?;
            checks_passed = check_results
                .iter()
                .all(|check_result: &CheckResult| check_result.passed);
            let blocked: bool = report
                .as_ref()
                .is_some_and(|agent_report: &AgentReport| agent_report.status == ReportStatus::Blocked);
            if checks_passed || blocked || round == IMPLEMENTOR_ROUNDS {
                break;
            }
            opening_items = vec![ConversationItem::HarnessNote {
                text: self
                    .tool_processor
                    .format_implementor_failures(&format_result, &check_results),
            }];
        }
        Ok(ImplementationResult {
            report,
            checks_passed,
            run_summary: run_summary.expect("the first round always runs"),
        })
    }

    // Scaffolds the plan's components, and reports each as the harness's own use of the scaffold tool. A component that can't be scaffolded after all (its plan checked out) is
    // told to the orchestrator, which writes it by hand.
    async fn scaffold_components(
        &self,
        workspace_directory: &Path,
        stack_pack: &StackPack,
        planned_components: &[PlannedComponent],
        events: &broadcast::Sender<SessionEvent>,
    ) -> ApplicationResult<Vec<String>> {
        let Some(stack_pack_scaffold) = stack_pack.scaffold.as_ref() else {
            return Ok(Vec::new());
        };
        // A component that lives within another comes after those that live within nothing, so the one it lives
        // within is there when it is scaffolded.
        let mut ordered_components: Vec<&PlannedComponent> = planned_components.iter().collect();
        ordered_components.sort_by_key(|planned_component: &&PlannedComponent| planned_component.within.is_some());

        let mut scaffold_outputs: Vec<String> = Vec::with_capacity(ordered_components.len());
        for planned_component in ordered_components {
            let target: String = match planned_component.within.as_deref() {
                Some(within) => format!("{} {} in {within}", planned_component.kind, planned_component.name),
                None => format!("{} {}", planned_component.kind, planned_component.name),
            };
            let scaffolded: ApplicationResult<String> = self
                .scaffold(
                    workspace_directory,
                    stack_pack_scaffold,
                    stack_pack.format.as_ref(),
                    &planned_component.kind,
                    &planned_component.name,
                    planned_component.within.as_deref(),
                )
                .await;
            let (scaffold_output, succeeded): (String, bool) = match scaffolded {
                Ok(scaffold_output) => (scaffold_output, true),
                Err(exception) => (
                    format!(
                        "The {target} could not be scaffolded ({}); write it by hand.",
                        self.to_tool_error_message(exception)?
                    ),
                    false,
                ),
            };
            events
                .send(SessionEvent::ToolUsed {
                    agent: None,
                    tool_name: SCAFFOLD.to_string(),
                    target,
                    succeeded,
                })
                .ok();
            scaffold_outputs.push(scaffold_output);
        }
        Ok(scaffold_outputs)
    }

    // The orchestrator's run context and the note of how the workspace changed since its last run. The session keeps
    // the orchestrator's map from run to run, so its requests' prefix (and its lane) carries over.
    async fn prepare_orchestrator(
        &self,
        session: &Session,
        agent_assignment: &AgentAssignment,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(RunContext, Option<ConversationItem>)> {
        let (stack_pack, rules): (Option<StackPack>, Option<String>) = self
            .detect_stack_pack(&agent_assignment.workspace_directory, data_facade)
            .await?;
        let (project_map, project_map_note): (Option<String>, Option<ConversationItem>) =
            match AgentRunEngine::get_project_map_source(stack_pack.as_ref()) {
                Some((language_name, stack_pack_project_map)) => {
                    let current_map: String = self
                        .build_project_map(
                            &agent_assignment.workspace_directory,
                            language_name,
                            stack_pack_project_map,
                        )
                        .await?;
                    let (layer_map, project_map_note): (String, Option<ConversationItem>) =
                        self.choose_project_map(session, current_map, data_facade).await?;
                    (Some(layer_map), project_map_note)
                }
                None => (None, None),
            };
        let run_context: RunContext =
            self.build_run_context(agent_assignment, stack_pack, rules.as_deref(), project_map.as_deref());
        Ok((run_context, project_map_note))
    }

    // The implementor's run context. Its requests open with the orchestrator's kept map, whose layer is cached already,
    // and a note of what changed since it was built (the plan's scaffolds, for one), unless there is so much that a map
    // of the workspace as it is now is the better layer. Nothing is kept: the orchestrator's notes still start from its
    // own last run.
    async fn prepare_implementor(
        &self,
        session: &Session,
        agent_assignment: &AgentAssignment,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(RunContext, Option<ConversationItem>)> {
        let (stack_pack, rules): (Option<StackPack>, Option<String>) = self
            .detect_stack_pack(&agent_assignment.workspace_directory, data_facade)
            .await?;
        let (project_map, project_map_note): (Option<String>, Option<ConversationItem>) =
            match AgentRunEngine::get_project_map_source(stack_pack.as_ref()) {
                Some((language_name, stack_pack_project_map)) => {
                    let current_map: String = self
                        .build_project_map(
                            &agent_assignment.workspace_directory,
                            language_name,
                            stack_pack_project_map,
                        )
                        .await?;
                    let session_project_maps: SessionProjectMaps =
                        data_facade.get_session_project_maps(&session.id).await?;
                    match session_project_maps.layer_map {
                        Some(layer_map) if layer_map == current_map => (Some(layer_map), None),
                        Some(layer_map) if !self.project_map_processor.should_rebuild(&layer_map, &current_map) => {
                            let changes: String = self.project_map_processor.describe_changes(&layer_map, &current_map);
                            (Some(layer_map), Some(ConversationItem::HarnessNote { text: changes }))
                        }
                        Some(_) | None => (Some(current_map), None),
                    }
                }
                None => (None, None),
            };
        let run_context: RunContext =
            self.build_run_context(agent_assignment, stack_pack, rules.as_deref(), project_map.as_deref());
        Ok((run_context, project_map_note))
    }

    // The session's own agent.
    fn to_orchestrator_assignment(session: &Session) -> AgentAssignment {
        AgentAssignment {
            lane_name: ORCHESTRATOR_LANE_NAME.to_string(),
            role: AgentRole::Orchestrator,
            workspace_directory: session.workspace_directory.clone(),
        }
    }

    // One agent's turns on its lane, to completion. Each turn sends the lane's whole conversation (the API keeps
    // nothing), does what the model asked and sends what came of it back, until the model answers without asking for
    // anything, or the harness accepts its plan. Every item is appended to the lane's conversation in the session store
    // as it happens.
    async fn run_lane(
        &self,
        session: &Session,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        opening_items: Vec<ConversationItem>,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<AgentRunOutcome> {
        let lane_name: &str = &agent_assignment.lane_name;
        let mut conversation_items: Vec<ConversationItem> =
            data_facade.get_conversation_items(&session.id, lane_name).await?;
        // A run cancelled midway can leave tool calls unanswered, which the API refuses: they are answered first.
        let unfinished_tool_call_outputs: Vec<ConversationItem> =
            self.tool_processor.answer_unfinished_tool_calls(&conversation_items);
        if !unfinished_tool_call_outputs.is_empty() {
            data_facade
                .append_conversation_items(&session.id, lane_name, &unfinished_tool_call_outputs)
                .await?;
            conversation_items.extend(unfinished_tool_call_outputs);
        }
        let mut new_items: Vec<ConversationItem> = Vec::with_capacity(opening_items.len() + 1);
        // A new lane's role sets how hard it reasons, once, in its conversation.
        if conversation_items.is_empty()
            && let Some(reasoning_effort) = self.prompt_processor.get_reasoning_effort(agent_assignment.role)
        {
            new_items.push(ConversationItem::ConfigurationUpdate { reasoning_effort });
        }
        new_items.extend(opening_items);
        data_facade
            .append_conversation_items(&session.id, lane_name, &new_items)
            .await?;
        conversation_items.extend(new_items);

        let lane_id: String = format!("{}/{}", session.id, lane_name);
        let agent: Option<String> = AgentRunEngine::to_agent(agent_assignment);
        let mut usage: ModelUsage = ModelUsage::default();
        for turn in 1..=self.max_turns {
            let model_response: ModelResponse = self
                .openai_responses_gateway
                .create_response(&ModelRequest {
                    lane_id: lane_id.clone(),
                    prompt_layers: run_context.prompt_layers.clone(),
                    items: conversation_items.clone(),
                    tools: run_context.tools.clone(),
                    allowed_tool_names: run_context.allowed_tool_names.clone(),
                    service_tier: session.service_tier,
                })
                .await?;
            usage = usage.plus(model_response.usage);
            data_facade
                .append_usage_record(&AgentRunEngine::to_usage_record(
                    session,
                    agent_assignment,
                    &model_response,
                ))
                .await?;
            data_facade
                .append_conversation_items(&session.id, lane_name, &model_response.output_items)
                .await?;
            conversation_items.extend(model_response.output_items.iter().cloned());

            if let Some(text) = model_response.text.as_ref() {
                events
                    .send(SessionEvent::AssistantReplied {
                        agent: agent.clone(),
                        text: text.clone(),
                    })
                    .ok();
            }
            let run_summary: RunSummary = RunSummary {
                turns: turn,
                usage,
                service_tier: model_response.service_tier,
            };
            if model_response.tool_calls.is_empty() {
                return Ok(AgentRunOutcome {
                    run_summary,
                    lane_ending: None,
                });
            }

            let mut tool_outputs: Vec<ConversationItem> = Vec::with_capacity(model_response.tool_calls.len());
            let mut lane_ending: Option<LaneEnding> = None;
            for tool_call in &model_response.tool_calls {
                let (tool_output, tool_lane_ending): (ConversationItem, Option<LaneEnding>) =
                    self.use_tool(agent_assignment, run_context, tool_call, events).await?;
                tool_outputs.push(tool_output);
                lane_ending = lane_ending.or(tool_lane_ending);
            }
            data_facade
                .append_conversation_items(&session.id, lane_name, &tool_outputs)
                .await?;
            conversation_items.extend(tool_outputs);
            // An accepted plan or a report ends the lane here; the workflow carries on from it.
            if lane_ending.is_some() {
                return Ok(AgentRunOutcome {
                    run_summary,
                    lane_ending,
                });
            }
        }

        Err(ApplicationBaseException::new(
            AgentRunEngineException::MaxTurnsReached,
            format!("The run stopped after {} model turns without finishing", self.max_turns),
            ContextualData::new()
                .with("session_id", session.id.as_str())
                .with("lane_name", lane_name)
                .with("max_turns", self.max_turns),
        ))
    }

    // The workspace's stack (the first pack whose files it has), and its rules.
    async fn detect_stack_pack(
        &self,
        workspace_directory: &Path,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(Option<StackPack>, Option<String>)> {
        for stack_pack in data_facade.get_stack_packs().await? {
            if self.has_files(workspace_directory, &stack_pack.detect_files).await? {
                let rules: String = data_facade.get_stack_pack_rules(&stack_pack).await?;
                return Ok((Some(stack_pack), Some(rules)));
            }
        }
        Ok((None, None))
    }

    // The prompt layers and tools every turn of the agent's run sends, and the tools its role may call.
    fn build_run_context(
        &self,
        agent_assignment: &AgentAssignment,
        stack_pack: Option<StackPack>,
        rules: Option<&str>,
        project_map: Option<&str>,
    ) -> RunContext {
        let mut prompt_layers: Vec<PromptLayer> =
            self.prompt_processor
                .build_prompt_layers(stack_pack.as_ref(), rules, project_map);
        prompt_layers.extend(self.prompt_processor.build_assignment_layer(agent_assignment.role));
        RunContext {
            prompt_layers,
            tools: self.tool_processor.get_tool_definitions(),
            allowed_tool_names: Some(self.tool_processor.get_allowed_tool_names(agent_assignment.role)),
            stack_pack,
        }
    }

    // The pack's grammar and project map settings, if it has both.
    fn get_project_map_source(stack_pack: Option<&StackPack>) -> Option<(&str, &StackPackProjectMap)> {
        stack_pack
            .and_then(|stack_pack: &StackPack| stack_pack.language.as_deref().zip(stack_pack.project_map.as_ref()))
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

    // Does what one tool call asks, reports it, and returns the output the model is sent for it, and the plan it
    // submitted if the harness accepted it. A tool the agent's role may not call, or a change to a file it doesn't own,
    // is refused, and the model is told why.
    async fn use_tool(
        &self,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        tool_call: &ToolCall,
        events: &broadcast::Sender<SessionEvent>,
    ) -> ApplicationResult<(ConversationItem, Option<LaneEnding>)> {
        match tool_call {
            ToolCall::Function {
                call_id,
                name,
                arguments,
            } => {
                let (target, output, succeeded, lane_ending): (String, String, bool, Option<LaneEnding>) = match self
                    .check_tool_allowed(agent_assignment, name)
                    .and_then(|(): ()| self.tool_processor.to_tool_request(name, arguments))
                {
                    Ok(tool_request) => {
                        let (output, succeeded): (String, bool) = self
                            .use_function_tool(agent_assignment, run_context, &tool_request)
                            .await?;
                        let lane_ending: Option<LaneEnding> = match &tool_request {
                            ToolRequest::SubmitPlan { work_plan } if succeeded => {
                                Some(LaneEnding::PlanAccepted(work_plan.clone()))
                            }
                            ToolRequest::Report { agent_report } => Some(LaneEnding::Reported(agent_report.clone())),
                            _ => None,
                        };
                        (
                            self.tool_processor.describe_target(&tool_request),
                            output,
                            succeeded,
                            lane_ending,
                        )
                    }
                    Err(exception) => (String::new(), self.to_tool_error(exception)?, false, None),
                };
                events
                    .send(SessionEvent::ToolUsed {
                        agent: AgentRunEngine::to_agent(agent_assignment),
                        tool_name: name.clone(),
                        target,
                        succeeded,
                    })
                    .ok();
                Ok((
                    ConversationItem::FunctionCallOutput {
                        call_id: call_id.clone(),
                        output,
                    },
                    lane_ending,
                ))
            }
            ToolCall::ApplyPatch {
                call_id,
                patch_operation,
            } => {
                let (output, succeeded): (String, bool) =
                    match self.apply_allowed_patch(agent_assignment, patch_operation).await {
                        Ok(output) => (output, true),
                        // The model knows this error format from the reference applier.
                        Err(exception) => (format!("Error: {}", self.to_tool_error_message(exception)?), false),
                    };
                events
                    .send(SessionEvent::ToolUsed {
                        agent: AgentRunEngine::to_agent(agent_assignment),
                        tool_name: APPLY_PATCH.to_string(),
                        target: format!("{} {}", patch_operation.kind.name(), patch_operation.path),
                        succeeded,
                    })
                    .ok();
                Ok((
                    ConversationItem::ApplyPatchCallOutput {
                        call_id: call_id.clone(),
                        succeeded,
                        output,
                    },
                    None,
                ))
            }
        }
    }

    // The tool's output and whether it did what was asked.
    async fn use_function_tool(
        &self,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        tool_request: &ToolRequest,
    ) -> ApplicationResult<(String, bool)> {
        let workspace_directory: &Path = &agent_assignment.workspace_directory;
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
                            .scaffold(
                                workspace_directory,
                                stack_pack_scaffold,
                                run_context
                                    .stack_pack
                                    .as_ref()
                                    .and_then(|stack_pack: &StackPack| stack_pack.format.as_ref()),
                                kind,
                                name,
                                within.as_deref(),
                            )
                            .await
                        {
                            Ok(output) => Ok((output, true)),
                            Err(exception) => Ok((self.to_tool_error(exception)?, false)),
                        }
                    }
                }
            }
            ToolRequest::Report { .. } => Ok(("Reported.".to_string(), true)),
            ToolRequest::SubmitPlan { work_plan } => {
                match self.check_work_plan(agent_assignment, run_context, work_plan).await {
                    Ok(()) => Ok((self.tool_processor.format_plan_accepted(work_plan), true)),
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
                    let (format_result, check_results): (FormatResult, Vec<CheckResult>) =
                        self.format_and_check(workspace_directory, stack_pack).await?;
                    let succeeded: bool = check_results
                        .iter()
                        .all(|check_result: &CheckResult| check_result.passed);
                    Ok((
                        self.tool_processor.format_checks_run(&format_result, &check_results),
                        succeeded,
                    ))
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
        stack_pack_format: Option<&StackPackFormat>,
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

        let format_result: FormatResult = self.format_workspace(workspace_directory, stack_pack_format).await?;

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

    // Runs the stack's formatters, and finds which of their files they changed (by reading them before and after), so
    // the agent reads those again before it patches them. A formatter that fails (on code that doesn't parse or
    // compile) is reported as such; the checks say why. Each runs even if one before it failed.
    async fn format_workspace(
        &self,
        workspace_directory: &Path,
        stack_pack_format: Option<&StackPackFormat>,
    ) -> ApplicationResult<FormatResult> {
        let Some(stack_pack_format) = stack_pack_format else {
            return Ok(FormatResult {
                succeeded: true,
                reformatted_paths: Vec::new(),
            });
        };
        let (_, before): (Vec<String>, Vec<WorkspaceFile>) = self
            .read_source_files(workspace_directory, &stack_pack_format.source_globs)
            .await?;
        let mut succeeded: bool = true;
        for command in &stack_pack_format.commands {
            succeeded &= match self
                .command_runner_gateway
                .run(workspace_directory, command, self.check_timeout)
                .await
            {
                Ok(command_result) => command_result.exit_code == Some(0),
                Err(exception) => {
                    self.to_tool_error_message(exception)?;
                    false
                }
            };
        }
        let (_, after): (Vec<String>, Vec<WorkspaceFile>) = self
            .read_source_files(workspace_directory, &stack_pack_format.source_globs)
            .await?;

        let before_contents: HashMap<&str, &str> = before
            .iter()
            .map(|workspace_file: &WorkspaceFile| (workspace_file.path.as_str(), workspace_file.content.as_str()))
            .collect();
        let reformatted_paths: Vec<String> = after
            .iter()
            .filter(|workspace_file: &&WorkspaceFile| {
                before_contents.get(workspace_file.path.as_str()) != Some(&workspace_file.content.as_str())
            })
            .map(|workspace_file: &WorkspaceFile| workspace_file.path.clone())
            .collect();
        Ok(FormatResult {
            succeeded,
            reformatted_paths,
        })
    }

    // The stack's formatter, then every check, as one run of the checks: the formatter first, so no check fails on
    // formatting an agent would otherwise fix by hand.
    async fn format_and_check(
        &self,
        workspace_directory: &Path,
        stack_pack: &StackPack,
    ) -> ApplicationResult<(FormatResult, Vec<CheckResult>)> {
        let format_result: FormatResult = self
            .format_workspace(workspace_directory, stack_pack.format.as_ref())
            .await?;
        let check_results: Vec<CheckResult> = self.run_checks(workspace_directory, stack_pack).await?;
        Ok((format_result, check_results))
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

    // Checks a plan against the workspace and the stack: each component is scaffolded on paper (its kind exists, the
    // component it lives within exists or is planned too, none of its files exists yet), then the WorkPlanProcessor
    // checks the rest. Every problem is reported in one InvalidWorkPlan.
    async fn check_work_plan(
        &self,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        work_plan: &WorkPlan,
    ) -> ApplicationResult<()> {
        let workspace_directory: &Path = &agent_assignment.workspace_directory;
        let Some(stack_pack) = run_context.stack_pack.as_ref() else {
            return Err(ApplicationBaseException::new(
                AgentRunEngineException::InvalidWorkPlan,
                "No stack pack matches the workspace, so there is nothing to check a plan against: make the change \
                 yourself",
                ContextualData::new(),
            ));
        };

        let mut component_errors: Vec<String> = Vec::new();
        let mut scaffold_plans: Vec<ScaffoldPlan> = Vec::with_capacity(work_plan.components.len());
        for planned_component in &work_plan.components {
            let Some(stack_pack_scaffold) = stack_pack.scaffold.as_ref() else {
                component_errors.push("the stack has no scaffolds, so a plan can't have components".to_string());
                break;
            };
            match self.scaffold_processor.plan(
                stack_pack_scaffold,
                &planned_component.kind,
                &planned_component.name,
                planned_component.within.as_deref(),
            ) {
                Ok(scaffold_plan) => scaffold_plans.push(scaffold_plan),
                Err(exception) if AgentRunEngine::is_told_to_model(exception.kind()) => {
                    component_errors.push(format!(
                        "the {} {}: {}",
                        planned_component.kind,
                        planned_component.name,
                        exception.message()
                    ));
                }
                Err(exception) => return Err(exception),
            }
        }

        let created_paths: Vec<String> = scaffold_plans
            .iter()
            .flat_map(|scaffold_plan: &ScaffoldPlan| scaffold_plan.created_files.iter())
            .map(|scaffold_file: &ScaffoldFile| scaffold_file.path.clone())
            .collect();
        for scaffold_plan in &scaffold_plans {
            if let Some(within_path) = scaffold_plan.within_path.as_deref()
                && !created_paths
                    .iter()
                    .any(|created_path: &String| created_path == within_path)
                && !self
                    .workspace_gateway
                    .file_exists(workspace_directory, within_path)
                    .await?
            {
                component_errors.push(format!(
                    "{within_path} doesn't exist and isn't planned: a component lives within a component that does"
                ));
            }
            for scaffold_file in &scaffold_plan.created_files {
                if self
                    .workspace_gateway
                    .file_exists(workspace_directory, &scaffold_file.path)
                    .await?
                {
                    component_errors.push(format!(
                        "{} already exists: the component is there already",
                        scaffold_file.path
                    ));
                }
            }
        }

        self.work_plan_processor.check(work_plan, component_errors)
    }

    // The InvalidToolCall-like refusal of a tool the agent's role may not call.
    fn check_tool_allowed(&self, agent_assignment: &AgentAssignment, tool_name: &str) -> ApplicationResult<()> {
        if self.tool_processor.is_tool_allowed(agent_assignment.role, tool_name) {
            return Ok(());
        }
        let allowed_tool_names: Vec<String> = self.tool_processor.get_allowed_tool_names(agent_assignment.role);
        Err(ApplicationBaseException::new(
            AgentRunEngineException::ToolNotAllowed,
            format!(
                "As the {}, you may not call {tool_name}; you may call {}",
                agent_assignment.role.name(),
                allowed_tool_names.join(", ")
            ),
            ContextualData::new()
                .with("lane_name", agent_assignment.lane_name.as_str())
                .with("tool_name", tool_name),
        ))
    }

    // A patch's file operation, if the agent's role may patch.
    async fn apply_allowed_patch(
        &self,
        agent_assignment: &AgentAssignment,
        patch_operation: &PatchOperation,
    ) -> ApplicationResult<String> {
        self.check_tool_allowed(agent_assignment, APPLY_PATCH)?;
        self.apply_patch(&agent_assignment.workspace_directory, patch_operation)
            .await
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
                | AgentRunEngineException::ScaffoldNotApplicable
                | AgentRunEngineException::ToolNotAllowed
                | AgentRunEngineException::InvalidWorkPlan => true,
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

    // The agent as events name it: the orchestrator is the session's own, so it is unnamed.
    fn to_agent(agent_assignment: &AgentAssignment) -> Option<String> {
        match agent_assignment.role {
            AgentRole::Orchestrator => None,
            AgentRole::Implementor | AgentRole::Reviewer => Some(agent_assignment.lane_name.clone()),
        }
    }

    fn to_usage_record(
        session: &Session,
        agent_assignment: &AgentAssignment,
        model_response: &ModelResponse,
    ) -> UsageRecord {
        UsageRecord {
            session_id: session.id.clone(),
            lane_name: agent_assignment.lane_name.clone(),
            role: agent_assignment.role,
            response_id: model_response.id.clone(),
            model: model_response.model.clone(),
            requested_service_tier: session.service_tier,
            served_service_tier: model_response.service_tier,
            usage: model_response.usage,
            recorded_on: Utc::now(),
        }
    }
}
