use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TranscriptMessageResponse {
    /// person or assistant.
    pub speaker: String,
    pub text: String,
    pub said_on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TranscriptResponse {
    /// Oldest first, in plain words: what the person wrote and what otto answered.
    pub messages: Vec<TranscriptMessageResponse>,
}
