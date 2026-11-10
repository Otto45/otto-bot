use crate::managers::models::reasoning_effort::ReasoningEffort;
use crate::managers::models::tool_call_reference::ToolCallReference;

/// One item of a lane's conversation, in the order the model saw it. With `store=false` the API keeps nothing, so
/// every request sends the whole conversation, and these are what it is made of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationItem {
    UserMessage {
        text: String,
    },
    /// Something the harness tells the model between requests (e.g. how the project map changed), sent as a developer
    /// message.
    HarnessNote {
        text: String,
    },
    /// Sets how hard the model reasons from here on (a lane's role decides it), without changing the cached prefix.
    ConfigurationUpdate {
        reasoning_effort: ReasoningEffort,
    },
    /// An item the model produced (a message, reasoning with its encrypted content, a tool call), kept exactly as the
    /// API returned it, as JSON text, so it is sent back unchanged. The domain reads what it needs from the
    /// ModelResponse, never from this, except which tool call the item is, if it is one.
    ModelOutput {
        item: String,
        tool_call: Option<ToolCallReference>,
    },
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
    ApplyPatchCallOutput {
        call_id: String,
        succeeded: bool,
        output: String,
    },
    /// The conversation before it, compacted by the API: the items it gave back (every user and developer message,
    /// then an encrypted `compaction` item that carries the rest), each kept exactly as the API returned it, as JSON
    /// text, without the prompt layers, which every request opens with anyway. A lane is loaded from its last one on.
    Compaction {
        items: Vec<String>,
    },
}
