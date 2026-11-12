use crate::managers::data_layer::managers::gateways::relational_database::resource_models::run_record_row::RunRecordRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::managers::models::run_status::RunStatus;

pub struct ValidatorRunRecordRow;

impl ValidatorRunRecordRow {
    pub fn validate_run_record_row(run_record_row: &RunRecordRow, session_id: &str) -> ApplicationResult<()> {
        let checks: [(&str, bool); 2] = [
            (
                "status (started, building, built, completed, failed or cancelled)",
                run_record_row
                    .status
                    .as_deref()
                    .and_then(RunStatus::from_name)
                    .is_some(),
            ),
            ("recorded_on", run_record_row.recorded_on.is_some()),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresRunRecordRow,
                "Run record row from the Postgres run_records table failed validation",
                ContextualData::new()
                    .with("resource_name", "run_record")
                    .with("session_id", session_id)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
