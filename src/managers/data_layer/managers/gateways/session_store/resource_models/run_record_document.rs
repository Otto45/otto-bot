use serde::{Deserialize, Serialize};

// One line of a session's run_records.jsonl: a step one of its runs got to.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RunRecordDocument {
    pub status: Option<String>,
    pub recorded_on: Option<String>,
}
