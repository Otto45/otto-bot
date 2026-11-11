use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::transcript_entry::TranscriptEntry;

pub struct TranscriptEntryQueryBuilder;

impl TranscriptEntryQueryBuilder {
    pub fn build_append_transcript_entry_query(session_id: &str, transcript_entry: &TranscriptEntry) -> SqlQuery {
        SqlQuery {
            statement: "INSERT INTO transcript_entries (session_id, speaker, text, said_on) VALUES ($1, $2, $3, $4)",
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::Text(transcript_entry.speaker.name().to_string()),
                SqlParameter::Text(transcript_entry.text.clone()),
                SqlParameter::Timestamp(transcript_entry.said_on),
            ],
        }
    }

    // Oldest first: the identity column numbers them in the order they were said.
    pub fn build_get_transcript_query(session_id: &str) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "SELECT speaker, text, said_on FROM transcript_entries ",
                "WHERE session_id = $1 ORDER BY transcript_entry_id",
            ),
            parameters: vec![SqlParameter::Text(session_id.to_string())],
        }
    }
}
