/// What a run of the stack's formatter did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatResult {
    /// False when it couldn't format every file (e.g. one doesn't parse); the format check then says why.
    pub succeeded: bool,
    /// The source files it changed, which an agent reads again before it patches them.
    pub reformatted_paths: Vec<String>,
}
