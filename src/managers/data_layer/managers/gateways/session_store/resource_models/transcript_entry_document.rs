use serde::{Deserialize, Serialize};

// One line of a session's transcript.jsonl: something the person or the orchestrator said.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TranscriptEntryDocument {
    pub speaker: Option<String>,
    pub text: Option<String>,
    pub said_on: Option<String>,
}
