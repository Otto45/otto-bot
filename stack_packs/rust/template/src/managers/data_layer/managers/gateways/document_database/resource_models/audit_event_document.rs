use chrono::{DateTime, Utc};
use serde::Deserialize;

// A document as the database stores it: every field optional, so that any document deserializes and the validator
// can say what is wrong with it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AuditEventDocument {
    pub event_id: Option<String>,
    pub customer_id: Option<String>,
    pub description: Option<String>,
    pub occurred_on: Option<DateTime<Utc>>,
}
