import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";
import type { AuditEventListResponse } from "#src/wire_models/audit_event_list_response.ts";
import type { AuditEventResponse } from "#src/wire_models/audit_event_response.ts";

export class AuditEventResponseMapper {
  public static toAuditEventListResponse(auditEvents: readonly AuditEvent[]): AuditEventListResponse {
    return {
      auditEvents: auditEvents.map((auditEvent: AuditEvent): AuditEventResponse =>
        AuditEventResponseMapper.toAuditEventResponse(auditEvent),
      ),
    };
  }

  private static toAuditEventResponse(auditEvent: AuditEvent): AuditEventResponse {
    return {
      id: auditEvent.id,
      description: auditEvent.description,
      occurredOn: auditEvent.occurredOn.toISOString(),
    };
  }
}
