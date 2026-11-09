import { z } from "@hono/zod-openapi";

export const UserProfileResponseSchema = z
  .object({
    id: z.int(),
    name: z.string(),
    username: z.string(),
    email: z.string(),
    city: z.string(),
    company_name: z.string(),
  })
  .openapi("UserProfileResponse");

export type UserProfileResponse = z.infer<typeof UserProfileResponseSchema>;
