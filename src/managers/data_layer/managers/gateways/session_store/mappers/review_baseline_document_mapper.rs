use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::review_baseline_document::ReviewBaselineDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::workspace_file_document::WorkspaceFileDocument;
use crate::managers::models::workspace_file::WorkspaceFile;

pub struct ReviewBaselineDocumentMapper;

impl ReviewBaselineDocumentMapper {
    /// `files` is the baseline; None says everything the session built has been reviewed.
    pub fn to_review_baseline_document(
        files: Option<&[WorkspaceFile]>,
        recorded_on: DateTime<Utc>,
    ) -> ReviewBaselineDocument {
        ReviewBaselineDocument {
            reviewed: Some(files.is_none()),
            files: files.map(|files: &[WorkspaceFile]| {
                files
                    .iter()
                    .map(|workspace_file: &WorkspaceFile| WorkspaceFileDocument {
                        path: Some(workspace_file.path.clone()),
                        content: Some(workspace_file.content.clone()),
                    })
                    .collect()
            }),
            recorded_on: Some(recorded_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }

    /// Expects a document validated by `ValidatorReviewBaselineDocument`. The baseline's files; None when the
    /// document says everything was reviewed.
    pub fn to_review_baseline(document: ReviewBaselineDocument) -> Option<Vec<WorkspaceFile>> {
        if document.reviewed.expect("validated by ValidatorReviewBaselineDocument") {
            return None;
        }
        Some(
            document
                .files
                .expect("validated by ValidatorReviewBaselineDocument")
                .into_iter()
                .map(|file: WorkspaceFileDocument| WorkspaceFile {
                    path: file.path.expect("validated by ValidatorReviewBaselineDocument"),
                    content: file.content.expect("validated by ValidatorReviewBaselineDocument"),
                })
                .collect(),
        )
    }
}
