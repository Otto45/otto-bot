use serde::{Deserialize, Serialize};

// One line of a session's briefs.jsonl: a brief the harness accepted, in the order they were accepted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ImplementationBriefDocument {
    pub task: Option<String>,
    pub decisions: Option<Vec<String>>,
    pub done_when: Option<String>,
    pub files: Option<Vec<String>>,
    pub accepted_on: Option<String>,
}
