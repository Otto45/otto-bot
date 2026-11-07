use crate::managers::models::patch_operation::PatchOperation;

/// A tool call the model made, which the harness answers with one output carrying the same call_id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCall {
    /// A function tool; `arguments` is JSON text, read by whoever knows the tool.
    Function {
        call_id: String,
        name: String,
        arguments: String,
    },
    ApplyPatch {
        call_id: String,
        patch_operation: PatchOperation,
    },
}
