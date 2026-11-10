use serde::Deserialize;
use serde_json::Value;

use crate::managers::gateways::openai_responses::resource_models::response_resource::UsageResource;

// What `/responses/compact` answers: the compacted conversation and its usage. It has no status: a compaction that
// failed is an error status.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CompactionResource {
    pub id: Option<String>,
    // Kept as the API sent them: each is sent back unchanged in the lane's next requests.
    pub output: Option<Vec<Value>>,
    pub usage: Option<UsageResource>,
}
