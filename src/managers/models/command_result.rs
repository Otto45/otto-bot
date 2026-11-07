/// How a command the harness ran ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandResult {
    /// None when the command was stopped (by a signal, or for taking too long).
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    /// What it wrote to stdout, then stderr.
    pub output: String,
}
