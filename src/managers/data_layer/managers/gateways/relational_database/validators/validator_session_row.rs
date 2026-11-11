use crate::managers::data_layer::managers::gateways::relational_database::resource_models::session_row::SessionRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::managers::models::service_tier::ServiceTier;

pub struct ValidatorSessionRow;

impl ValidatorSessionRow {
    pub fn validate_session_row(session_row: &SessionRow) -> ApplicationResult<()> {
        let checks: [(&str, bool); 4] = [
            (
                "session_id",
                session_row
                    .session_id
                    .as_deref()
                    .is_some_and(|session_id: &str| !session_id.is_empty()),
            ),
            (
                "workspace_directory",
                session_row
                    .workspace_directory
                    .as_deref()
                    .is_some_and(|workspace_directory: &str| !workspace_directory.is_empty()),
            ),
            (
                "service_tier (one of standard, fast, ultrafast)",
                session_row
                    .service_tier
                    .as_deref()
                    .and_then(ServiceTier::from_name)
                    .is_some(),
            ),
            ("started_on", session_row.started_on.is_some()),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresSessionRow,
                "Session row from the Postgres sessions table failed validation",
                ContextualData::new()
                    .with("resource_name", "session")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
