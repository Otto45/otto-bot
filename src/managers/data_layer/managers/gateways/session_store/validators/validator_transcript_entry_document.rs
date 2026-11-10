use chrono::DateTime;

use crate::managers::data_layer::managers::gateways::session_store::resource_models::transcript_entry_document::TranscriptEntryDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::models::transcript_speaker::TranscriptSpeaker;

pub struct ValidatorTranscriptEntryDocument;

impl ValidatorTranscriptEntryDocument {
    pub fn validate_transcript_entry_document(
        document: &TranscriptEntryDocument,
        session_id: &str,
        line_number: i64,
    ) -> ApplicationResult<()> {
        let checks: [(&str, bool); 3] = [
            (
                "speaker (person or assistant)",
                document
                    .speaker
                    .as_deref()
                    .and_then(TranscriptSpeaker::from_name)
                    .is_some(),
            ),
            ("text", document.text.is_some()),
            (
                "said_on (an RFC 3339 time)",
                document
                    .said_on
                    .as_deref()
                    .is_some_and(|said_on: &str| DateTime::parse_from_rfc3339(said_on).is_ok()),
            ),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(property_name, _): (&str, bool)| format!("{property_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                SessionStoreGatewayException::InvalidTranscriptEntryDocument,
                "A transcript entry in the session store is invalid",
                ContextualData::new()
                    .with("resource_name", "transcript_entry")
                    .with("session_id", session_id)
                    .with("line_number", line_number)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
