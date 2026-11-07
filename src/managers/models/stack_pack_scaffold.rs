use crate::managers::models::scaffold_kind::ScaffoldKind;

/// What a stack pack can scaffold. The pack's formatter runs afterwards, and sorts the lines a scaffold added in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackScaffold {
    pub kinds: Vec<ScaffoldKind>,
}
