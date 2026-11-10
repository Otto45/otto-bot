use chrono::DateTime;

use crate::managers::data_layer::managers::gateways::session_store::resource_models::session_request_document::SessionRequestDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;

pub struct ValidatorSessionRequestDocument;

impl ValidatorSessionRequestDocument {
    pub fn validate_session_request_document(
        document: &SessionRequestDocument,
        session_id: &str,
        line_number: i64,
    ) -> ApplicationResult<()> {
        let checks: [(&str, bool); 2] = [
            ("text", document.text.is_some()),
            (
                "submitted_on (an RFC 3339 time)",
                document
                    .submitted_on
                    .as_deref()
                    .is_some_and(|submitted_on: &str| DateTime::parse_from_rfc3339(submitted_on).is_ok()),
            ),
        ];
        let errors: Vec<String> = checks
            .into_iter()
            .filter(|(_, valid): &(&str, bool)| !valid)
            .map(|(property_name, _): (&str, bool)| format!("{property_name} is missing or not valid"))
            .collect();
        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                SessionStoreGatewayException::InvalidSessionRequestDocument,
                "A request in the session store is invalid",
                ContextualData::new()
                    .with("resource_name", "session_request")
                    .with("session_id", session_id)
                    .with("line_number", line_number)
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
