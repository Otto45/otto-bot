use crate::managers::data_layer::managers::gateways::document_database::resource_models::audit_event_document::AuditEventDocument;
use crate::managers::models::audit_event::AuditEvent;

pub struct AuditEventMapper;

impl AuditEventMapper {
    /// Expects a document that has already passed ValidatorAuditEventDocument: each `expect` states a checked fact,
    /// and fails only if the validator is wrong.
    pub fn to_audit_event(audit_event_document: AuditEventDocument) -> AuditEvent {
        AuditEvent {
            id: audit_event_document
                .event_id
                .expect("validated by ValidatorAuditEventDocument"),
            customer_id: audit_event_document
                .customer_id
                .expect("validated by ValidatorAuditEventDocument"),
            description: audit_event_document
                .description
                .expect("validated by ValidatorAuditEventDocument"),
            occurred_on: audit_event_document
                .occurred_on
                .expect("validated by ValidatorAuditEventDocument"),
        }
    }
}
