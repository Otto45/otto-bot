use serde::{Deserialize, Serialize};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::workspace_file_document::WorkspaceFileDocument;

// One line of a session's review_baselines.jsonl. The last line says what the session's next review is made
// against: `files` (the workspace's text files as they were when everything in it had been reviewed), or nothing,
// when `reviewed` says all the session built has been reviewed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ReviewBaselineDocument {
    pub reviewed: Option<bool>,
    pub files: Option<Vec<WorkspaceFileDocument>>,
    pub recorded_on: Option<String>,
}
