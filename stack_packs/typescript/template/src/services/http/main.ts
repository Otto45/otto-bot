import { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { ApplicationLogger } from "#src/managers/logger/application_logger.ts";
import { LogLevel } from "#src/managers/logger/log_level.ts";
import { UncaughtExceptionLogger } from "#src/managers/logger/uncaught_exception_logger.ts";
import { HttpService } from "#src/services/http/http_service.ts";

// Where the service listens unless HOST and PORT say otherwise, as a deployment does. Kept here, outside the domain
// container, because the ConfigurationProvider serves the application behind the facade, not the processes that host
// it.
const HOST: string = "127.0.0.1";
const PORT: number = 8000;

// Entry point (example-app-http): logger, uncaught errors, HOST and PORT, serve. Hono and the libraries under it log
// nothing of their own, so there is nothing to bridge to the ApplicationLogger.
async function main(): Promise<number> {
  ApplicationLogger.configure(LogLevel.Info);
  UncaughtExceptionLogger.install();
  try {
    return await run();
  } finally {
    ApplicationLogger.close();
  }
}

async function run(): Promise<number> {
  const host: string = process.env["HOST"] ?? HOST;
  const portText: string = process.env["PORT"] ?? String(PORT);
  const port: number = Number(portText);
  if (!Number.isInteger(port) || port < 0 || port > 65_535) {
    ApplicationLogger.error("PORT is not a port number", new ContextualData().withValue("port", portText));
    return 1;
  }

  try {
    await HttpService.serve(host, port);
    return 0;
  } catch (error: unknown) {
    if (error instanceof ApplicationBaseException) {
      ApplicationLogger.logException(error, LogLevel.Error);
    } else {
      ApplicationLogger.error(
        "The service stopped on an unexpected error",
        undefined,
        error instanceof Error ? error : new Error(String(error)),
      );
    }
    return 1;
  }
}

process.exitCode = await main();
