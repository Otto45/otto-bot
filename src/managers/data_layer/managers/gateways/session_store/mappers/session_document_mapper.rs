use chrono::SecondsFormat;

use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_document::SessionDocument;
use crate::managers::models::session::Session;

pub struct SessionDocumentMapper;

impl SessionDocumentMapper {
    pub fn to_session_document(session: &Session) -> SessionDocument {
        SessionDocument {
            id: Some(session.id.clone()),
            workspace_directory: Some(session.workspace_directory.to_string_lossy().into_owned()),
            service_tier: Some(session.service_tier.name().to_string()),
            started_on: Some(session.started_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }
}
