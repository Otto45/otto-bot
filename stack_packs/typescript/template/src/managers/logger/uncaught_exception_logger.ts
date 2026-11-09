import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { ApplicationLogger } from "#src/managers/logger/application_logger.ts";

// Logs an error nothing caught, as every other line is logged: one that would end the process (an uncaught exception),
// and a promise rejected with nothing to handle it. Every entry point installs it, so nothing that can fail runs
// without something to catch it.
export class UncaughtExceptionLogger {
  public static install(): void {
    process.on("uncaughtException", (error: Error): void => {
      ApplicationLogger.error("The app stopped on an unexpected error", new ContextualData(), error);
      ApplicationLogger.close();
      // What Node does after an uncaught exception, once it is logged.
      process.exit(1);
    });
    // A rejection nothing handled is a bug where it was created, but nothing else is wrong: it is logged and the app
    // keeps going, as an asyncio task's is.
    process.on("unhandledRejection", (reason: unknown): void => {
      ApplicationLogger.error(
        "A promise was rejected with nothing to handle it",
        new ContextualData(),
        reason instanceof Error ? reason : new Error(String(reason)),
      );
    });
  }
}
