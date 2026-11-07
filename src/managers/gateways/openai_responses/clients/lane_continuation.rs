use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::prompt_layer::PromptLayer;

/// What a lane's connection holds: its latest response, and the prompt and conversation that response covers (the
/// request's items, then the response's own). A request that opens with exactly these continues it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneContinuation {
    pub response_id: String,
    pub prompt_layers: Vec<PromptLayer>,
    pub items: Vec<ConversationItem>,
}
