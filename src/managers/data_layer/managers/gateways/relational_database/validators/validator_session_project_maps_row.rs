use crate::managers::data_layer::managers::gateways::relational_database::resource_models::session_project_maps_row::SessionProjectMapsRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;

pub struct ValidatorSessionProjectMapsRow;

impl ValidatorSessionProjectMapsRow {
    // Both maps are None before a session's first run; a layer map is a map too, so there is no layer map without a
    // latest one.
    pub fn validate_session_project_maps_row(
        session_project_maps_row: &SessionProjectMapsRow,
        session_id: &str,
    ) -> ApplicationResult<()> {
        let checks: [(&str, bool); 1] = [(
            "latest_map (there is a layer map)",
            session_project_maps_row.layer_map.is_none() || session_project_maps_row.latest_map.is_some(),
        )];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(column_name, _): (&str, bool)| format!("{column_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresSessionProjectMapsRow,
                "Project maps row from the Postgres project_maps table failed validation",
                ContextualData::new()
                    .with("resource_name", "session_project_maps")
                    .with("session_id", session_id)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
