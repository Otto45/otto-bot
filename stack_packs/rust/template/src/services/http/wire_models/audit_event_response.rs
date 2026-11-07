use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuditEventResponse {
    pub id: String,
    pub customer_id: String,
    pub description: String,
    pub occurred_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuditEventListResponse {
    pub audit_events: Vec<AuditEventResponse>,
}
