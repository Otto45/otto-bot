use serde::Deserialize;

// What the harness reads of one output item: a message's text, a function call, or an apply_patch call. Other items
// (reasoning) are only sent back.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ContentResource {
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PatchOperationResource {
    #[serde(rename = "type")]
    pub operation_type: Option<String>,
    pub path: Option<String>,
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OutputItemResource {
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub content: Option<Vec<ContentResource>>,
    pub call_id: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
    pub operation: Option<PatchOperationResource>,
}
