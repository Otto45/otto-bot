use chrono::{DateTime, Utc};

use crate::managers::models::transcript_speaker::TranscriptSpeaker;

/// One thing said in a session's conversation with the person, in plain words: what the person wrote, or what the
/// orchestrator answered. Tool calls, briefs and notes are not part of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptEntry {
    pub speaker: TranscriptSpeaker,
    pub text: String,
    pub said_on: DateTime<Utc>,
}
