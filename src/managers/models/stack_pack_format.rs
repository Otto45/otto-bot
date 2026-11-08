/// The stack's formatters (the code's formatting, generated files such as a contract), which the harness runs itself,
/// in order, before every check run and after a scaffold, so no model spends a turn on either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackFormat {
    pub commands: Vec<Vec<String>>,
    /// The files they write, so the harness can say which ones they changed.
    pub source_globs: Vec<String>,
}
