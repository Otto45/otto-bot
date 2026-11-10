use chrono::{DateTime, Utc};

use crate::managers::models::session::Session;

/// A session as a client lists it, to pick one to resume: the session, and what it was asked so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    pub session: Session,
    /// None for a session that was never given a request.
    pub first_request: Option<String>,
    pub request_count: usize,
    /// When its last request came, or when it started if none has.
    pub last_active_on: DateTime<Utc>,
}
