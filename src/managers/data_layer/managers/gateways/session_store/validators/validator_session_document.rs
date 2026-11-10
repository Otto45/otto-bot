use chrono::DateTime;

use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_document::SessionDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::models::service_tier::ServiceTier;

pub struct ValidatorSessionDocument;

impl ValidatorSessionDocument {
    /// The session.json of the session folder named `session_id`, which is the session's id.
    pub fn validate_session_document(document: &SessionDocument, session_id: &str) -> ApplicationResult<()> {
        let checks: [(&str, bool); 4] = [
            (
                "id (the session folder's name)",
                document.id.as_deref() == Some(session_id),
            ),
            (
                "workspace_directory",
                document
                    .workspace_directory
                    .as_deref()
                    .is_some_and(|workspace_directory: &str| !workspace_directory.is_empty()),
            ),
            (
                "service_tier (one of standard, fast, ultrafast)",
                document
                    .service_tier
                    .as_deref()
                    .and_then(ServiceTier::from_name)
                    .is_some(),
            ),
            (
                "started_on (an RFC 3339 time)",
                document
                    .started_on
                    .as_deref()
                    .is_some_and(|started_on: &str| DateTime::parse_from_rfc3339(started_on).is_ok()),
            ),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(property_name, _): (&str, bool)| format!("{property_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                SessionStoreGatewayException::InvalidSessionDocument,
                "A session in the session store is invalid",
                ContextualData::new()
                    .with("resource_name", "session")
                    .with("session_id", session_id)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
