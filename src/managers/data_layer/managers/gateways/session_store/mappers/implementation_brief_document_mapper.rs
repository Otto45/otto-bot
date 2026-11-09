use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::implementation_brief_document::ImplementationBriefDocument;
use crate::managers::models::implementation_brief::ImplementationBrief;

pub struct ImplementationBriefDocumentMapper;

impl ImplementationBriefDocumentMapper {
    pub fn to_implementation_brief_document(
        implementation_brief: &ImplementationBrief,
        accepted_on: DateTime<Utc>,
    ) -> ImplementationBriefDocument {
        ImplementationBriefDocument {
            task: Some(implementation_brief.task.clone()),
            decisions: Some(implementation_brief.decisions.clone()),
            done_when: Some(implementation_brief.done_when.clone()),
            files: Some(implementation_brief.files.clone()),
            accepted_on: Some(accepted_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }
}
