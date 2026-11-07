use serde::de::DeserializeOwned;
use serde_json::json;

use crate::managers::engines::agent_run::processors::tool::redactors::secret_redactor::SecretRedactor;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::list_files_arguments::ListFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::planned_component_arguments::PlannedComponentArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::read_files_arguments::ReadFilesArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::scaffold_arguments::ScaffoldArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::search_arguments::SearchArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::submit_plan_arguments::SubmitPlanArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::work_package_arguments::WorkPackageArguments;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::agent_role::AgentRole;
use crate::managers::models::check_result::CheckResult;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::format_result::FormatResult;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::search_match::SearchMatch;
use crate::managers::models::tool_call_kind::ToolCallKind;
use crate::managers::models::tool_call_reference::ToolCallReference;
use crate::managers::models::tool_definition::ToolDefinition;
use crate::managers::models::tool_request::ToolRequest;
use crate::managers::models::work_package::WorkPackage;
use crate::managers::models::work_plan::WorkPlan;
use crate::managers::models::workspace_file::WorkspaceFile;

const READ_FILES: &str = "read_files";
const LIST_FILES: &str = "list_files";
const SEARCH: &str = "search";
const RUN_CHECKS: &str = "run_checks";
const SCAFFOLD: &str = "scaffold";
const SUBMIT_PLAN: &str = "submit_plan";
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
                name: SCAFFOLD.to_string(),
                description: "Create a new component and wire it in (its files, its module declarations, the service \
                              locator), without writing it by hand; then read what it created and write the logic. \
                              The stack's kinds are listed in the instructions."
                    .to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "description": "One of the stack's scaffold kinds" },
                        "name": { "type": "string", "description": "The component's name, snake_case" },
                        "within": {
                            "type": ["string", "null"],
                            "description": "For a kind that lives inside another component: that component's name; \
                                            otherwise null",
                        },
                    },
                    "required": ["kind", "name", "within"],
                    "additionalProperties": false,
                }),
            },
            ToolDefinition {
                name: SUBMIT_PLAN.to_string(),
                description: "Plan a change too big to build well alone, instead of building it: the harness checks \
                              the plan, scaffolds its components, has you write the contracts, then starts one worker \
                              per package, all at the same time. Call it before you change any file."
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
                        "components": {
                            "type": "array",
                            "description": "Every new component, to be scaffolded before the workers start",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "kind": { "type": "string", "description": "One of the stack's scaffold kinds" },
                                    "name": { "type": "string", "description": "snake_case" },
                                    "within": {
                                        "type": ["string", "null"],
                                        "description": "For a kind that lives inside another component: its name",
                                    },
                                },
                                "required": ["kind", "name", "within"],
                                "additionalProperties": false,
                            },
                        },
                        "packages": {
                            "type": "array",
                            "description": "At least two packages, each built by its own worker at the same time as \
                                            the others; a package owns the files only it changes",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "snake_case" },
                                    "brief": {
                                        "type": "string",
                                        "description": "What the worker builds, and against which contracts: what \
                                                        it needs to know beyond its files and the project map",
                                    },
                                    "owned_paths": {
                                        "type": "array",
                                        "items": { "type": "string" },
                                        "description": "The files only this package changes, new or existing",
                                    },
                                },
                                "required": ["name", "brief", "owned_paths"],
                                "additionalProperties": false,
                            },
                        },
                    },
                    "required": ["summary", "assumptions", "components", "packages"],
                    "additionalProperties": false,
                }),
            },
        ]
    }

    /// The tools the role may call, by name; none when it may call every tool. Every role is sent every tool, so the
    /// cached prefix is shared, and restricted to these.
    pub fn get_allowed_tool_names(&self, agent_role: AgentRole, may_plan: bool) -> Option<Vec<String>> {
        let tool_names: &[&str] = match agent_role {
            AgentRole::Orchestrator if may_plan => return None,
            // Carrying its plan out: the plan's components are scaffolded, and it writes the contracts and the
            // coordinations.
            AgentRole::Orchestrator => &[READ_FILES, LIST_FILES, SEARCH, RUN_CHECKS, APPLY_PATCH],
            // A worker's components were scaffolded with the skeleton; it writes their logic.
            AgentRole::Worker => &[READ_FILES, LIST_FILES, SEARCH, RUN_CHECKS, APPLY_PATCH],
            AgentRole::Reviewer => &[READ_FILES, LIST_FILES, SEARCH],
        };
        Some(
            tool_names
                .iter()
                .map(|tool_name: &&str| tool_name.to_string())
                .collect(),
        )
    }

    /// Whether the role may call the tool: the model is told which it may, and the harness holds it to that too.
    pub fn is_tool_allowed(&self, agent_role: AgentRole, may_plan: bool, tool_name: &str) -> bool {
        self.get_allowed_tool_names(agent_role, may_plan)
            .is_none_or(|tool_names: Vec<String>| tool_names.iter().any(|allowed: &String| allowed == tool_name))
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
            SCAFFOLD => {
                let scaffold_arguments: ScaffoldArguments = ToolProcessor::parse_arguments(name, arguments)?;
                match (scaffold_arguments.kind, scaffold_arguments.name) {
                    (Some(kind), Some(component_name)) => Ok(ToolRequest::Scaffold {
                        kind,
                        name: component_name,
                        within: scaffold_arguments
                            .within
                            .filter(|within: &String| !within.trim().is_empty()),
                    }),
                    _ => Err(ToolProcessor::to_invalid_tool_call(name, "kind and name are required")),
                }
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
            ToolRequest::Scaffold { kind, name, within } => match within {
                Some(within) => format!("{kind} {name} in {within}"),
                None => format!("{kind} {name}"),
            },
            ToolRequest::SubmitPlan { work_plan } => format!(
                "{} packages: {}",
                work_plan.packages.len(),
                work_plan
                    .packages
                    .iter()
                    .map(|work_package: &WorkPackage| work_package.name.as_str())
                    .collect::<Vec<&str>>()
                    .join(", ")
            ),
        }
    }

    /// What the orchestrator is told once the harness has scaffolded its plan's components: what they are, as they are,
    /// and how to write the skeleton. `scaffold_outputs` are what each scaffold reported (its files' contents).
    pub fn format_skeleton_brief(&self, scaffold_outputs: &[String], placeholder: &str) -> String {
        let mut brief: String = String::from("The harness scaffolded the plan's components");
        if scaffold_outputs.is_empty() {
            brief.push_str(": the plan has none.\n");
        } else {
            brief.push_str(", as they are now:\n\n");
            brief.push_str(&scaffold_outputs.join("\n\n"));
            brief.push('\n');
        }
        brief.push_str(&format!(
            "\nNow write the skeleton: the contracts every package builds against, and nothing more.\n\
             - the models (their fields), the exception variants and any settings the plan needs;\n\
             - in each file a package owns, the public types and signatures it is to build, each function with a \
             placeholder body: {placeholder};\n\
             - the wiring the scaffolds didn't write: what is new in the service locator, module declarations, the \
             facade's and the managers' signatures.\n\
             Don't write the logic of a package's files, or the managers' coordinations: the workers build the \
             packages, then you write the coordinations. The workers start when every check passes, so run \
             run_checks and fix what fails; then say in one line that the skeleton is ready."
        ));
        brief
    }

    /// What the orchestrator is told when the skeleton fails a check after it finished.
    pub fn format_skeleton_failures(&self, format_result: &FormatResult, check_results: &[CheckResult]) -> String {
        format!(
            "The skeleton doesn't pass every check yet, and the workers start only once it does:\n\n{}\n\nFix it, \
             and run the checks again.",
            self.format_checks_run(format_result, check_results)
        )
    }

    /// What the orchestrator is told when its plan is accepted.
    pub fn format_plan_accepted(&self, work_plan: &WorkPlan) -> String {
        format!(
            "The plan is accepted: {} components to scaffold, {} packages. The harness scaffolds the components next; \
             then you write the contracts.",
            work_plan.components.len(),
            work_plan.packages.len()
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
                "\n\nThe formatter ran first and reformatted these files, so read them again before you patch them:\n{}",
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

    /// What a scaffold did: the files it created, whole (as they are now), so the model can patch them without reading
    /// them; the files it wired the component into; and how formatting went.
    pub fn format_scaffold(
        &self,
        created_files: &[WorkspaceFile],
        updated_paths: &[String],
        format_result: &FormatResult,
    ) -> String {
        let created: String = created_files
            .iter()
            .map(|created_file: &WorkspaceFile| self.format_file(created_file))
            .collect::<Vec<String>>()
            .join("\n\n");
        // The files it created and wired into are named below; any other the formatter changed is too.
        let other_reformatted_paths: Vec<String> = format_result
            .reformatted_paths
            .iter()
            .filter(|path: &&String| {
                !updated_paths.contains(path)
                    && !created_files
                        .iter()
                        .any(|created_file: &WorkspaceFile| &created_file.path == *path)
            })
            .cloned()
            .collect();
        let mut format_note: String = if format_result.succeeded {
            "Formatted.".to_string()
        } else {
            "The formatter failed on a file that doesn't parse; run_checks says which.".to_string()
        };
        if !other_reformatted_paths.is_empty() {
            format_note.push_str(&format!(
                " The formatter also reformatted these, so read them again before you patch them:\n{}",
                ToolProcessor::to_bullets(&other_reformatted_paths)
            ));
        }
        format!(
            "{format_note}\nWired into (module declarations and the service locator; no need to read them for \
             that):\n{}\n\nCreated, as they are now: write the component's logic in them.\n\n{created}",
            ToolProcessor::to_bullets(updated_paths)
        )
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

    // The plan as the model wrote it; what is missing is the reason the call is invalid. Whether the plan holds up is
    // the WorkPlanProcessor's to check.
    fn to_work_plan(submit_plan_arguments: SubmitPlanArguments) -> Result<WorkPlan, String> {
        let components: Vec<PlannedComponent> = submit_plan_arguments
            .components
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(
                |(index, component): (usize, PlannedComponentArguments)| match (component.kind, component.name) {
                    (Some(kind), Some(component_name)) => Ok(PlannedComponent {
                        kind,
                        name: component_name,
                        within: component.within.filter(|within: &String| !within.trim().is_empty()),
                    }),
                    _ => Err(format!("components[{index}] needs a kind and a name")),
                },
            )
            .collect::<Result<Vec<PlannedComponent>, String>>()?;
        let packages: Vec<WorkPackage> = submit_plan_arguments
            .packages
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(index, package): (usize, WorkPackageArguments)| {
                match (package.name, package.brief, package.owned_paths) {
                    (Some(package_name), Some(brief), Some(owned_paths)) => Ok(WorkPackage {
                        name: package_name,
                        brief,
                        owned_paths,
                    }),
                    _ => Err(format!("packages[{index}] needs a name, a brief and owned_paths")),
                }
            })
            .collect::<Result<Vec<WorkPackage>, String>>()?;
        Ok(WorkPlan {
            summary: submit_plan_arguments.summary.unwrap_or_default(),
            assumptions: submit_plan_arguments.assumptions.unwrap_or_default(),
            components,
            packages,
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
