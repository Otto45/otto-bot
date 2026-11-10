use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::implementation_result::ImplementationResult;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::run_summary::RunSummary;

/// What happens in a session, as every client sees it. A run starts with `RunStarted` and ends with exactly one of
/// `RunCompleted`, `RunFailed` or `RunCancelled`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    RunStarted,
    /// An agent wrote to the person. `agent` names the agent (its lane), unless it is the session's own (the
    /// orchestrator).
    AssistantReplied {
        agent: Option<String>,
        text: String,
    },
    /// An agent used a tool: which, on what (e.g. the paths it read), and whether it worked.
    ToolUsed {
        agent: Option<String>,
        tool_name: String,
        target: String,
        succeeded: bool,
    },
    /// The orchestrator's plan passed the harness's checks; the implementor builds it.
    BriefAccepted {
        implementation_brief: ImplementationBrief,
    },
    /// The implementor finished building the accepted plan.
    BriefImplemented {
        implementation_result: ImplementationResult,
    },
    RunCompleted {
        run_summary: RunSummary,
    },
    /// An agent's conversation was compacted, so its requests stay short: during its run (a long task), or the
    /// orchestrator's after a run has ended (which a client may see before the next run's events). `agent` is as in
    /// `AssistantReplied`; `usage` is what compacting it used.
    LaneCompacted {
        agent: Option<String>,
        usage: ModelUsage,
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
