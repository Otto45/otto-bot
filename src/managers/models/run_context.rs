use crate::managers::models::prompt_layer::PromptLayer;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::tool_definition::ToolDefinition;

/// What an agent's run works with for its whole length: the prompt layers and tools every turn sends, which tools its
/// role may call, and the workspace's stack (if a pack matches it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunContext {
    pub prompt_layers: Vec<PromptLayer>,
    pub tools: Vec<ToolDefinition>,
    /// The tools the agent's role may call; none for every tool.
    pub allowed_tool_names: Option<Vec<String>>,
    pub stack_pack: Option<StackPack>,
}
