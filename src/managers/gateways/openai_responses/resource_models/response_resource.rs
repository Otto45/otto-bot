use serde::Deserialize;

// Resource models: the JSON body deserialized as it is, with every field optional so that anything the API sends
// deserializes and the validator can say what is wrong with it. Fields the harness doesn't use are left out.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ContentResource {
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OutputItemResource {
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub role: Option<String>,
    pub content: Option<Vec<ContentResource>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct InputTokensDetailsResource {
    pub cached_tokens: Option<i64>,
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
    pub status: Option<String>,
    pub incomplete_details: Option<IncompleteDetailsResource>,
    pub output: Option<Vec<OutputItemResource>>,
    pub usage: Option<UsageResource>,
    pub service_tier: Option<String>,
}
