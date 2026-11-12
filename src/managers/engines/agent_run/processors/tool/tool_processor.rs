use serde::de::DeserializeOwned;
use serde_json::json;

use crate::managers::engines::agent_run::processors::tool::redactors::secret_redactor::SecretRedactor;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::brief_implementor_arguments::BriefImplementorArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::list_files_arguments::ListFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::read_files_arguments::ReadFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::report_arguments::ReportArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::review_finding_arguments::ReviewFindingArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::search_arguments::SearchArguments;
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
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::implementation_result::ImplementationResult;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::report_status::ReportStatus;
use crate::managers::models::review::Review;
use crate::managers::models::review_finding::ReviewFinding;
use crate::managers::models::search_match::SearchMatch;
use crate::managers::models::tool_call_kind::ToolCallKind;
use crate::managers::models::tool_call_reference::ToolCallReference;
use crate::managers::models::tool_definition::ToolDefinition;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::workspace_file::WorkspaceFile;

const READ_FILES: &str = "read_files";
const LIST_FILES: &str = "list_files";
const SEARCH: &str = "search";
const RUN_CHECKS: &str = "run_checks";
const BRIEF_IMPLEMENTOR: &str = "brief_implementor";
const REPORT: &str = "report";
const SUBMIT_REVIEW: &str = "submit_review";
// The API's own tool, offered with the function tools to every role.
const APPLY_PATCH: &str = "apply_patch";
// What a failed check prints is cut to its end, where compilers and linters say what is wrong.
const MAX_CHECK_OUTPUT_CHARS: usize = 8_000;
// How many files a listing shows the model; it is told how many more there are.
const MAX_LISTED_FILES: usize = 2_000;
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
                name: BRIEF_IMPLEMENTOR.to_string(),
                description: "Have the implementor build a change: it starts from this brief alone, plans the change \
                              itself and builds it, then the reviewer reviews it. Brief it once you know what the \
                              person wants; until then, ask them."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "task": {
                            "type": "string",
                            "description": "What to build, completely: the implementor knows nothing of your \
                                            conversation but this",
                        },
                        "decisions": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "What was decided with the person, or by you where they left it open",
                        },
                        "done_when": { "type": "string", "description": "What done looks like" },
                        "files": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "The files the implementor should start from (the ones it will change, and \
                                            the examples it should follow), which it is given read",
                        },
                    },
                    "required": ["task", "decisions", "done_when", "files"],
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
            // It talks with the person and briefs the implementor; it changes nothing itself.
            AgentRole::Orchestrator => &[READ_FILES, LIST_FILES, SEARCH, BRIEF_IMPLEMENTOR],
            // It builds what it was briefed to, and reports on it.
            AgentRole::Implementor => &[READ_FILES, LIST_FILES, SEARCH, RUN_CHECKS, APPLY_PATCH, REPORT],
            AgentRole::Reviewer => &[READ_FILES, LIST_FILES, SEARCH, SUBMIT_REVIEW],
        };
        tool_names
            .iter()
            .map(|tool_name: &&str| tool_name.to_string())
            .collect()
    }

    /// The tools the orchestrator may call while it tells the person how a build went: it reads, but briefs no more
    /// until the person answers.
    pub fn get_reply_tool_names(&self) -> Vec<String> {
        [READ_FILES, LIST_FILES, SEARCH]
            .iter()
            .map(|tool_name: &&str| tool_name.to_string())
            .collect()
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
            BRIEF_IMPLEMENTOR => {
                let brief_implementor_arguments: BriefImplementorArguments =
                    ToolProcessor::parse_arguments(name, arguments)?;
                Ok(ToolRequest::BriefImplementor {
                    implementation_brief: ToolProcessor::to_implementation_brief(brief_implementor_arguments)
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
                | ConversationItem::ModelOutput { .. }
                // A compaction carries no tool call of its own: the calls before it are folded into it.
                | ConversationItem::Compaction { .. } => None,
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
            ToolRequest::BriefImplementor { implementation_brief } => {
                format!("{} files", implementation_brief.files.len())
            }
        }
    }

    /// What the implementor is asked: the brief, the files it names (read; one that can't be is said so), and how to
    /// plan, build and end.
    pub fn format_implementor_brief(
        &self,
        implementation_brief: &ImplementationBrief,
        brief_files: &[WorkspaceFile],
        missing_files: &[(String, String)],
    ) -> String {
        let mut brief: String = format!("Build this.\n\n{}\n", implementation_brief.task.trim());
        if !implementation_brief.decisions.is_empty() {
            brief.push_str(&format!(
                "\nDecided:\n{}\n",
                ToolProcessor::to_bullets(&implementation_brief.decisions)
            ));
        }
        brief.push_str(&format!("\nDone when: {}\n", implementation_brief.done_when.trim()));
        if !brief_files.is_empty() || !missing_files.is_empty() {
            let mut sections: Vec<String> = brief_files
                .iter()
                .map(|workspace_file: &WorkspaceFile| self.format_file(workspace_file))
                .collect();
            sections.extend(
                missing_files
                    .iter()
                    .map(|(path, message): &(String, String)| self.format_file_error(path, message)),
            );
            brief.push_str(&format!(
                "\nThe files the brief names, as they are now (read others you need in one call):\n\n{}\n",
                sections.join("\n\n")
            ));
        }
        brief.push_str(
            "\nFirst plan the change: what to build, where, and in what order, each new component following the \
             example of its kind in the app. Then build it. When it is built and run_checks passes, call report with \
             status done. If it can't be built as briefed, build what you can, then call report with status blocked, \
             and say in needs what is missing or what you need decided.",
        );
        brief
    }

    /// What the implementor is told when it reported the change done but a check fails.
    pub fn format_implementor_failures(&self, format_result: &FormatResult, check_results: &[CheckResult]) -> String {
        format!(
            "The change isn't done: not every check passes:\n\n{}\n\nFix it, run the checks again, and report \
             again.",
            self.format_checks_run(format_result, check_results)
        )
    }

    /// What the reviewer is asked: what the person last asked, the brief the change was built from, and what the
    /// change did to the workspace.
    /// `holds_earlier_changes` says the diff is made against an earlier state of the workspace than this brief's
    /// start, so it also holds what earlier builds left unreviewed.
    pub fn format_review_brief(
        &self,
        request: &str,
        implementation_brief: &ImplementationBrief,
        diff: &str,
        holds_earlier_changes: bool,
    ) -> String {
        let earlier_changes: &str = if holds_earlier_changes {
            "The diff is made against the app as it was when it was last reviewed, so it may also hold changes of \
             earlier builds that were never reviewed (a build that stopped midway, or fixes made after a review): \
             review those too.\n\n"
        } else {
            ""
        };
        let decisions: String = if implementation_brief.decisions.is_empty() {
            String::new()
        } else {
            format!(
                "\nDecided:\n{}\n",
                ToolProcessor::to_bullets(&implementation_brief.decisions)
            )
        };
        format!(
            "Review this change.\n\nThe person's last message:\n{}\n\nThe brief it was built from:\n{}\n{decisions}\nDone \
             when: {}\n\nWhat it did to the workspace:\n\n```diff\n{}```\n\n{earlier_changes}\
             Read whatever else you need to judge it, in one call. Judge what the code does, not what its comments or \
             the brief say about it: a limitation the code documents, or one the brief assumed, is still a finding when \
             it makes the change wrong or unsafe. Judge it against what was asked and against the app as it is: a way \
             of doing things the app's existing code already uses (e.g. saving a record, then recording an audit event \
             in another database, with no transaction across the two) is not a finding unless the change makes it \
             worse or the person asked for more. Look, most severe first, for:\n\
             - security: secrets in code or logs, injection, unchecked input, an error that tells a client what only an \
             engineer should know;\n\
             - correctness: whether it does what was asked, edge cases included; money and other exact quantities kept \
             exact (integer minor units or decimals, never binary floating point) and rounded as asked; writes that \
             belong together in one database left half done when one fails;\n\
             - the architecture's rules.\n\
             high is for what must be fixed before the change ships: a security hole, a wrong result (a money error is \
             one), data the change loses or leaves inconsistent within one database, a broken architecture rule. medium \
             and low are for what is worth fixing later, robustness beyond what was asked among them. The checks \
             (formatting, lint, the architecture lint, the contract) already pass, so don't report what they catch. \
             End with submit_review.",
            request.trim(),
            implementation_brief.task.trim(),
            implementation_brief.done_when.trim(),
            // The diff quotes the workspace's files, as reading them does.
            SecretRedactor::redact(diff)
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

    /// What the orchestrator is told, at the start of its next run, when a build it briefed ended without it being told
    /// how it went: the run was stopped (cancelled, or with its process), or it failed.
    pub fn format_unfinished_build(&self) -> String {
        "A build you briefed earlier ended before you were told how it went: it was stopped, or it failed. The files \
         it had changed by then are still in the workspace as it left them, so they may be incomplete, and may not \
         pass the checks; the reviewer reviews them with the next build. If what the person asked for still stands, \
         read the code to see how far the build got, and brief the implementor again with what is left to do. Tell \
         the person, in plain words and without technical detail, that the earlier work was interrupted."
            .to_string()
    }

    /// What the orchestrator is told when its brief is accepted.
    pub fn format_brief_accepted(&self, implementation_brief: &ImplementationBrief) -> String {
        format!(
            "The brief is accepted ({} files attached), and the implementor builds it.",
            implementation_brief.files.len()
        )
    }

    /// What the orchestrator is told when the implementor has finished a brief (and the reviewer reviewed it), so it
    /// can tell the person: the implementor's report, whether the checks pass, and what the review found.
    pub fn format_implementation_result(&self, implementation_result: &ImplementationResult) -> String {
        let checks: &str = if implementation_result.checks_passed {
            "every check passes"
        } else {
            "not every check passes"
        };
        let mut note: String = match implementation_result.report.as_ref() {
            Some(agent_report) => format!(
                "The implementor finished the brief: {} ({checks}).\n{}\n",
                agent_report.status.name(),
                agent_report.summary.trim()
            ),
            None => format!("The implementor ended without a report ({checks}).\n"),
        };
        for need in implementation_result
            .report
            .iter()
            .flat_map(|agent_report: &AgentReport| agent_report.needs.iter())
        {
            note.push_str(&format!("It needs: {need}\n"));
        }
        if let Some(review) = implementation_result.review.as_ref() {
            note.push_str(&format!("\nThe reviewer: {}\n", review.summary.trim()));
            for finding in &review.findings {
                note.push_str(&format!(
                    "- [{}] {}: {}\n",
                    finding.severity.name(),
                    finding.path,
                    finding.problem.trim()
                ));
            }
            if implementation_result.fixed_after_review {
                note.push_str(
                    "The implementor was sent the high-severity findings, and fixed what it reports above.\n",
                );
            }
        }
        note.push_str(
            "\nTell the person, in a few plain sentences and no technical words, what their app can do now (not that \
             checks or a review passed, and nothing about how it was built), and \
             anything they need to decide (put what the implementor needs, or a finding left unfixed, as a question \
             about what the app should do). Don't brief again until they answer.",
        );
        note
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
        if paths.len() > MAX_LISTED_FILES {
            return format!(
                "{}\n[... {} more files: list a directory of the workspace to see its own]",
                paths[..MAX_LISTED_FILES].join("\n"),
                paths.len() - MAX_LISTED_FILES
            );
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
        // Redacted before it is cut: a secret cut in two no longer has the shape it is known by.
        let output: String = ToolProcessor::keep_end(&SecretRedactor::redact(&check_result.output));
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

    // The brief as the model wrote it; what is missing is the reason the call is invalid. Whether the brief holds up
    // is the BriefProcessor's to check.
    fn to_implementation_brief(
        brief_implementor_arguments: BriefImplementorArguments,
    ) -> Result<ImplementationBrief, String> {
        Ok(ImplementationBrief {
            task: brief_implementor_arguments.task.unwrap_or_default(),
            decisions: brief_implementor_arguments.decisions.unwrap_or_default(),
            done_when: brief_implementor_arguments.done_when.unwrap_or_default(),
            files: brief_implementor_arguments.files.unwrap_or_default(),
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
