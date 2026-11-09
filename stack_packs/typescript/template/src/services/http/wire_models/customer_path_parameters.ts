import { z } from "@hono/zod-openapi";

export const CustomerPathParametersSchema = z.object({
  customer_id: z.string().openapi({ param: { name: "customer_id", in: "path", required: true } }),
});

export type CustomerPathParameters = z.infer<typeof CustomerPathParametersSchema>;
