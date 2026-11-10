use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::services::http::wire_models::transcript_response::{TranscriptMessageResponse, TranscriptResponse};

pub struct TranscriptResponseMapper;

impl TranscriptResponseMapper {
    pub fn to_transcript_response(transcript_entries: Vec<TranscriptEntry>) -> TranscriptResponse {
        TranscriptResponse {
            messages: transcript_entries
                .into_iter()
                .map(|transcript_entry: TranscriptEntry| TranscriptMessageResponse {
                    speaker: transcript_entry.speaker.name().to_string(),
                    text: transcript_entry.text,
                    said_on: transcript_entry.said_on,
                })
                .collect(),
        }
    }
}
