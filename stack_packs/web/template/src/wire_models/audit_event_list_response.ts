import type { AuditEventResponse } from "#src/wire_models/audit_event_response.ts";

export interface AuditEventListResponse {
  /** Oldest first. */
  readonly auditEvents: readonly AuditEventResponse[];
}
