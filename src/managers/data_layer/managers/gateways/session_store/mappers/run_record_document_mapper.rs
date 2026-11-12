use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::run_record_document::RunRecordDocument;
use crate::managers::models::run_record::RunRecord;
use crate::managers::models::run_status::RunStatus;

pub struct RunRecordDocumentMapper;

impl RunRecordDocumentMapper {
    pub fn to_run_record_document(run_record: &RunRecord) -> RunRecordDocument {
        RunRecordDocument {
            status: Some(run_record.status.name().to_string()),
            recorded_on: Some(run_record.recorded_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }

    /// Expects a document validated by `ValidatorRunRecordDocument`.
    pub fn to_run_record(document: RunRecordDocument) -> RunRecord {
        let status: String = document.status.expect("validated by ValidatorRunRecordDocument");
        let recorded_on: String = document.recorded_on.expect("validated by ValidatorRunRecordDocument");
        RunRecord {
            status: RunStatus::from_name(&status).expect("validated by ValidatorRunRecordDocument"),
            recorded_on: DateTime::parse_from_rfc3339(&recorded_on)
                .expect("validated by ValidatorRunRecordDocument")
                .with_timezone(&Utc),
        }
    }
}
