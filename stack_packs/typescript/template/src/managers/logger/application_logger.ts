import { fileURLToPath } from "node:url";

import type { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { ConsoleLogHandler } from "#src/managers/logger/handlers/console_log_handler.ts";
import type { LogHandlerProtocol } from "#src/managers/logger/handlers/log_handler_protocol.ts";
import type { LogEntry } from "#src/managers/logger/log_entry.ts";
import { LogLevel } from "#src/managers/logger/log_level.ts";
import type { LogSource } from "#src/managers/logger/log_source.ts";

// A stack frame as V8 writes it: "    at Function.name (file:///path/file.ts:12:34)", or with no name.
const STACK_FRAME_PATTERN: RegExp =
  /^\s*at (?:(?<functionName>.+?) \()?(?<location>.+):(?<line>\d+):(?<column>\d+)\)?$/;
// The frames above the code that logged: the Error, getCallerSource, log and the public method.
const CALLER_FRAME_INDEX: number = 4;

// Exception to the architecture: ApplicationLogger is a global singleton that owns its own handlers, so any code can
// import it and log (e.g. ApplicationLogger.info(...)) without going through the ServiceLocator. It configures itself
// with defaults on first use.
export class ApplicationLogger {
  private static level: LogLevel = LogLevel.Info;
  private static handlers: LogHandlerProtocol[] | undefined = undefined;

  /** Called only at app boot (e.g. in main()). Without handlers (e.g. local development), logs go to the console. */
  public static configure(level: LogLevel = LogLevel.Info, handlers?: LogHandlerProtocol[]): void {
    ApplicationLogger.close();
    ApplicationLogger.level = level;
    ApplicationLogger.handlers = handlers ?? [new ConsoleLogHandler()];
  }

  public static close(): void {
    for (const handler of ApplicationLogger.handlers ?? []) {
      handler.close();
    }
    ApplicationLogger.handlers = undefined;
  }

  public static debug(message: string, context?: ContextualData): void {
    ApplicationLogger.log(LogLevel.Debug, message, context, undefined);
  }

  public static info(message: string, context?: ContextualData): void {
    ApplicationLogger.log(LogLevel.Info, message, context, undefined);
  }

  public static warning(message: string, context?: ContextualData, exception?: Error): void {
    ApplicationLogger.log(LogLevel.Warning, message, context, exception);
  }

  /**
   * Logs what failed, with the exception that made it fail when there is one: an unexpected exception is logged this
   * way, its message saying what was being done (e.g. "A request failed unexpectedly").
   */
  public static error(message: string, context?: ContextualData, exception?: Error): void {
    ApplicationLogger.log(LogLevel.Error, message, context, exception);
  }

  /**
   * Logs one of the application's own exceptions, with its message and context. A client's mistake (a value that is
   * not valid, something not found) is a WARNING; ERROR is kept for what an engineer must look at.
   */
  public static logException(exception: ApplicationBaseException, level: LogLevel): void {
    ApplicationLogger.log(level, exception.message, exception.context, exception);
  }

  private static log(
    level: LogLevel,
    message: string,
    context: ContextualData | undefined,
    exception: Error | undefined,
  ): void {
    if (level < ApplicationLogger.level) {
      return;
    }
    if (ApplicationLogger.handlers === undefined) {
      ApplicationLogger.configure();
    }

    const logEntry: LogEntry = {
      level,
      message,
      context: context ?? new ContextualData(),
      occurredOn: new Date(),
      source: ApplicationLogger.getCallerSource(),
      exception,
    };
    for (const handler of ApplicationLogger.handlers ?? []) {
      handler.handle(logEntry);
    }
  }

  private static getCallerSource(): LogSource | undefined {
    const frame: string | undefined = new Error().stack?.split("\n")[CALLER_FRAME_INDEX];
    const groups: Record<string, string> | undefined =
      frame === undefined ? undefined : STACK_FRAME_PATTERN.exec(frame)?.groups;
    if (groups === undefined) {
      return undefined;
    }
    const location: string = groups["location"] ?? "";
    return {
      filePath: location.startsWith("file://") ? fileURLToPath(location) : location,
      lineNumber: Number(groups["line"]),
      columnNumber: Number(groups["column"]),
      functionName: groups["functionName"] ?? "",
    };
  }
}
