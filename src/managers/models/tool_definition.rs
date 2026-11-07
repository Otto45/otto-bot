use serde_json::Value;

/// A function tool the model may call: its name, what it does, and its parameters as a JSON Schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}
