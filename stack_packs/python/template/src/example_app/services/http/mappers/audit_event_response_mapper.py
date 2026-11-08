from typing import final

from example_app.managers.models.audit_event import AuditEvent
from example_app.services.http.wire_models.audit_event_list_response import AuditEventListResponse
from example_app.services.http.wire_models.audit_event_response import AuditEventResponse


@final
class AuditEventResponseMapper:
    @staticmethod
    def to_audit_event_list_response(audit_events: list[AuditEvent]) -> AuditEventListResponse:
        return AuditEventListResponse(
            audit_events=[
                AuditEventResponseMapper._to_audit_event_response(audit_event) for audit_event in audit_events
            ]
        )

    @staticmethod
    def _to_audit_event_response(audit_event: AuditEvent) -> AuditEventResponse:
        return AuditEventResponse(
            id=audit_event.id,
            customer_id=audit_event.customer_id,
            description=audit_event.description,
            occurred_on=audit_event.occurred_on,
        )
