import type { z } from "@hono/zod-openapi";

import type { ErrorResponse } from "#src/services/http/wire_models/error_response.ts";

// How Hono's own answer to a request that doesn't fit (a body or a path parameter that doesn't fit its model) goes over
// the wire: in the ErrorResponse every other error is answered with, so that callers handle one shape of error. It says
// where each problem is, never the value sent.
export class RequestRejectionMapper {
  public static toErrorResponse(error: z.ZodError): ErrorResponse {
    return {
      error: error.issues
        .map((issue: z.core.$ZodIssue): string => {
          const location: string = issue.path.map((part: PropertyKey): string => String(part)).join(".");
          return `${location === "" ? "request" : location}: ${issue.message}`;
        })
        .join("; "),
    };
  }
}
