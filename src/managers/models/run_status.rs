/// How far a session's run has got, as its run records say: a run starts, may get as far as building a brief and
/// as having built it, and ends completed or failed, or cancelled. A run stopped with its process ends with no
/// record of its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Started,
    /// The orchestrator's brief was accepted, and the implementor builds it.
    Building,
    /// The brief was built (and reviewed, if it got that far); the orchestrator is still to be told how it went.
    Built,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn from_name(name: &str) -> Option<RunStatus> {
        match name {
            "started" => Some(RunStatus::Started),
            "building" => Some(RunStatus::Building),
            "built" => Some(RunStatus::Built),
            "completed" => Some(RunStatus::Completed),
            "failed" => Some(RunStatus::Failed),
            "cancelled" => Some(RunStatus::Cancelled),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RunStatus::Started => "started",
            RunStatus::Building => "building",
            RunStatus::Built => "built",
            RunStatus::Completed => "completed",
            RunStatus::Failed => "failed",
            RunStatus::Cancelled => "cancelled",
        }
    }
}
