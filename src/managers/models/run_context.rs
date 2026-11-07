use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::prompt_layer::PromptLayer;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::tool_definition::ToolDefinition;

/// What a run works with for its whole length: the prompt layers and tools every turn sends, the workspace's stack (if
/// a pack matches it), and the note of how the project map changed since the last run (if it did).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunContext {
    pub prompt_layers: Vec<PromptLayer>,
    pub tools: Vec<ToolDefinition>,
    pub stack_pack: Option<StackPack>,
    pub project_map_note: Option<ConversationItem>,
}
