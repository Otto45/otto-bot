use crate::managers::models::audit_event::AuditEvent;
use crate::services::http::wire_models::audit_event_response::{AuditEventListResponse, AuditEventResponse};

pub struct AuditEventResponseMapper;

impl AuditEventResponseMapper {
    pub fn to_audit_event_list_response(audit_events: Vec<AuditEvent>) -> AuditEventListResponse {
        AuditEventListResponse {
            audit_events: audit_events
                .into_iter()
                .map(AuditEventResponseMapper::to_audit_event_response)
                .collect(),
        }
    }

    fn to_audit_event_response(audit_event: AuditEvent) -> AuditEventResponse {
        AuditEventResponse {
            id: audit_event.id,
            customer_id: audit_event.customer_id,
            description: audit_event.description,
            occurred_on: audit_event.occurred_on,
        }
    }
}
