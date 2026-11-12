use chrono::{DateTime, Utc};

// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct RunRecordRow {
    pub status: Option<String>,
    pub recorded_on: Option<DateTime<Utc>>,
}
