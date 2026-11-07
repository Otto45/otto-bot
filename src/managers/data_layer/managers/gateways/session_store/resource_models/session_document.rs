use serde::{Deserialize, Serialize};

// Documents: a session's files' contents as they are written and read, every field optional so that anything read
// back deserializes and the validator can say what is wrong with it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionDocument {
    pub id: Option<String>,
    pub workspace_directory: Option<String>,
    pub service_tier: Option<String>,
    pub started_on: Option<String>,
}
