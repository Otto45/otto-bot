use serde::{Deserialize, Serialize};

// A file of a review baseline, as the `files` column's JSON text holds it: its path in the workspace, and its text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ReviewBaselineFileResource {
    pub path: Option<String>,
    pub content: Option<String>,
}
