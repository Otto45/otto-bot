import { z } from "zod";

// A document as the database stores it: every field optional (missing or null), so that any document is read and the
// validator can say what is wrong with it.
export const AuditEventDocumentSchema = z.object({
  event_id: z.string().nullish(),
  customer_id: z.string().nullish(),
  description: z.string().nullish(),
  occurred_on: z.iso.datetime({ offset: true }).nullish(),
});

export type AuditEventDocument = z.infer<typeof AuditEventDocumentSchema>;
