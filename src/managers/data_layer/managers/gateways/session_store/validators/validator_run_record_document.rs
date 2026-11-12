use chrono::DateTime;

use crate::managers::data_layer::managers::gateways::session_store::resource_models::run_record_document::RunRecordDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::models::run_status::RunStatus;

pub struct ValidatorRunRecordDocument;

impl ValidatorRunRecordDocument {
    pub fn validate_run_record_document(
        document: &RunRecordDocument,
        session_id: &str,
        line_number: i64,
    ) -> ApplicationResult<()> {
        let checks: [(&str, bool); 2] = [
            (
                "status (started, building, built, completed, failed or cancelled)",
                document.status.as_deref().and_then(RunStatus::from_name).is_some(),
            ),
            (
                "recorded_on (an RFC 3339 time)",
                document
                    .recorded_on
                    .as_deref()
                    .is_some_and(|recorded_on: &str| DateTime::parse_from_rfc3339(recorded_on).is_ok()),
            ),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(property_name, _): (&str, bool)| format!("{property_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                SessionStoreGatewayException::InvalidRunRecordDocument,
                "A run record in the session store is invalid",
                ContextualData::new()
                    .with("resource_name", "run_record")
                    .with("session_id", session_id)
                    .with("line_number", line_number)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
