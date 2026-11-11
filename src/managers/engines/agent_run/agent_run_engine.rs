use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::broadcast;

use crate::managers::common::matchers::path_glob_matcher::PathGlobMatcher;
use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::agent_run::processors::architecture_lint::architecture_lint_processor::ArchitectureLintProcessor;
use crate::managers::engines::agent_run::processors::brief::brief_processor::BriefProcessor;
use crate::managers::engines::agent_run::processors::diff::diff_processor::DiffProcessor;
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
use crate::managers::models::agent_assignment::AgentAssignment;
use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::agent_role::AgentRole;
use crate::managers::models::agent_run_outcome::AgentRunOutcome;
use crate::managers::models::check_result::CheckResult;
use crate::managers::models::compacted_conversation::CompactedConversation;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::finding_severity::FindingSeverity;
use crate::managers::models::format_result::FormatResult;
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::implementation_result::ImplementationResult;
use crate::managers::models::lane_ending::LaneEnding;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::patch_operation::PatchOperation;
use crate::managers::models::patch_operation_kind::PatchOperationKind;
use crate::managers::models::project_map_role::ProjectMapRole;
use crate::managers::models::prompt_layer::PromptLayer;
use crate::managers::models::reasoning_effort::ReasoningEffort;
use crate::managers::models::report_status::ReportStatus;
use crate::managers::models::review::Review;
use crate::managers::models::review_finding::ReviewFinding;
use crate::managers::models::run_context::RunContext;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_project_maps::SessionProjectMaps;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_format::StackPackFormat;
use crate::managers::models::stack_pack_lint::StackPackLint;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::tool_call::ToolCall;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::transcript_speaker::TranscriptSpeaker;
use crate::managers::models::usage_record::UsageRecord;
use crate::managers::models::user_request::UserRequest;
use crate::managers::models::workspace_file::WorkspaceFile;

const APPLY_PATCH: &str = "apply_patch";
// How many times the implementor, once it reports a brief done, is sent the checks that fail, before the build ends as
// it is.
const IMPLEMENTOR_ROUNDS: i64 = 3;
// The session's own agent's lane, and the start of the lanes of each brief's implementor and reviewer
// (`implementor_1`, `reviewer_1`, ...): each brief is built and reviewed in fresh conversations.
const ORCHESTRATOR_LANE_NAME: &str = "orchestrator";
const IMPLEMENTOR_LANE_NAME: &str = "implementor";
const REVIEWER_LANE_NAME: &str = "reviewer";
// How many of a brief's files are read into the implementor's first message; it reads any others itself.
const MAX_BRIEF_FILES: usize = 24;
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
    brief_processor: Arc<BriefProcessor>,
    diff_processor: Arc<DiffProcessor>,
    max_turns: i64,
    check_timeout: Duration,
    orchestrator_compact_tokens: i64,
    lane_compact_tokens: i64,
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
        brief_processor: Arc<BriefProcessor>,
        diff_processor: Arc<DiffProcessor>,
        max_turns: i64,
        check_timeout: Duration,
        orchestrator_compact_tokens: i64,
        lane_compact_tokens: i64,
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
            brief_processor,
            diff_processor,
            max_turns,
            check_timeout,
            orchestrator_compact_tokens,
            lane_compact_tokens,
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

    /// Builds the orchestrator's accepted brief, the session's `brief_number`th: the implementor, on a fresh lane of its
    /// own, plans and builds the change from the brief alone (and the files it names, read into its first message).
    /// Once the implementor reports it done, the harness runs every check itself and sends it what fails, for a few
    /// rounds; a blocked report ends the build as it is. A change that is done, with every check passing, is reviewed,
    /// and what the review says must be fixed goes back to the implementor, once.
    pub async fn implement_brief(
        &self,
        session: &Session,
        user_request: &UserRequest,
        implementation_brief: &ImplementationBrief,
        brief_number: usize,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<ImplementationResult> {
        let agent_assignment: AgentAssignment = AgentAssignment {
            lane_name: format!("{IMPLEMENTOR_LANE_NAME}_{brief_number}"),
            role: AgentRole::Implementor,
            workspace_directory: session.workspace_directory.clone(),
        };
        let workspace_directory: &Path = &agent_assignment.workspace_directory;
        // The workspace before the change, which the review's diff is made against.
        let before_change: Vec<WorkspaceFile> = self.snapshot_workspace(workspace_directory).await?;
        let (run_context, project_map_note): (RunContext, Option<ConversationItem>) =
            self.prepare_assignment(session, &agent_assignment, data_facade).await?;
        let (brief_files, missing_files): (Vec<WorkspaceFile>, Vec<(String, String)>) =
            self.read_brief_files(workspace_directory, implementation_brief).await?;
        let mut opening_items: Vec<ConversationItem> = Vec::with_capacity(2);
        opening_items.extend(project_map_note);
        opening_items.push(ConversationItem::UserMessage {
            text: self
                .tool_processor
                .format_implementor_brief(implementation_brief, &brief_files, &missing_files),
        });
        let (mut report, mut checks_passed, mut run_summary): (Option<AgentReport>, bool, RunSummary) = self
            .run_implementor(
                session,
                &agent_assignment,
                &run_context,
                opening_items,
                events,
                data_facade,
            )
            .await?;

        let done: bool = checks_passed
            && report
                .as_ref()
                .is_some_and(|agent_report: &AgentReport| agent_report.status == ReportStatus::Done);
        let diff: String = if done {
            let after_change: Vec<WorkspaceFile> = self.snapshot_workspace(workspace_directory).await?;
            self.diff_processor.diff(&before_change, &after_change)
        } else {
            String::new()
        };
        if diff.is_empty() {
            return Ok(ImplementationResult {
                report,
                checks_passed,
                review: None,
                fixed_after_review: false,
                run_summary,
            });
        }

        let (review, review_summary): (Option<Review>, RunSummary) = self
            .review_change(
                session,
                user_request,
                implementation_brief,
                brief_number,
                &diff,
                events,
                data_facade,
            )
            .await?;
        run_summary = run_summary.plus(review_summary);
        let must_fix: bool = review.as_ref().is_some_and(|review: &Review| {
            review
                .findings
                .iter()
                .any(|finding: &ReviewFinding| finding.severity == FindingSeverity::High)
        });
        if must_fix && let Some(review) = review.as_ref() {
            let fixes: Vec<ConversationItem> = vec![ConversationItem::HarnessNote {
                text: self.tool_processor.format_review_fixes(review),
            }];
            let (fixed_report, fixed_checks_passed, fix_summary): (Option<AgentReport>, bool, RunSummary) = self
                .run_implementor(session, &agent_assignment, &run_context, fixes, events, data_facade)
                .await?;
            report = fixed_report;
            checks_passed = fixed_checks_passed;
            run_summary = run_summary.plus(fix_summary);
        }
        Ok(ImplementationResult {
            report,
            checks_passed,
            review,
            fixed_after_review: must_fix,
            run_summary,
        })
    }

    /// Gives the orchestrator a turn to tell the person how its brief went: what the implementor built (or what it
    /// needs), and what the review found, told in its conversation as a harness note. In this turn it reads but doesn't
    /// brief again: a new brief waits for the person's answer.
    pub async fn reply_after_build(
        &self,
        session: &Session,
        implementation_result: &ImplementationResult,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<AgentRunOutcome> {
        let agent_assignment: AgentAssignment = AgentRunEngine::to_orchestrator_assignment(session);
        let (mut run_context, project_map_note): (RunContext, Option<ConversationItem>) = self
            .prepare_orchestrator(session, &agent_assignment, data_facade)
            .await?;
        run_context.allowed_tool_names = Some(self.tool_processor.get_reply_tool_names());

        let mut opening_items: Vec<ConversationItem> = Vec::with_capacity(2);
        opening_items.extend(project_map_note);
        opening_items.push(ConversationItem::HarnessNote {
            text: self.tool_processor.format_implementation_result(implementation_result),
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

    /// Compacts the orchestrator's conversation if its last request (`last_input_tokens` input tokens) passed the
    /// threshold, so the session's next requests stay short. The manager calls it once a run has ended: compacting
    /// takes a while, and the person is reading the reply meanwhile. Whether it compacted.
    pub async fn compact_orchestrator(
        &self,
        session: &Session,
        last_input_tokens: i64,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<bool> {
        if last_input_tokens < self.orchestrator_compact_tokens {
            return Ok(false);
        }
        let agent_assignment: AgentAssignment = AgentRunEngine::to_orchestrator_assignment(session);
        let run_context: RunContext = self
            .prepare_kept_orchestrator(session, &agent_assignment, data_facade)
            .await?;
        let conversation_items: Vec<ConversationItem> = data_facade
            .get_conversation_items(&session.id, &agent_assignment.lane_name)
            .await?;
        self.compact_lane(
            session,
            &agent_assignment,
            &run_context,
            &conversation_items,
            events,
            data_facade,
        )
        .await?;
        Ok(true)
    }

    // The lane's conversation, compacted by the API, in place of everything in it so far: what it gives back is
    // appended as one Compaction, then the role's reasoning effort again (the API won't take one before a compaction),
    // in one write, so a compaction cut short leaves the lane as it was. Its usage is kept, and reported. The lane's
    // conversation from now on.
    async fn compact_lane(
        &self,
        session: &Session,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        conversation_items: &[ConversationItem],
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<Vec<ConversationItem>> {
        let compacted_conversation: CompactedConversation = self
            .openai_responses_gateway
            .compact_conversation(&ModelRequest {
                lane_id: format!("{}/{}", session.id, agent_assignment.lane_name),
                prompt_layers: run_context.prompt_layers.clone(),
                items: conversation_items.to_vec(),
                tools: run_context.tools.clone(),
                allowed_tool_names: None,
                service_tier: session.service_tier,
            })
            .await?;

        let mut compacted_items: Vec<ConversationItem> = vec![ConversationItem::Compaction {
            items: compacted_conversation.items,
        }];
        compacted_items.extend(
            self.prompt_processor
                .get_reasoning_effort(agent_assignment.role)
                .map(|reasoning_effort: ReasoningEffort| ConversationItem::ConfigurationUpdate { reasoning_effort }),
        );
        data_facade
            .append_conversation_items(&session.id, &agent_assignment.lane_name, &compacted_items)
            .await?;
        data_facade
            .append_usage_record(&UsageRecord {
                session_id: session.id.clone(),
                lane_name: agent_assignment.lane_name.clone(),
                role: agent_assignment.role,
                response_id: compacted_conversation.id,
                model: compacted_conversation.model,
                requested_service_tier: session.service_tier,
                // A compaction doesn't say which tier served it.
                served_service_tier: None,
                usage: compacted_conversation.usage,
                recorded_on: Utc::now(),
            })
            .await?;
        events
            .send(SessionEvent::LaneCompacted {
                agent: AgentRunEngine::to_agent(agent_assignment),
                usage: compacted_conversation.usage,
            })
            .ok();
        Ok(compacted_items)
    }

    // The orchestrator's run context as its last run left it: the session's kept project map, with nothing built or
    // kept, so a compaction between runs changes nothing a run sees.
    async fn prepare_kept_orchestrator(
        &self,
        session: &Session,
        agent_assignment: &AgentAssignment,
        data_facade: &DataFacade,
    ) -> ApplicationResult<RunContext> {
        let (stack_pack, rules): (Option<StackPack>, Option<String>) = self
            .detect_stack_pack(&agent_assignment.workspace_directory, data_facade)
            .await?;
        let project_map: Option<String> = match AgentRunEngine::get_project_map_source(stack_pack.as_ref()) {
            Some(_) => data_facade.get_session_project_maps(&session.id).await?.layer_map,
            None => None,
        };
        Ok(self.build_run_context(agent_assignment, stack_pack, rules.as_deref(), project_map.as_deref()))
    }

    // The files a brief names, read, up to MAX_BRIEF_FILES; one that can't be read (one the change will create, a path
    // outside the workspace) is listed with why, for the implementor to know.
    async fn read_brief_files(
        &self,
        workspace_directory: &Path,
        implementation_brief: &ImplementationBrief,
    ) -> ApplicationResult<(Vec<WorkspaceFile>, Vec<(String, String)>)> {
        let mut brief_files: Vec<WorkspaceFile> = Vec::new();
        let mut missing_files: Vec<(String, String)> = Vec::new();
        for path in implementation_brief.files.iter().take(MAX_BRIEF_FILES) {
            match self.workspace_gateway.read_file(workspace_directory, path).await {
                Ok(workspace_file) => brief_files.push(workspace_file),
                Err(exception) => missing_files.push((path.clone(), self.to_tool_error_message(exception)?)),
            }
        }
        Ok((brief_files, missing_files))
    }

    // The implementor's rounds: it builds until it reports, then the harness runs every check, and sends it what fails,
    // until every check passes, it reports itself blocked, or the rounds run out. Its last report, whether the checks
    // passed, and what the rounds cost.
    async fn run_implementor(
        &self,
        session: &Session,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        opening_items: Vec<ConversationItem>,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(Option<AgentReport>, bool, RunSummary)> {
        let workspace_directory: &Path = &agent_assignment.workspace_directory;
        let mut opening_items: Vec<ConversationItem> = opening_items;
        let mut run_summary: Option<RunSummary> = None;
        let mut report: Option<AgentReport> = None;
        let mut checks_passed: bool = false;
        for round in 1..=IMPLEMENTOR_ROUNDS {
            let agent_run_outcome: AgentRunOutcome = self
                .run_lane(
                    session,
                    agent_assignment,
                    run_context,
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
                Some(LaneEnding::BriefAccepted(_)) | Some(LaneEnding::Reviewed(_)) | None => None,
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
        Ok((report, checks_passed, run_summary.expect("the first round always runs")))
    }

    // The reviewer, on a fresh lane of its own for the brief, reads the change (the person's last message, the brief,
    // the diff, and whatever else it needs) and ends with its review; none if it ended without one.
    #[allow(clippy::too_many_arguments)]
    async fn review_change(
        &self,
        session: &Session,
        user_request: &UserRequest,
        implementation_brief: &ImplementationBrief,
        brief_number: usize,
        diff: &str,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(Option<Review>, RunSummary)> {
        let agent_assignment: AgentAssignment = AgentAssignment {
            lane_name: format!("{REVIEWER_LANE_NAME}_{brief_number}"),
            role: AgentRole::Reviewer,
            workspace_directory: session.workspace_directory.clone(),
        };
        let (run_context, project_map_note): (RunContext, Option<ConversationItem>) =
            self.prepare_assignment(session, &agent_assignment, data_facade).await?;
        let mut opening_items: Vec<ConversationItem> = Vec::with_capacity(2);
        opening_items.extend(project_map_note);
        opening_items.push(ConversationItem::UserMessage {
            text: self
                .tool_processor
                .format_review_brief(&user_request.text, implementation_brief, diff),
        });
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
        let review: Option<Review> = match agent_run_outcome.lane_ending {
            Some(LaneEnding::Reviewed(review)) => Some(review),
            Some(LaneEnding::BriefAccepted(_)) | Some(LaneEnding::Reported(_)) | None => None,
        };
        Ok((review, agent_run_outcome.run_summary))
    }

    // Every text file of the workspace (what .gitignore ignores left out), as it is now. A file that isn't text, or is
    // protected (.env files), isn't part of it.
    async fn snapshot_workspace(&self, workspace_directory: &Path) -> ApplicationResult<Vec<WorkspaceFile>> {
        let paths: Vec<String> = self.workspace_gateway.list_files(workspace_directory, ".").await?;
        let mut workspace_files: Vec<WorkspaceFile> = Vec::with_capacity(paths.len());
        for path in &paths {
            match self.workspace_gateway.read_file(workspace_directory, path).await {
                Ok(workspace_file) => workspace_files.push(workspace_file),
                Err(exception)
                    if matches!(
                        exception.kind(),
                        ExceptionKind::WorkspaceGateway(
                            WorkspaceGatewayException::NotTextFile
                                | WorkspaceGatewayException::ProtectedFile
                                | WorkspaceGatewayException::FileNotFound
                        )
                    ) => {}
                Err(exception) => return Err(exception),
            }
        }
        Ok(workspace_files)
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

    // The implementor's or the reviewer's run context. Its requests open with the orchestrator's kept map, whose layer is
    // cached already, and a note of what changed since it was built (the plan's change, for the reviewer), unless
    // there is so much that a map of the workspace as it is now is the better layer. Nothing is kept: the orchestrator's
    // notes still start from its own last run.
    async fn prepare_assignment(
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
                // What the orchestrator says is said to the person: it is their conversation's.
                if agent_assignment.role == AgentRole::Orchestrator {
                    data_facade
                        .append_transcript_entry(
                            &session.id,
                            &TranscriptEntry {
                                speaker: TranscriptSpeaker::Assistant,
                                text: text.clone(),
                                said_on: Utc::now(),
                            },
                        )
                        .await?;
                }
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
                    last_input_tokens: model_response.usage.input_tokens,
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
                    last_input_tokens: model_response.usage.input_tokens,
                });
            }
            // A task long enough to near the model's limit goes on from its conversation compacted. If that fails, it
            // goes on as it is: the limit may never be reached.
            if model_response.usage.input_tokens >= self.lane_compact_tokens {
                match self
                    .compact_lane(
                        session,
                        agent_assignment,
                        run_context,
                        &conversation_items,
                        events,
                        data_facade,
                    )
                    .await
                {
                    Ok(compacted_items) => conversation_items = compacted_items,
                    Err(exception) => ApplicationLogger::error(
                        "A lane's conversation could not be compacted; it goes on as it is",
                        ContextualData::new()
                            .with("session_id", session.id.as_str())
                            .with("lane_name", lane_name),
                        Some(&exception),
                    ),
                }
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
                    .check_tool_allowed(agent_assignment, run_context, name)
                    .and_then(|(): ()| self.tool_processor.to_tool_request(name, arguments))
                {
                    Ok(tool_request) => {
                        let (output, succeeded): (String, bool) = self
                            .use_function_tool(agent_assignment, run_context, &tool_request)
                            .await?;
                        let lane_ending: Option<LaneEnding> = match &tool_request {
                            ToolRequest::BriefImplementor { implementation_brief } if succeeded => {
                                Some(LaneEnding::BriefAccepted(implementation_brief.clone()))
                            }
                            ToolRequest::Report { agent_report } => Some(LaneEnding::Reported(agent_report.clone())),
                            ToolRequest::SubmitReview { review } => Some(LaneEnding::Reviewed(review.clone())),
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
                let (output, succeeded): (String, bool) = match self
                    .apply_allowed_patch(agent_assignment, run_context, patch_operation)
                    .await
                {
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
            ToolRequest::Report { .. } => Ok(("Reported.".to_string(), true)),
            ToolRequest::SubmitReview { .. } => Ok(("Review received.".to_string(), true)),
            ToolRequest::BriefImplementor { implementation_brief } => {
                match self.brief_processor.check(implementation_brief) {
                    Ok(()) => Ok((self.tool_processor.format_brief_accepted(implementation_brief), true)),
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

    // The refusal of a tool the agent may not call now: the run's allowed tools (its role's, or fewer, as when the
    // orchestrator tells the person how a build went).
    fn check_tool_allowed(
        &self,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        tool_name: &str,
    ) -> ApplicationResult<()> {
        let allowed_tool_names: Vec<String> = run_context
            .allowed_tool_names
            .clone()
            .unwrap_or_else(|| self.tool_processor.get_allowed_tool_names(agent_assignment.role));
        if allowed_tool_names.iter().any(|allowed: &String| allowed == tool_name) {
            return Ok(());
        }
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

    // A patch's file operation, if the agent may patch.
    async fn apply_allowed_patch(
        &self,
        agent_assignment: &AgentAssignment,
        run_context: &RunContext,
        patch_operation: &PatchOperation,
    ) -> ApplicationResult<String> {
        self.check_tool_allowed(agent_assignment, run_context, APPLY_PATCH)?;
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
                | AgentRunEngineException::ToolNotAllowed
                | AgentRunEngineException::InvalidBrief => true,
                AgentRunEngineException::MaxTurnsReached => false,
            },
            ExceptionKind::WorkspaceGateway(_) | ExceptionKind::CommandRunnerGateway(_) => true,
            ExceptionKind::ConfigurationProvider(_)
            | ExceptionKind::OpenAiResponsesGateway(_)
            | ExceptionKind::OttoBotManager(_)
            | ExceptionKind::ProjectBootstrapEngine(_)
            | ExceptionKind::LockStoreGateway(_)
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
