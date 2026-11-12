use crate::managers::data_layer::managers::gateways::relational_database::resource_models::review_baseline_file_resource::ReviewBaselineFileResource;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::review_baseline_row::ReviewBaselineRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;

pub struct ValidatorReviewBaselineRow;

impl ValidatorReviewBaselineRow {
    /// A baseline says whether everything was reviewed; one that says it wasn't has its files as JSON text: an array
    /// of files, each with a path and its content.
    pub fn validate_review_baseline_row(
        review_baseline_row: &ReviewBaselineRow,
        session_id: &str,
    ) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        match (review_baseline_row.reviewed, review_baseline_row.files.as_deref()) {
            (None, _) => errors.push("reviewed is missing".to_string()),
            (Some(true), _) => {}
            (Some(false), None) => errors.push("files is missing".to_string()),
            (Some(false), Some(files)) => match serde_json::from_str::<Vec<ReviewBaselineFileResource>>(files) {
                Err(exception) => errors.push(format!("files is not a JSON array of files: {exception}")),
                Ok(files) => errors.extend(
                    files
                        .iter()
                        .enumerate()
                        .filter(|(_, file): &(usize, &ReviewBaselineFileResource)| {
                            file.path.is_none() || file.content.is_none()
                        })
                        .map(|(index, _): (usize, &ReviewBaselineFileResource)| {
                            format!("files[{index}] has no path or no content")
                        }),
                ),
            },
        }
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresReviewBaselineRow,
                "Review baseline row from the Postgres review_baselines table failed validation",
                ContextualData::new()
                    .with("resource_name", "review_baseline")
                    .with("session_id", session_id)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
