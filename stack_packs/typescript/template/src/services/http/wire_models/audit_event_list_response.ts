import { z } from "@hono/zod-openapi";

import { AuditEventResponseSchema } from "#src/services/http/wire_models/audit_event_response.ts";

export const AuditEventListResponseSchema = z
  .object({ audit_events: z.array(AuditEventResponseSchema) })
  .openapi("AuditEventListResponse");

export type AuditEventListResponse = z.infer<typeof AuditEventListResponseSchema>;
