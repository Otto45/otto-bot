use chrono::{DateTime, Utc};

// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct SessionRow {
    pub session_id: Option<String>,
    pub workspace_directory: Option<String>,
    pub service_tier: Option<String>,
    pub started_on: Option<DateTime<Utc>>,
}
