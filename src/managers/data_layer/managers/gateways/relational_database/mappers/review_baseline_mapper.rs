use crate::managers::data_layer::managers::gateways::relational_database::resource_models::review_baseline_file_resource::ReviewBaselineFileResource;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::review_baseline_row::ReviewBaselineRow;
use crate::managers::models::workspace_file::WorkspaceFile;

pub struct ReviewBaselineMapper;

impl ReviewBaselineMapper {
    /// The baseline's files as the `files` column holds them: JSON text.
    pub fn to_files_text(files: &[WorkspaceFile]) -> String {
        let file_resources: Vec<ReviewBaselineFileResource> = files
            .iter()
            .map(|workspace_file: &WorkspaceFile| ReviewBaselineFileResource {
                path: Some(workspace_file.path.clone()),
                content: Some(workspace_file.content.clone()),
            })
            .collect();
        serde_json::to_string(&file_resources).expect("a review baseline's files serialize to JSON")
    }

    /// Expects a row that has already passed ValidatorReviewBaselineRow: each `expect` states a checked fact, and
    /// fails only if the validator is wrong. The baseline's files; None when the row says everything was reviewed.
    pub fn to_review_baseline(review_baseline_row: ReviewBaselineRow) -> Option<Vec<WorkspaceFile>> {
        if review_baseline_row
            .reviewed
            .expect("validated by ValidatorReviewBaselineRow")
        {
            return None;
        }
        let files: String = review_baseline_row
            .files
            .expect("validated by ValidatorReviewBaselineRow");
        let file_resources: Vec<ReviewBaselineFileResource> =
            serde_json::from_str(&files).expect("validated by ValidatorReviewBaselineRow");
        Some(
            file_resources
                .into_iter()
                .map(|file: ReviewBaselineFileResource| WorkspaceFile {
                    path: file.path.expect("validated by ValidatorReviewBaselineRow"),
                    content: file.content.expect("validated by ValidatorReviewBaselineRow"),
                })
                .collect(),
        )
    }
}
