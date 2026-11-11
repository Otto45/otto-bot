use crate::managers::data_layer::managers::gateways::relational_database::resource_models::transcript_entry_row::TranscriptEntryRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::managers::models::transcript_speaker::TranscriptSpeaker;

pub struct ValidatorTranscriptEntryRow;

impl ValidatorTranscriptEntryRow {
    pub fn validate_transcript_entry_row(
        transcript_entry_row: &TranscriptEntryRow,
        session_id: &str,
    ) -> ApplicationResult<()> {
        let checks: [(&str, bool); 3] = [
            (
                "speaker (person or assistant)",
                transcript_entry_row
                    .speaker
                    .as_deref()
                    .and_then(TranscriptSpeaker::from_name)
                    .is_some(),
            ),
            ("text", transcript_entry_row.text.is_some()),
            ("said_on", transcript_entry_row.said_on.is_some()),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresTranscriptEntryRow,
                "Transcript entry row from the Postgres transcript_entries table failed validation",
                ContextualData::new()
                    .with("resource_name", "transcript_entry")
                    .with("session_id", session_id)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
