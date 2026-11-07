use serde::{Deserialize, Serialize};

// A component of a work plan, as plans.jsonl keeps it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PlannedComponentDocument {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub within: Option<String>,
}
