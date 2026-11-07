use crate::managers::models::scaffold_kind::ScaffoldKind;

/// What a stack pack can scaffold, and the command that formats the files afterwards (sorting the lines it added).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackScaffold {
    pub kinds: Vec<ScaffoldKind>,
    pub format_command: Vec<String>,
}
