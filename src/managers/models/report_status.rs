/// How an agent's assignment ended, as it reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportStatus {
    /// Its part is built, and the checks pass.
    Done,
    /// Its part can't be finished as planned (the plan is wrong, or something it needs is missing).
    Blocked,
}

impl ReportStatus {
    pub fn name(self) -> &'static str {
        match self {
            ReportStatus::Done => "done",
            ReportStatus::Blocked => "blocked",
        }
    }

    pub fn from_name(name: &str) -> Option<ReportStatus> {
        match name {
            "done" => Some(ReportStatus::Done),
            "blocked" => Some(ReportStatus::Blocked),
            _ => None,
        }
    }
}
