use serde::{Deserialize, Serialize};

// One line of a session's conversation_items.jsonl. Which fields it has depends on its type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConversationItemDocument {
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    // For a model output that is a tool call: which kind (its call_id is in call_id).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub succeeded: Option<bool>,
}
