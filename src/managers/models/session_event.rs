use crate::managers::models::run_summary::RunSummary;

/// What happens in a session, as every client sees it. A run starts with `RunStarted` and ends with exactly one of
/// `RunCompleted`, `RunFailed` or `RunCancelled`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    RunStarted,
    /// The agent wrote to the person.
    AssistantReplied {
        text: String,
    },
    /// The agent used a tool: which, on what (e.g. the paths it read), and whether it worked.
    ToolUsed {
        tool_name: String,
        target: String,
        succeeded: bool,
    },
    RunCompleted {
        run_summary: RunSummary,
    },
    /// The run stopped on an exception, which has been logged; the message says what failed.
    RunFailed {
        message: String,
    },
    RunCancelled,
}

impl SessionEvent {
    /// Whether the event ends a run.
    pub fn ends_run(&self) -> bool {
        matches!(
            self,
            SessionEvent::RunCompleted { .. } | SessionEvent::RunFailed { .. } | SessionEvent::RunCancelled
        )
    }
}
