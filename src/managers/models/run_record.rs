use chrono::{DateTime, Utc};

use crate::managers::models::run_status::RunStatus;

/// One step of a session's runs, kept as it happens, so that what a run was doing when it stopped is known to the
/// session's next run, in this process or a later one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub status: RunStatus,
    pub recorded_on: DateTime<Utc>,
}
