import handler, { createServerEntry } from "@tanstack/react-start/server-entry";
import { useNitroHooks } from "nitro/app";

import { ExampleAppFacade } from "#src/domain_container/example_app_facade.ts";
import { ApplicationBaseException } from "#src/domain_container/managers/exceptions/application_base_exception.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { ApplicationLogger } from "#src/domain_container/managers/logger/application_logger.ts";
import { LogLevel } from "#src/domain_container/managers/logger/log_level.ts";
import { UncaughtExceptionLogger } from "#src/domain_container/managers/logger/uncaught_exception_logger.ts";
import type { RequestContext } from "#src/request_context.ts";

// The server's entry point: logger, uncaught errors, the one facade, serve. Nitro listens where HOST and PORT say (a
// deployment sets them), and stops on Ctrl-C and SIGTERM once the requests under way have finished.
ApplicationLogger.configure(LogLevel.Info);
UncaughtExceptionLogger.install();

// Created once, as the server starts (src/nitro_plugins/server_entry_loader.ts loads this entry before any request), and
// lives as long as the server: every request is handed this instance, and nothing builds one per request.
const exampleAppFacade: ExampleAppFacade = createFacade();

// Nitro calls its close hooks once the server has stopped taking requests, so the facade (and everything its locator
// cached) is closed last.
useNitroHooks().hook("close", async (): Promise<void> => {
  await exampleAppFacade.close();
  ApplicationLogger.close();
});

// The request context's type, for both packages that read it: TanStack Start types a server function's and a
// middleware's `context` from the first, and this entry's `handler.fetch` from the second.
declare module "@tanstack/react-start" {
  interface Register {
    server: { requestContext: RequestContext };
  }
}
declare module "@tanstack/react-router" {
  interface Register {
    server: { requestContext: RequestContext };
  }
}

// A facade that can't be created (e.g. invalid configuration) ends the process, logged: a server that can't serve
// anything doesn't stay up answering every request with an error.
function createFacade(): ExampleAppFacade {
  try {
    return new ExampleAppFacade();
  } catch (error: unknown) {
    if (error instanceof ApplicationBaseException) {
      ApplicationLogger.logException(error, LogLevel.Error);
    } else {
      ApplicationLogger.error(
        "The server stopped on an unexpected error",
        new ContextualData(),
        error instanceof Error ? error : new Error(String(error)),
      );
    }
    ApplicationLogger.close();
    process.exit(1);
  }
}

export default createServerEntry({
  fetch: (request, options) => handler.fetch(request, { ...options, context: { exampleAppFacade } }),
});
