use serde::de::DeserializeOwned;
use serde_json::json;

use crate::managers::engines::agent_run::processors::tool::redactors::secret_redactor::SecretRedactor;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::list_files_arguments::ListFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::read_files_arguments::ReadFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::search_arguments::SearchArguments;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::check_result::CheckResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::lint_violation::LintViolation;
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
        ]
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
        }
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
