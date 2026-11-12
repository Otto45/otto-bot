import { z } from "zod";

// A row as Postgres returns it: every column optional (missing or null), so that any row is read and the validator can
// say what is wrong with it.
export const CustomerRowSchema = z.object({
  customer_id: z.string().nullish(),
  full_name: z.string().nullish(),
  email_address: z.string().nullish(),
});

export type CustomerRow = z.infer<typeof CustomerRowSchema>;
