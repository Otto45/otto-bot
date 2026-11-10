use serde::{Deserialize, Serialize};

// One line of a session's requests.jsonl: a request it was given.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionRequestDocument {
    pub text: Option<String>,
    pub submitted_on: Option<String>,
}
