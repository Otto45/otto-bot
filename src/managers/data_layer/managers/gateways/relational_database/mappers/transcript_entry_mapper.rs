use crate::managers::data_layer::managers::gateways::relational_database::resource_models::transcript_entry_row::TranscriptEntryRow;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::transcript_speaker::TranscriptSpeaker;

pub struct TranscriptEntryMapper;

impl TranscriptEntryMapper {
    /// Expects a row that has already passed ValidatorTranscriptEntryRow: each `expect` states a checked fact, and
    /// fails only if the validator is wrong.
    pub fn to_transcript_entry(transcript_entry_row: TranscriptEntryRow) -> TranscriptEntry {
        let speaker: String = transcript_entry_row
            .speaker
            .expect("validated by ValidatorTranscriptEntryRow");
        TranscriptEntry {
            speaker: TranscriptSpeaker::from_name(&speaker).expect("validated by ValidatorTranscriptEntryRow"),
            text: transcript_entry_row
                .text
                .expect("validated by ValidatorTranscriptEntryRow"),
            said_on: transcript_entry_row
                .said_on
                .expect("validated by ValidatorTranscriptEntryRow"),
        }
    }
}
