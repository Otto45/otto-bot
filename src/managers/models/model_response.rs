use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::tool_call::ToolCall;

/// What the model answered with, in one turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResponse {
    pub id: String,
    pub model: String,
    /// Every item the model produced, as the conversation keeps them (ConversationItem::ModelOutput).
    pub output_items: Vec<ConversationItem>,
    /// The text of its messages, if it wrote any.
    pub text: Option<String>,
    /// What it asked the harness to do; none means it has finished.
    pub tool_calls: Vec<ToolCall>,
    pub usage: ModelUsage,
    /// The tier the response actually ran at: a fast request can be served at the standard tier under load. None
    /// when the API names a tier the harness never asks for.
    pub service_tier: Option<ServiceTier>,
}
