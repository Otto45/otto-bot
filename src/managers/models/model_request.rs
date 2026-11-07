use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::prompt_layer::PromptLayer;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::tool_definition::ToolDefinition;

/// One request to the model, in the domain's terms; the gateway turns it into the API's. The native apply_patch tool
/// is always offered alongside `tools`, and `prompt_layers` open the request, each cached on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequest {
    pub prompt_layers: Vec<PromptLayer>,
    pub items: Vec<ConversationItem>,
    pub tools: Vec<ToolDefinition>,
    pub service_tier: ServiceTier,
}
