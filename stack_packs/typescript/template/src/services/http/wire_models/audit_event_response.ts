import { z } from "@hono/zod-openapi";

export const AuditEventResponseSchema = z
  .object({
    id: z.string(),
    customer_id: z.string(),
    description: z.string(),
    occurred_on: z.iso.datetime(),
  })
  .openapi("AuditEventResponse");

export type AuditEventResponse = z.infer<typeof AuditEventResponseSchema>;
