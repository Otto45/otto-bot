use serde::{Deserialize, Serialize};

// One line of a session's project_maps.jsonl: a project map a run started with, and its role: the latest "layer" is
// the one the session's requests open with; the latest of all is the one the last run started with.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectMapDocument {
    pub project_map: Option<String>,
    pub role: Option<String>,
    pub built_on: Option<String>,
}
