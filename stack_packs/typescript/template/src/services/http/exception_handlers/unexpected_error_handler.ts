import type { Context } from "hono";

import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { ApplicationLogger } from "#src/managers/logger/application_logger.ts";
import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import type { ErrorResponse } from "#src/services/http/wire_models/error_response.ts";

const HTTP_INTERNAL_SERVER_ERROR = 500 as const;

// Logs, with its stack and the request it failed, any error a request threw that isn't the app's own (those are
// ApplicationExceptionHandler's), and answers it generically so nothing of the app's internals leaks. It is logged
// once, here, where it is caught.
export class UnexpectedErrorHandler {
  public static handle(error: Error, context: Context<HttpEnvironment>): Response {
    ApplicationLogger.error(
      "A request failed unexpectedly",
      new ContextualData().withValue("method", context.req.method).withValue("path", context.req.path),
      error,
    );
    const errorResponse: ErrorResponse = { error: "The request could not be completed" };
    return context.json(errorResponse, HTTP_INTERNAL_SERVER_ERROR);
  }
}
