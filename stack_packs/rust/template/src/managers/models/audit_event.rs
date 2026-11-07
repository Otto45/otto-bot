use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub id: String,
    pub customer_id: String,
    pub description: String,
    pub occurred_on: DateTime<Utc>,
}
