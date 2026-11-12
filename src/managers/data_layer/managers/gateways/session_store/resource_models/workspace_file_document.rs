use serde::{Deserialize, Serialize};

// A file of a review baseline: its path in the workspace, and its text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceFileDocument {
    pub path: Option<String>,
    pub content: Option<String>,
}
