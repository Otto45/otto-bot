use crate::managers::models::model_usage::ModelUsage;

/// A lane's conversation as the API compacted it: the items to send from now on in place of it (every user and
/// developer message, then the encrypted compaction item), each as the API returned it, as JSON text, without the
/// prompt layers; and what compacting it used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactedConversation {
    pub id: String,
    pub model: String,
    pub items: Vec<String>,
    pub usage: ModelUsage,
}
