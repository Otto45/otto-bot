import { z } from "@hono/zod-openapi";

export const UserProfilePathParametersSchema = z.object({
  user_id: z.coerce
    .number()
    .int()
    .openapi({ param: { name: "user_id", in: "path", required: true }, type: "integer" }),
});

export type UserProfilePathParameters = z.infer<typeof UserProfilePathParametersSchema>;
