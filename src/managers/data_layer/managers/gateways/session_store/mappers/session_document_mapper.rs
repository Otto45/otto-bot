use std::path::PathBuf;

use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_document::SessionDocument;
use crate::managers::models::service_tier::ServiceTier;
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

    /// Expects a document validated by `ValidatorSessionDocument`.
    pub fn to_session(document: SessionDocument) -> Session {
        let service_tier: String = document.service_tier.expect("validated by ValidatorSessionDocument");
        let started_on: String = document.started_on.expect("validated by ValidatorSessionDocument");
        Session {
            id: document.id.expect("validated by ValidatorSessionDocument"),
            workspace_directory: PathBuf::from(
                document
                    .workspace_directory
                    .expect("validated by ValidatorSessionDocument"),
            ),
            service_tier: ServiceTier::from_name(&service_tier).expect("validated by ValidatorSessionDocument"),
            started_on: DateTime::parse_from_rfc3339(&started_on)
                .expect("validated by ValidatorSessionDocument")
                .with_timezone(&Utc),
        }
    }
}
