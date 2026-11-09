import { z } from "@hono/zod-openapi";

// Every error the service answers with, in one shape, so callers handle one shape of error.
export const ErrorResponseSchema = z.object({ error: z.string() }).openapi("ErrorResponse");

export type ErrorResponse = z.infer<typeof ErrorResponseSchema>;
