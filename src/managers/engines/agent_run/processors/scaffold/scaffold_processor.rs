use std::sync::LazyLock;

use regex::Regex;

use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::scaffold_file::ScaffoldFile;
use crate::managers::models::scaffold_insertion::ScaffoldInsertion;
use crate::managers::models::scaffold_kind::ScaffoldKind;
use crate::managers::models::scaffold_plan::ScaffoldPlan;
use crate::managers::models::stack_pack_scaffold::StackPackScaffold;

// A component's name: snake_case, as a module's is.
static COMPONENT_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9]*(_[a-z0-9]+)*$").expect("a valid regular expression"));

// Renders a stack pack's scaffold kind for one component, and places its insertions. Reading and writing the files is
// the workspace's: a plan is checked whole before anything is written.
pub struct ScaffoldProcessor;

impl ScaffoldProcessor {
    /// The kind rendered for `name` (and `within`, for a kind that lives inside another component). An unknown kind
    /// or a name that isn't snake_case is an InvalidToolCall, told to the model.
    pub fn plan(
        &self,
        stack_pack_scaffold: &StackPackScaffold,
        kind_name: &str,
        name: &str,
        within: Option<&str>,
    ) -> ApplicationResult<ScaffoldPlan> {
        let scaffold_kind: &ScaffoldKind = stack_pack_scaffold
            .kinds
            .iter()
            .find(|scaffold_kind: &&ScaffoldKind| scaffold_kind.name == kind_name)
            .ok_or_else(|| {
                let kind_names: Vec<&str> = stack_pack_scaffold
                    .kinds
                    .iter()
                    .map(|scaffold_kind: &ScaffoldKind| scaffold_kind.name.as_str())
                    .collect();
                ScaffoldProcessor::to_invalid_tool_call(format!(
                    "there is no scaffold kind {kind_name}; the kinds are {}",
                    kind_names.join(", ")
                ))
            })?;

        ScaffoldProcessor::check_name("name", name)?;
        let within: Option<&str> = match (scaffold_kind.within_path.is_some(), within) {
            (true, Some(within)) => {
                ScaffoldProcessor::check_name("within", within)?;
                Some(within)
            }
            (true, None) => {
                return Err(ScaffoldProcessor::to_invalid_tool_call(format!(
                    "the {kind_name} kind lives inside another component: give `within` its name"
                )));
            }
            (false, _) => None,
        };

        let render = |template: &str| -> String { ScaffoldProcessor::render(template, name, within) };
        Ok(ScaffoldPlan {
            within_path: scaffold_kind
                .within_path
                .as_deref()
                .map(|within_path: &str| render(within_path)),
            created_files: scaffold_kind
                .files
                .iter()
                .map(|scaffold_file: &ScaffoldFile| ScaffoldFile {
                    path: render(&scaffold_file.path),
                    content: render(&scaffold_file.content),
                })
                .collect(),
            insertions: scaffold_kind
                .insertions
                .iter()
                .map(|scaffold_insertion: &ScaffoldInsertion| ScaffoldInsertion {
                    path: render(&scaffold_insertion.path),
                    before: scaffold_insertion.before.clone(),
                    text: render(&scaffold_insertion.text),
                })
                .collect(),
        })
    }

    /// The file's content with the insertion made: at the start of the line holding its anchor, or at the end. A
    /// missing anchor means the project isn't laid out as the scaffold expects: ScaffoldNotApplicable.
    pub fn insert(&self, content: &str, scaffold_insertion: &ScaffoldInsertion) -> ApplicationResult<String> {
        let Some(anchor) = scaffold_insertion.before.as_deref() else {
            let mut inserted: String = content.to_string();
            if !inserted.is_empty() && !inserted.ends_with('\n') {
                inserted.push('\n');
            }
            inserted.push_str(&scaffold_insertion.text);
            return Ok(inserted);
        };

        let anchor_index: usize = content.find(anchor).ok_or_else(|| {
            ApplicationBaseException::new(
                AgentRunEngineException::ScaffoldNotApplicable,
                format!(
                    "{} doesn't have the line the scaffold wires into (`{anchor}`): write this part by hand",
                    scaffold_insertion.path
                ),
                ContextualData::new().with("path", scaffold_insertion.path.as_str()),
            )
        })?;
        let line_start: usize = content[..anchor_index].rfind('\n').map_or(0, |index: usize| index + 1);
        Ok(format!(
            "{}{}{}",
            &content[..line_start],
            scaffold_insertion.text,
            &content[line_start..]
        ))
    }

    fn render(template: &str, name: &str, within: Option<&str>) -> String {
        let mut rendered: String = template
            .replace("{name}", name)
            .replace("{Name}", &ScaffoldProcessor::to_pascal_case(name));
        if let Some(within) = within {
            rendered = rendered
                .replace("{within}", within)
                .replace("{Within}", &ScaffoldProcessor::to_pascal_case(within));
        }
        rendered
    }

    fn check_name(argument_name: &str, name: &str) -> ApplicationResult<()> {
        if COMPONENT_NAME.is_match(name) {
            return Ok(());
        }
        Err(ScaffoldProcessor::to_invalid_tool_call(format!(
            "`{argument_name}` must be snake_case (e.g. customer_billing), not {name}"
        )))
    }

    fn to_pascal_case(name: &str) -> String {
        name.split('_')
            .map(|word: &str| {
                let mut characters: std::str::Chars<'_> = word.chars();
                match characters.next() {
                    Some(first) => first.to_uppercase().chain(characters).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect()
    }

    fn to_invalid_tool_call(reason: String) -> ApplicationBaseException {
        ApplicationBaseException::new(
            AgentRunEngineException::InvalidToolCall,
            format!("The call to scaffold is invalid: {reason}"),
            ContextualData::new().with("tool_name", "scaffold"),
        )
    }
}
