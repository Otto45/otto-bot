import type { AuditEvent } from "#src/managers/models/audit_event.ts";
import type { AuditEventListResponse } from "#src/services/http/wire_models/audit_event_list_response.ts";
import type { AuditEventResponse } from "#src/services/http/wire_models/audit_event_response.ts";

export class AuditEventResponseMapper {
  public static toAuditEventListResponse(auditEvents: readonly AuditEvent[]): AuditEventListResponse {
    return {
      audit_events: auditEvents.map((auditEvent: AuditEvent): AuditEventResponse =>
        AuditEventResponseMapper.toAuditEventResponse(auditEvent),
      ),
    };
  }

  private static toAuditEventResponse(auditEvent: AuditEvent): AuditEventResponse {
    return {
      id: auditEvent.id,
      customer_id: auditEvent.customerId,
      description: auditEvent.description,
      occurred_on: auditEvent.occurredOn.toISOString(),
    };
  }
}
