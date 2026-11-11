use crate::managers::data_layer::managers::gateways::relational_database::resource_models::implementation_brief_count_row::ImplementationBriefCountRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;

pub struct ValidatorImplementationBriefCountRow;

impl ValidatorImplementationBriefCountRow {
    pub fn validate_implementation_brief_count_row(
        implementation_brief_count_row: &ImplementationBriefCountRow,
        session_id: &str,
    ) -> ApplicationResult<()> {
        let checks: [(&str, bool); 1] = [(
            "implementation_brief_count (a count)",
            implementation_brief_count_row
                .implementation_brief_count
                .is_some_and(|implementation_brief_count: i64| usize::try_from(implementation_brief_count).is_ok()),
        )];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresImplementationBriefCountRow,
                "Brief count row from the Postgres implementation_briefs table failed validation",
                ContextualData::new()
                    .with("resource_name", "implementation_brief_count")
                    .with("session_id", session_id)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
