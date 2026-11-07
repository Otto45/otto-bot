use serde::{Deserialize, Serialize};

// One line of a session's usage_records.jsonl: the usage ledger.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UsageRecordDocument {
    pub session_id: Option<String>,
    pub response_id: Option<String>,
    pub model: Option<String>,
    pub requested_service_tier: Option<String>,
    pub served_service_tier: Option<String>,
    pub input_tokens: Option<i64>,
    pub cached_input_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub recorded_on: Option<String>,
}
