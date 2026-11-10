use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_request_document::SessionRequestDocument;
use crate::managers::models::session_request::SessionRequest;

pub struct SessionRequestDocumentMapper;

impl SessionRequestDocumentMapper {
    pub fn to_session_request_document(session_request: &SessionRequest) -> SessionRequestDocument {
        SessionRequestDocument {
            text: Some(session_request.text.clone()),
            submitted_on: Some(
                session_request
                    .submitted_on
                    .to_rfc3339_opts(SecondsFormat::Millis, true),
            ),
        }
    }

    /// Expects a document validated by `ValidatorSessionRequestDocument`.
    pub fn to_session_request(document: SessionRequestDocument) -> SessionRequest {
        let submitted_on: String = document
            .submitted_on
            .expect("validated by ValidatorSessionRequestDocument");
        SessionRequest {
            text: document.text.expect("validated by ValidatorSessionRequestDocument"),
            submitted_on: DateTime::parse_from_rfc3339(&submitted_on)
                .expect("validated by ValidatorSessionRequestDocument")
                .with_timezone(&Utc),
        }
    }
}
