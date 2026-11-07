use serde::Deserialize;
use serde_json::Value;

// Resource models: the JSON body deserialized as it is, with every field optional so that anything the API sends
// deserializes and the validator can say what is wrong with it. Fields the harness doesn't use are left out.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct InputTokensDetailsResource {
    pub cached_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OutputTokensDetailsResource {
    pub reasoning_tokens: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UsageResource {
    pub input_tokens: Option<i64>,
    pub input_tokens_details: Option<InputTokensDetailsResource>,
    pub output_tokens: Option<i64>,
    pub output_tokens_details: Option<OutputTokensDetailsResource>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct IncompleteDetailsResource {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ResponseResource {
    pub id: Option<String>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub incomplete_details: Option<IncompleteDetailsResource>,
    // Kept as the API sent them: each is sent back unchanged in the next request (see OutputItemResource for what
    // the harness reads of one).
    pub output: Option<Vec<Value>>,
    pub usage: Option<UsageResource>,
    pub service_tier: Option<String>,
}
