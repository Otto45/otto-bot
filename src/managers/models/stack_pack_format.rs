/// The stack's formatter, which the harness runs itself (before every check run, and after a scaffold), so no model
/// spends a turn on formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackFormat {
    pub command: Vec<String>,
    /// The files it formats, so the harness can say which ones it changed.
    pub source_globs: Vec<String>,
}
