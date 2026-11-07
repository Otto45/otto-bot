use crate::managers::models::tool_call_kind::ToolCallKind;

/// Which tool call a model output item is: every one must be answered by an output with the same call_id before the
/// conversation can go on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallReference {
    pub call_id: String,
    pub kind: ToolCallKind,
}
