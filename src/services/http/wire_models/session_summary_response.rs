use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SessionSummaryResponse {
    pub session_id: String,
    /// standard, fast or ultrafast.
    pub service_tier: String,
    pub started_on: DateTime<Utc>,
    /// When the person or otto last said anything, or when it started.
    pub last_active_on: DateTime<Utc>,
    /// How many messages the person has sent.
    pub request_count: usize,
    /// The person's first message, if they have sent one.
    pub first_request: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SessionListResponse {
    /// Most recently active first.
    pub sessions: Vec<SessionSummaryResponse>,
}
