import { z } from "zod";

export const CustomerRequestSchema = z.object({ customerId: z.string() });

export type CustomerRequest = z.infer<typeof CustomerRequestSchema>;
