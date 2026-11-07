use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::data_layer::managers::gateways::document_database::resource_models::audit_event_document::AuditEventDocument;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::document_database_gateway_exception::DocumentDatabaseGatewayException;

pub struct ValidatorAuditEventDocument;

impl ValidatorAuditEventDocument {
    pub fn validate_audit_event_document(audit_event_document: &AuditEventDocument) -> ApplicationResult<()> {
        let mut errors: Vec<String> = [
            ("event_id", audit_event_document.event_id.as_deref()),
            ("customer_id", audit_event_document.customer_id.as_deref()),
            ("description", audit_event_document.description.as_deref()),
        ]
        .into_iter()
        .filter_map(|(field_name, field_value): (&str, Option<&str>)| {
            ValidatorString::validate(field_name, field_value)
        })
        .collect();

        if audit_event_document.occurred_on.is_none() {
            errors.push("occurred_on is missing".to_string());
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                DocumentDatabaseGatewayException::InvalidAuditEventDocument,
                "Audit event document from the document database audit_events collection failed validation",
                ContextualData::new()
                    .with("resource_name", "audit_event")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
