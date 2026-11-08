from pydantic import BaseModel

from example_app.services.http.wire_models.audit_event_response import AuditEventResponse


class AuditEventListResponse(BaseModel):
    audit_events: list[AuditEventResponse]
