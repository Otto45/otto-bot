use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::tool_definition::ToolDefinition;

/// One request to the model, in the domain's terms; the gateway turns it into the API's. The native apply_patch tool
/// is always offered alongside `tools`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequest {
    pub instructions: String,
    pub items: Vec<ConversationItem>,
    pub tools: Vec<ToolDefinition>,
    pub service_tier: ServiceTier,
}
