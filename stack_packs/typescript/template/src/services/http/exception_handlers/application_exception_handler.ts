import type { Context } from "hono";
import type { ContentfulStatusCode } from "hono/utils/http-status";

import type { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import { ApplicationLogger } from "#src/managers/logger/application_logger.ts";
import { LogLevel } from "#src/managers/logger/log_level.ts";
import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import { HttpErrorMapper } from "#src/services/http/mappers/http_error_mapper.ts";
import type { ErrorResponse } from "#src/services/http/wire_models/error_response.ts";

// What a route answers when the facade throws: Hono's idiom is its error handler (onError), so a route never catches an
// exception itself.
export class ApplicationExceptionHandler {
  public static handle(exception: ApplicationBaseException, context: Context<HttpEnvironment>): Response {
    const statusCode: ContentfulStatusCode = HttpErrorMapper.toStatusCode(exception);

    // A client's mistake is a warning, easy to find on its own; ERROR is kept for what an engineer must look at.
    ApplicationLogger.logException(exception, statusCode >= 500 ? LogLevel.Error : LogLevel.Warning);

    const errorResponse: ErrorResponse = HttpErrorMapper.toErrorResponse(exception, statusCode);
    return context.json(errorResponse, statusCode);
  }
}
