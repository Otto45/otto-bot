use serde::de::DeserializeOwned;
use serde_json::json;

use crate::managers::engines::agent_run::processors::tool::redactors::secret_redactor::SecretRedactor;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::list_files_arguments::ListFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::read_files_arguments::ReadFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::report_arguments::ReportArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::review_finding_arguments::ReviewFindingArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::search_arguments::SearchArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::submit_plan_arguments::SubmitPlanArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::submit_review_arguments::SubmitReviewArguments;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::agent_role::AgentRole;
use crate::managers::models::check_result::CheckResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::finding_severity::FindingSeverity;
use crate::managers::models::format_result::FormatResult;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::report_status::ReportStatus;
use crate::managers::models::review::Review;
use crate::managers::models::review_finding::ReviewFinding;
use crate::managers::models::search_match::SearchMatch;
use crate::managers::models::tool_call_kind::ToolCallKind;
use crate::managers::models::tool_call_reference::ToolCallReference;
use crate::managers::models::tool_definition::ToolDefinition;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::work_plan::WorkPlan;
use crate::managers::models::workspace_file::WorkspaceFile;

const READ_FILES: &str = "read_files";
const LIST_FILES: &str = "list_files";
const SEARCH: &str = "search";
const RUN_CHECKS: &str = "run_checks";
const SUBMIT_PLAN: &str = "submit_plan";
const REPORT: &str = "report";
const SUBMIT_REVIEW: &str = "submit_review";
// The API's own tool, offered with the function tools to every role.
const APPLY_PATCH: &str = "apply_patch";
// What a failed check prints is cut to its end, where compilers and linters say what is wrong.
const MAX_CHECK_OUTPUT_CHARS: usize = 8_000;
const UNFINISHED_TOOL_CALL_OUTPUT: &str =
    "The run was cancelled while this call was being carried out; it may or may not have taken effect.";

// The agent's function tools: what they are, what the model asked of one, and how what they did is told back to it.
// Every role gets the same tools in the same order, so the prompt's prefix (and its cache) is shared.
pub struct ToolProcessor;

impl ToolProcessor {
    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: READ_FILES.to_string(),
                description: "Read files of the workspace, whole. Read every file you need in one call.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "paths": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Paths relative to the workspace",
                        },
                    },
                    "required": ["paths"],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: LIST_FILES.to_string(),
                description: "List the files under a directory of the workspace, recursively; what .gitignore \
                              ignores is left out."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "A path relative to the workspace; \".\" for the whole workspace",
                        },
                    },
                    "required": ["directory"],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: SEARCH.to_string(),
                description: "Search the workspace's files for lines matching a regular expression.".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "A regular expression (Rust regex syntax)" },
                        "path_glob": {
                            "type": ["string", "null"],
                            "description": "Only search files matching this glob (e.g. \"src/**/*.rs\"); null for all",
                        },
                    },
                    "required": ["pattern", "path_glob"],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: RUN_CHECKS.to_string(),
                description: "Run the stack's checks (formatting, lint, contract) on the workspace and report which \
                              pass. Run them after changing code, and fix what fails."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: SUBMIT_PLAN.to_string(),
                description: "Plan a change too big to make well yourself, instead of making it: the harness checks \
                              the plan, then the implementor builds it. Call it before you change any file."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "What the change does, in a few sentences" },
                        "assumptions": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "What you decided where the request left it open",
                        },
                        "steps": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "How the implementor builds the change, in order: each step what to build, \
                                            where, and against what (signatures, fields, the files it touches)",
                        },
                    },
                    "required": ["summary", "assumptions", "steps"],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: REPORT.to_string(),
                description: "End your assignment with a terse report: done (your part is built and run_checks \
                              passes), or blocked (it can't be finished without a change you may not make)."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "enum": ["done", "blocked"] },
                        "summary": {
                            "type": "string",
                            "description": "What you built and anything you decided on your own, in a few lines",
                        },
                        "needs": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Each change you need but may not make: which file, what, and why",
                        },
                    },
                    "required": ["status", "summary", "needs"],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: SUBMIT_REVIEW.to_string(),
                description: "End your review with what you found wrong with the change, most severe first; no \
                              findings when it is sound."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "Your verdict, in a line or two" },
                        "findings": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "severity": {
                                        "type": "string",
                                        "enum": ["high", "medium", "low"],
                                        "description": "high: must be fixed before it ships (a security hole, a broken \
                                                        architecture rule, a bug); medium or low: worth fixing later",
                                    },
                                    "path": { "type": "string", "description": "The file it is in" },
                                    "problem": { "type": "string" },
                                    "fix": { "type": "string", "description": "What would put it right" },
                                },
                                "required": ["severity", "path", "problem", "fix"],
                                "additionalProperties": false,
                            },
                        },
                    },
                    "required": ["summary", "findings"],
                    "additionalProperties": false,
                }),
            },
        ]
    }

    /// The tools the role may call, by name; none when it may call every tool. Every role is sent every tool, so the
    /// cached prefix is shared, and restricted to these.
    pub fn get_allowed_tool_names(&self, agent_role: AgentRole) -> Vec<String> {
        let tool_names: &[&str] = match agent_role {
            // Everything but report and submit_review, which end the implementor's and the reviewer's assignments.
            AgentRole::Orchestrator => &[READ_FILES, LIST_FILES, SEARCH, RUN_CHECKS, SUBMIT_PLAN, APPLY_PATCH],
            // It builds the plan it was given, and reports on it.
            AgentRole::Implementor => &[READ_FILES, LIST_FILES, SEARCH, RUN_CHECKS, APPLY_PATCH, REPORT],
            AgentRole::Reviewer => &[READ_FILES, LIST_FILES, SEARCH, SUBMIT_REVIEW],
        };
        tool_names
            .iter()
            .map(|tool_name: &&str| tool_name.to_string())
            .collect()
    }

    /// Whether the role may call the tool: the model is told which it may, and the harness holds it to that too.
    pub fn is_tool_allowed(&self, agent_role: AgentRole, tool_name: &str) -> bool {
        self.get_allowed_tool_names(agent_role)
            .iter()
            .any(|allowed: &String| allowed == tool_name)
    }

    /// What a function tool call asks for. An unknown tool or arguments that don't fit are an InvalidToolCall, whose
    /// message is told to the model.
    pub fn to_tool_request(&self, name: &str, arguments: &str) -> ApplicationResult<ToolRequest> {
        match name {
            READ_FILES => {
                let read_files_arguments: ReadFilesArguments = ToolProcessor::parse_arguments(name, arguments)?;
                match read_files_arguments.paths {
                    Some(paths) if !paths.is_empty() => Ok(ToolRequest::ReadFiles { paths }),
                    _ => Err(ToolProcessor::to_invalid_tool_call(name, "paths is missing or empty")),
                }
            }
            LIST_FILES => {
                let list_files_arguments: ListFilesArguments = ToolProcessor::parse_arguments(name, arguments)?;
                Ok(ToolRequest::ListFiles {
                    directory: list_files_arguments.directory.unwrap_or_else(|| ".".to_string()),
                })
            }
            SEARCH => {
                let search_arguments: SearchArguments = ToolProcessor::parse_arguments(name, arguments)?;
                match search_arguments.pattern {
                    Some(pattern) if !pattern.is_empty() => Ok(ToolRequest::Search {
                        pattern,
                        path_glob: search_arguments
                            .path_glob
                            .filter(|path_glob: &String| !path_glob.trim().is_empty()),
                    }),
                    _ => Err(ToolProcessor::to_invalid_tool_call(name, "pattern is missing or empty")),
                }
            }
            RUN_CHECKS => Ok(ToolRequest::RunChecks),
            REPORT => {
                let report_arguments: ReportArguments = ToolProcessor::parse_arguments(name, arguments)?;
                match (
                    report_arguments.status.as_deref().and_then(ReportStatus::from_name),
                    report_arguments.summary,
                ) {
                    (Some(status), Some(summary)) => Ok(ToolRequest::Report {
                        agent_report: AgentReport {
                            status,
                            summary,
                            needs: report_arguments.needs.unwrap_or_default(),
                        },
                    }),
                    _ => Err(ToolProcessor::to_invalid_tool_call(
                        name,
                        "status (done or blocked) and summary are required",
                    )),
                }
            }
            SUBMIT_REVIEW => {
                let submit_review_arguments: SubmitReviewArguments = ToolProcessor::parse_arguments(name, arguments)?;
                Ok(ToolRequest::SubmitReview {
                    review: ToolProcessor::to_review(submit_review_arguments)
                        .map_err(|reason: String| ToolProcessor::to_invalid_tool_call(name, &reason))?,
                })
            }
            SUBMIT_PLAN => {
                let submit_plan_arguments: SubmitPlanArguments = ToolProcessor::parse_arguments(name, arguments)?;
                Ok(ToolRequest::SubmitPlan {
                    work_plan: ToolProcessor::to_work_plan(submit_plan_arguments)
                        .map_err(|reason: String| ToolProcessor::to_invalid_tool_call(name, &reason))?,
                })
            }
            _ => Err(ToolProcessor::to_invalid_tool_call(
                name,
                "there is no tool by that name",
            )),
        }
    }

    /// Outputs for the tool calls of a conversation that were never answered (their run was cancelled midway), in
    /// their order: the API refuses a conversation with a tool call that has no output.
    pub fn answer_unfinished_tool_calls(&self, conversation_items: &[ConversationItem]) -> Vec<ConversationItem> {
        let answered_call_ids: std::collections::HashSet<&str> = conversation_items
            .iter()
            .filter_map(|conversation_item: &ConversationItem| match conversation_item {
                ConversationItem::FunctionCallOutput { call_id, .. }
                | ConversationItem::ApplyPatchCallOutput { call_id, .. } => Some(call_id.as_str()),
                ConversationItem::UserMessage { .. }
                | ConversationItem::HarnessNote { .. }
                | ConversationItem::ConfigurationUpdate { .. }
                | ConversationItem::ModelOutput { .. } => None,
            })
            .collect();

        conversation_items
            .iter()
            .filter_map(|conversation_item: &ConversationItem| match conversation_item {
                ConversationItem::ModelOutput {
                    tool_call: Some(tool_call),
                    ..
                } if !answered_call_ids.contains(tool_call.call_id.as_str()) => Some(tool_call),
                _ => None,
            })
            .map(|tool_call: &ToolCallReference| match tool_call.kind {
                ToolCallKind::Function => ConversationItem::FunctionCallOutput {
                    call_id: tool_call.call_id.clone(),
                    output: UNFINISHED_TOOL_CALL_OUTPUT.to_string(),
                },
                ToolCallKind::ApplyPatch => ConversationItem::ApplyPatchCallOutput {
                    call_id: tool_call.call_id.clone(),
                    succeeded: false,
                    output: UNFINISHED_TOOL_CALL_OUTPUT.to_string(),
                },
            })
            .collect()
    }

    /// What the tool is used on, as a person reading along would want it named.
    pub fn describe_target(&self, tool_request: &ToolRequest) -> String {
        match tool_request {
            ToolRequest::ReadFiles { paths } => paths.join(", "),
            ToolRequest::ListFiles { directory } => directory.clone(),
            ToolRequest::Search { pattern, path_glob } => match path_glob {
                Some(path_glob) => format!("/{pattern}/ in {path_glob}"),
                None => format!("/{pattern}/"),
            },
            ToolRequest::RunChecks => String::new(),
            ToolRequest::Report { agent_report } => agent_report.status.name().to_string(),
            ToolRequest::SubmitReview { review } => format!("{} findings", review.findings.len()),
            ToolRequest::SubmitPlan { work_plan } => format!("{} steps", work_plan.steps.len()),
        }
    }

    /// What the implementor is asked: the plan, and how to end its build.
    pub fn format_implementor_brief(&self, work_plan: &WorkPlan) -> String {
        let mut brief: String = format!("Build this plan.\n\n{}\n", work_plan.summary.trim());
        if !work_plan.assumptions.is_empty() {
            brief.push_str(&format!(
                "\nDecided where the request left it open:\n{}\n",
                ToolProcessor::to_bullets(&work_plan.assumptions)
            ));
        }
        let steps: Vec<String> = work_plan
            .steps
            .iter()
            .enumerate()
            .map(|(index, step): (usize, &String)| format!("{}. {}", index + 1, step.trim()))
            .collect();
        brief.push_str(&format!("\nSteps:\n{}\n", steps.join("\n")));
        brief.push_str(
            "\nWhen the change is built and run_checks passes, call report with status done. If the plan can't be \
             built as it is, build what you can, then call report with status blocked, and say in needs what is \
             missing or wrong.",
        );
        brief
    }

    /// What the implementor is told when it reported the plan done but a check fails.
    pub fn format_implementor_failures(&self, format_result: &FormatResult, check_results: &[CheckResult]) -> String {
        format!(
            "The change isn't done: not every check passes:\n\n{}\n\nFix it, run the checks again, and report \
             again.",
            self.format_checks_run(format_result, check_results)
        )
    }

    /// What the reviewer is asked: the change's plan, and what it did to the workspace.
    pub fn format_review_brief(&self, work_plan: &WorkPlan, diff: &str) -> String {
        format!(
            "Review this change. Its plan: {}\n\nWhat it did to the workspace:\n\n```diff\n{}```\n\nRead whatever else you \
             need to judge it, in one call. Look, most severe first, for: security (secrets in code or logs, injection, \
             unchecked input, an error that leaks internals to a client), the architecture's rules, and whether it does \
             what the plan says. The checks (formatting, lint, the architecture lint, the contract) already pass, so \
             don't report what they catch. End with submit_review.",
            work_plan.summary.trim(),
            diff
        )
    }

    /// What the implementor is told to fix: the review's findings that must be fixed before the change ships.
    pub fn format_review_fixes(&self, review: &Review) -> String {
        let high_findings: Vec<String> = review
            .findings
            .iter()
            .filter(|finding: &&ReviewFinding| finding.severity == FindingSeverity::High)
            .map(|finding: &ReviewFinding| format!("{}: {} Fix: {}", finding.path, finding.problem, finding.fix))
            .collect();
        format!(
            "The reviewer found what must be fixed before the change ships:\n{}\n\nFix these, run the checks, and report \
             again.",
            ToolProcessor::to_bullets(&high_findings)
        )
    }

    /// What the orchestrator is told when its plan is accepted.
    pub fn format_plan_accepted(&self, work_plan: &WorkPlan) -> String {
        format!(
            "The plan is accepted ({} steps), and the implementor builds it.",
            work_plan.steps.len()
        )
    }

    pub fn format_file(&self, workspace_file: &WorkspaceFile) -> String {
        format!(
            "=== {}\n{}",
            workspace_file.path,
            SecretRedactor::redact(&workspace_file.content)
        )
    }

    pub fn format_file_error(&self, path: &str, message: &str) -> String {
        format!("=== {path}\n[error: {message}]")
    }

    pub fn format_file_list(&self, paths: &[String]) -> String {
        if paths.is_empty() {
            return "No files.".to_string();
        }
        paths.join("\n")
    }

    pub fn format_search_matches(&self, search_matches: &[SearchMatch]) -> String {
        if search_matches.is_empty() {
            return "No matches.".to_string();
        }
        search_matches
            .iter()
            .map(|search_match: &SearchMatch| {
                format!(
                    "{}:{}: {}",
                    search_match.path,
                    search_match.line_number,
                    SecretRedactor::redact(&search_match.line)
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// A run of the checks: what each check found, then which files the formatter (which ran first) changed, which the
    /// agent reads again before it patches them.
    pub fn format_checks_run(&self, format_result: &FormatResult, check_results: &[CheckResult]) -> String {
        let mut output: String = self.format_check_results(check_results);
        if !format_result.reformatted_paths.is_empty() {
            output.push_str(&format!(
                "\n\nThe formatters ran first and rewrote these files, so read them again before you patch them:\n{}",
                ToolProcessor::to_bullets(&format_result.reformatted_paths)
            ));
        }
        output
    }

    pub fn format_check_results(&self, check_results: &[CheckResult]) -> String {
        if check_results.is_empty() {
            return "The stack has no checks to run.".to_string();
        }
        check_results
            .iter()
            .map(ToolProcessor::format_check_result)
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    /// The architecture lint's violations, one per line, as a failed check prints them.
    pub fn format_lint_violations(&self, lint_violations: &[LintViolation]) -> String {
        lint_violations
            .iter()
            .map(|lint_violation: &LintViolation| {
                let excerpt: String = if lint_violation.excerpt.is_empty() {
                    String::new()
                } else {
                    format!(": `{}`", lint_violation.excerpt)
                };
                format!(
                    "{}:{}: {} ({}){excerpt}",
                    lint_violation.path, lint_violation.line_number, lint_violation.message, lint_violation.rule_name
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn format_error(&self, message: &str) -> String {
        format!("error: {message}")
    }

    fn format_check_result(check_result: &CheckResult) -> String {
        if check_result.passed {
            return format!("{}: passed", check_result.name);
        }
        // A check the harness runs itself (the architecture lint) has no exit code.
        let how: String = if check_result.timed_out {
            " (timed out)".to_string()
        } else {
            match check_result.exit_code {
                Some(exit_code) => format!(" (exit code {exit_code})"),
                None => String::new(),
            }
        };
        let output: String = SecretRedactor::redact(&ToolProcessor::keep_end(&check_result.output));
        format!("{}: failed{how}\n{}", check_result.name, output.trim_end())
    }

    fn to_bullets(paths: &[String]) -> String {
        if paths.is_empty() {
            return "- nothing".to_string();
        }
        paths
            .iter()
            .map(|path: &String| format!("- {path}"))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn keep_end(text: &str) -> String {
        let char_count: usize = text.chars().count();
        if char_count <= MAX_CHECK_OUTPUT_CHARS {
            return text.to_string();
        }
        let kept: String = text.chars().skip(char_count - MAX_CHECK_OUTPUT_CHARS).collect();
        format!(
            "[... {} characters before this]\n{kept}",
            char_count - MAX_CHECK_OUTPUT_CHARS
        )
    }

    // The review as the model wrote it, most severe findings first; what is missing is the reason the call is invalid.
    fn to_review(submit_review_arguments: SubmitReviewArguments) -> Result<Review, String> {
        let mut findings: Vec<ReviewFinding> = submit_review_arguments
            .findings
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(index, finding): (usize, ReviewFindingArguments)| {
                match (
                    finding.severity.as_deref().and_then(FindingSeverity::from_name),
                    finding.problem,
                ) {
                    (Some(severity), Some(problem)) => Ok(ReviewFinding {
                        severity,
                        path: finding.path.unwrap_or_default(),
                        problem,
                        fix: finding.fix.unwrap_or_default(),
                    }),
                    _ => Err(format!(
                        "findings[{index}] needs a severity (high, medium or low) and a problem"
                    )),
                }
            })
            .collect::<Result<Vec<ReviewFinding>, String>>()?;
        findings.sort_by_key(|finding: &ReviewFinding| match finding.severity {
            FindingSeverity::High => 0,
            FindingSeverity::Medium => 1,
            FindingSeverity::Low => 2,
        });
        Ok(Review {
            summary: submit_review_arguments.summary.unwrap_or_default(),
            findings,
        })
    }

    // The plan as the model wrote it; what is missing is the reason the call is invalid. Whether the plan holds up is
    // the WorkPlanProcessor's to check.
    fn to_work_plan(submit_plan_arguments: SubmitPlanArguments) -> Result<WorkPlan, String> {
        Ok(WorkPlan {
            summary: submit_plan_arguments.summary.unwrap_or_default(),
            assumptions: submit_plan_arguments.assumptions.unwrap_or_default(),
            steps: submit_plan_arguments.steps.unwrap_or_default(),
        })
    }

    fn parse_arguments<T: DeserializeOwned>(name: &str, arguments: &str) -> ApplicationResult<T> {
        serde_json::from_str::<T>(arguments).map_err(|exception: serde_json::Error| {
            ToolProcessor::to_invalid_tool_call(name, &format!("its arguments are not valid: {exception}"))
        })
    }

    fn to_invalid_tool_call(name: &str, reason: &str) -> ApplicationBaseException {
        ApplicationBaseException::new(
            AgentRunEngineException::InvalidToolCall,
            format!("The call to {name} is invalid: {reason}"),
            ContextualData::new().with("tool_name", name),
        )
    }
}
