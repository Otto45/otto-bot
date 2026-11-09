import type { ContentfulStatusCode } from "hono/utils/http-status";

import type { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import { CustomerNotFoundException } from "#src/managers/exceptions/customer_not_found_exception.ts";
import type { ExceptionFamily } from "#src/managers/exceptions/exception_family.ts";
import { UserProfileNotFoundException } from "#src/managers/exceptions/user_profile_not_found_exception.ts";
import type { ErrorResponse } from "#src/services/http/wire_models/error_response.ts";

const HTTP_BAD_REQUEST: ContentfulStatusCode = 400;
const HTTP_NOT_FOUND: ContentfulStatusCode = 404;
const HTTP_INTERNAL_SERVER_ERROR: ContentfulStatusCode = 500;
const HTTP_BAD_GATEWAY: ContentfulStatusCode = 502;

// Translates the domain container's exceptions into HTTP errors. Malformed requests never get here: Hono answers those
// itself (see RequestRejectionMapper).
export class HttpErrorMapper {
  public static toStatusCode(exception: ApplicationBaseException): ContentfulStatusCode {
    const family: ExceptionFamily = exception.getFamily();
    // Every family is named, with no `default`, so a new one doesn't compile until it is given a status here.
    switch (family.family) {
      case "InvalidCustomer":
        return HTTP_BAD_REQUEST;
      case "RelationalDatabaseGateway":
        return family instanceof CustomerNotFoundException ? HTTP_NOT_FOUND : HTTP_INTERNAL_SERVER_ERROR;
      case "JsonPlaceholderGateway":
        return family instanceof UserProfileNotFoundException ? HTTP_NOT_FOUND : HTTP_BAD_GATEWAY;
      case "ConfigurationProvider":
      case "DocumentDatabaseGateway":
        return HTTP_INTERNAL_SERVER_ERROR;
    }
  }

  public static toErrorResponse(exception: ApplicationBaseException, statusCode: ContentfulStatusCode): ErrorResponse {
    // A client error answers with the exception's own message: those exceptions are the ones listed above as 4xx, and
    // their messages are written for the person using the app. Every other message and every context describes
    // internals (tables, SQL statements, upstream services) that callers must not see; they are logged instead.
    if (statusCode < HTTP_INTERNAL_SERVER_ERROR) {
      return { error: exception.message };
    }
    if (statusCode === HTTP_BAD_GATEWAY) {
      return { error: "An upstream service failed" };
    }
    return { error: "The request could not be completed" };
  }
}
