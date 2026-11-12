use crate::managers::data_layer::managers::gateways::session_store::resource_models::review_baseline_document::ReviewBaselineDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::workspace_file_document::WorkspaceFileDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;

pub struct ValidatorReviewBaselineDocument;

impl ValidatorReviewBaselineDocument {
    /// A baseline says whether everything was reviewed; one that says it wasn't has its files, each with a path and
    /// its content.
    pub fn validate_review_baseline_document(
        document: &ReviewBaselineDocument,
        session_id: &str,
        line_number: i64,
    ) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        match (document.reviewed, document.files.as_deref()) {
            (None, _) => errors.push("reviewed is missing".to_string()),
            (Some(true), _) => {}
            (Some(false), None) => errors.push("files is missing".to_string()),
            (Some(false), Some(files)) => {
                errors.extend(
                    files
                        .iter()
                        .enumerate()
                        .filter(|(_, file): &(usize, &WorkspaceFileDocument)| {
                            file.path.is_none() || file.content.is_none()
                        })
                        .map(|(index, _): (usize, &WorkspaceFileDocument)| {
                            format!("files[{index}] has no path or no content")
                        }),
                );
            }
        }
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                SessionStoreGatewayException::InvalidReviewBaselineDocument,
                "A review baseline in the session store is invalid",
                ContextualData::new()
                    .with("resource_name", "review_baseline")
                    .with("session_id", session_id)
                    .with("line_number", line_number)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
