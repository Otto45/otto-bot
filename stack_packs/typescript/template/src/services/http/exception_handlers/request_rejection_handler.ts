import type { Context } from "hono";
import type { HTTPException } from "hono/http-exception";

import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import type { ErrorResponse } from "#src/services/http/wire_models/error_response.ts";

// What Hono itself refused before a route ran (e.g. a body that isn't JSON), answered in the ErrorResponse every other
// error is answered with.
export class RequestRejectionHandler {
  public static handle(rejection: HTTPException, context: Context<HttpEnvironment>): Response {
    const errorResponse: ErrorResponse = { error: rejection.message };
    return context.json(errorResponse, rejection.status);
  }
}
