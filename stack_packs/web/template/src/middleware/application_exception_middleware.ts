import { createMiddleware } from "@tanstack/react-start";
import { isNotFound, isRedirect, notFound } from "@tanstack/react-router";

import { ApplicationBaseException } from "#src/domain_container/managers/exceptions/application_base_exception.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { ApplicationLogger } from "#src/domain_container/managers/logger/application_logger.ts";
import { LogLevel } from "#src/domain_container/managers/logger/log_level.ts";
import { ServerFunctionErrorMapper } from "#src/mappers/server_function_error_mapper.ts";
import { ServerFunctionFailure } from "#src/mappers/server_function_failure.ts";

// Every server function's exception handling (registered for all of them in src/start.ts), so a server function never
// catches one itself: the exception is logged once, here, and the browser is told only what the person may see.
export const applicationExceptionMiddleware = createMiddleware({ type: "function" }).server(async ({ next }) => {
  try {
    return await next();
  } catch (error: unknown) {
    // A server function's own redirect() or notFound() is the router's to follow, as it is.
    if (isRedirect(error) || isNotFound(error)) {
      throw error;
    }
    if (error instanceof ApplicationBaseException) {
      const failure: ServerFunctionFailure = ServerFunctionErrorMapper.toFailure(error);
      // A person's mistake is a warning, easy to find on its own; ERROR is kept for what an engineer must look at.
      const isClientError: boolean =
        failure === ServerFunctionFailure.Invalid || failure === ServerFunctionFailure.NotFound;
      ApplicationLogger.logException(error, isClientError ? LogLevel.Warning : LogLevel.Error);
      if (failure === ServerFunctionFailure.NotFound) {
        throw notFound();
      }
      // The browser is sent the message only: the cause stays on the server.
      throw new Error(ServerFunctionErrorMapper.toBrowserMessage(error, failure), { cause: error });
    }
    ApplicationLogger.error(
      "A server function failed on an unexpected error",
      new ContextualData(),
      error instanceof Error ? error : new Error(String(error)),
    );
    throw new Error(ServerFunctionErrorMapper.toUnexpectedBrowserMessage(), { cause: error });
  }
});
