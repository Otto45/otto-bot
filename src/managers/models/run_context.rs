use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::tool_definition::ToolDefinition;

/// What a run works with for its whole length: the instructions and tools every turn sends, and the workspace's
/// stack, if a pack matches it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunContext {
    pub instructions: String,
    pub tools: Vec<ToolDefinition>,
    pub stack_pack: Option<StackPack>,
}
