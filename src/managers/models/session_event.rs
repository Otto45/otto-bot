use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;

/// What happens in a session, as every client sees it. A run starts with `RunStarted` and ends with exactly one of
/// `RunCompleted`, `RunFailed` or `RunCancelled`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    RunStarted,
    ModelResponded {
        text: String,
        usage: ModelUsage,
        service_tier: Option<ServiceTier>,
    },
    RunCompleted,
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
            SessionEvent::RunCompleted | SessionEvent::RunFailed { .. } | SessionEvent::RunCancelled
        )
    }
}
