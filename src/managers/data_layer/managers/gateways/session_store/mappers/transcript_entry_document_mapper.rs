use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::transcript_entry_document::TranscriptEntryDocument;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::transcript_speaker::TranscriptSpeaker;

pub struct TranscriptEntryDocumentMapper;

impl TranscriptEntryDocumentMapper {
    pub fn to_transcript_entry_document(transcript_entry: &TranscriptEntry) -> TranscriptEntryDocument {
        TranscriptEntryDocument {
            speaker: Some(transcript_entry.speaker.name().to_string()),
            text: Some(transcript_entry.text.clone()),
            said_on: Some(transcript_entry.said_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }

    /// Expects a document validated by `ValidatorTranscriptEntryDocument`.
    pub fn to_transcript_entry(document: TranscriptEntryDocument) -> TranscriptEntry {
        let speaker: String = document.speaker.expect("validated by ValidatorTranscriptEntryDocument");
        let said_on: String = document.said_on.expect("validated by ValidatorTranscriptEntryDocument");
        TranscriptEntry {
            speaker: TranscriptSpeaker::from_name(&speaker).expect("validated by ValidatorTranscriptEntryDocument"),
            text: document.text.expect("validated by ValidatorTranscriptEntryDocument"),
            said_on: DateTime::parse_from_rfc3339(&said_on)
                .expect("validated by ValidatorTranscriptEntryDocument")
                .with_timezone(&Utc),
        }
    }
}
