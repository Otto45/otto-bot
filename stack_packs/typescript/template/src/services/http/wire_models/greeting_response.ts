import { z } from "@hono/zod-openapi";

export const GreetingResponseSchema = z.object({ message: z.string() }).openapi("GreetingResponse");

export type GreetingResponse = z.infer<typeof GreetingResponseSchema>;
