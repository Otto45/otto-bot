/// What the orchestrator asks the implementor to build: the task as the person and the orchestrator settled it, what
/// was decided along the way, what done looks like, and the files to start from. The implementor plans the change
/// itself, from a fresh conversation, so the brief says what to build, not how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationBrief {
    pub task: String,
    /// What was decided with the person, or by the orchestrator where the person left it open.
    pub decisions: Vec<String>,
    /// How the implementor (and the reviewer) knows the change is finished.
    pub done_when: String,
    /// Paths relative to the workspace, which the harness reads into the implementor's first message.
    pub files: Vec<String>,
}
