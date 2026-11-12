import { z } from "zod";

// A request model: the zod schema a server function checks its input against (`.validator`), and the type of what it
// reads. It says what shape a request has (its fields and their types) and no more: whether the values are acceptable
// is the domain container's decision (here CustomerProcessor's), so that a person who leaves the name blank is told the
// domain's own message.
export const SaveCustomerRequestSchema = z.object({ customerId: z.string(), name: z.string(), email: z.string() });

export type SaveCustomerRequest = z.infer<typeof SaveCustomerRequestSchema>;
