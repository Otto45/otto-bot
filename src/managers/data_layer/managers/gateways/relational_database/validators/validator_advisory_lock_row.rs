use crate::managers::data_layer::managers::gateways::relational_database::resource_models::advisory_lock_row::AdvisoryLockRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;

pub struct ValidatorAdvisoryLockRow;

impl ValidatorAdvisoryLockRow {
    pub fn validate_advisory_lock_row(advisory_lock_row: &AdvisoryLockRow) -> ApplicationResult<()> {
        let checks: [(&str, bool); 1] = [("locked", advisory_lock_row.locked.is_some())];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresAdvisoryLockRow,
                "Advisory lock row from Postgres failed validation",
                ContextualData::new()
                    .with("resource_name", "advisory_lock")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
