use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::managers::models::service_tier::ServiceTier;

/// A session: the work one client asks for, request by request, in one workspace, under the options it was started
/// with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    /// Absolute and canonical: every path the agent uses is relative to it.
    pub workspace_directory: PathBuf,
    pub service_tier: ServiceTier,
    pub started_on: DateTime<Utc>,
}
