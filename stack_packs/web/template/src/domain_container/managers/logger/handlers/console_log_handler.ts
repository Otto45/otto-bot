import { inspect } from "node:util";

import { ApplicationBaseException } from "#src/domain_container/managers/exceptions/application_base_exception.ts";
import type { ContextValue } from "#src/domain_container/managers/exceptions/context_value.ts";
import type { LogHandlerProtocol } from "#src/domain_container/managers/logger/handlers/log_handler_protocol.ts";
import type { LogEntry } from "#src/domain_container/managers/logger/log_entry.ts";
import { LogLevel } from "#src/domain_container/managers/logger/log_level.ts";
import { LogRedactor } from "#src/domain_container/managers/logger/log_redactor.ts";

const LOGGER_NAME: string = "example_app";
// The longest a message and a stack may be: an exception that quotes a whole response or file must not make a line that
// fills the log.
const MAX_MESSAGE_LENGTH: number = 4_000;
const MAX_STACK_LENGTH: number = 16_000;
const TRUNCATED: string = " [truncated]";
const LEVEL_NAMES: Readonly<Record<LogLevel, string>> = {
  [LogLevel.Debug]: "DEBUG",
  [LogLevel.Info]: "INFO",
  [LogLevel.Warning]: "WARNING",
  [LogLevel.Error]: "ERROR",
};

// Writes each log entry to the console (stderr) as one line of JSON, with an exception's stack inside it, so a log store
// (CloudWatch) keeps each entry, stack and all, as one event whose fields it can search and count. What must never be
// logged is taken out first (LogRedactor).
export class ConsoleLogHandler implements LogHandlerProtocol {
  public handle(logEntry: LogEntry): void {
    // One write per line, so lines never interleave. A line that can't be written has nowhere else to go.
    process.stderr.write(JSON.stringify(ConsoleLogHandler.toConsoleLine(logEntry)) + "\n");
  }

  public close(): void {
    // Holds nothing to release: the process owns stderr.
  }

  private static toConsoleLine(logEntry: LogEntry): Record<string, unknown> {
    const context: Record<string, ContextValue> = {};
    for (const [key, value] of Object.entries(logEntry.context.toRecord())) {
      context[key] = typeof value === "string" ? ConsoleLogHandler.clean(value, MAX_MESSAGE_LENGTH) : value;
    }
    const consoleLine: Record<string, unknown> = {
      time: logEntry.occurredOn.toISOString(),
      level: LEVEL_NAMES[logEntry.level],
      logger: LOGGER_NAME,
      message: ConsoleLogHandler.clean(logEntry.message, MAX_MESSAGE_LENGTH),
      context,
    };
    if (logEntry.source !== undefined) {
      consoleLine["source"] = {
        file: logEntry.source.filePath,
        line: logEntry.source.lineNumber,
        column: logEntry.source.columnNumber,
        function: logEntry.source.functionName,
      };
    }
    if (logEntry.exception !== undefined) {
      const error: Record<string, unknown> = {
        type: ConsoleLogHandler.typeName(logEntry.exception),
        message: ConsoleLogHandler.clean(logEntry.exception.message, MAX_MESSAGE_LENGTH),
      };
      // Only what an engineer must look at carries its stack: a client's mistake, a warning, needs none.
      if (logEntry.level >= LogLevel.Error) {
        error["stack"] = ConsoleLogHandler.clean(ConsoleLogHandler.stack(logEntry.exception), MAX_STACK_LENGTH);
      }
      consoleLine["error"] = error;
    }
    return consoleLine;
  }

  private static typeName(exception: Error): string {
    return exception instanceof ApplicationBaseException ? exception.name : exception.constructor.name;
  }

  // The stack where the exception was thrown, then each error it was thrown from (its causes).
  private static stack(exception: Error): string {
    let stack: string = exception.stack ?? `${ConsoleLogHandler.typeName(exception)}: ${exception.message}`;
    let cause: unknown = exception.cause;
    while (cause !== undefined) {
      stack += `\nCaused by: ${cause instanceof Error ? (cause.stack ?? cause.message) : inspect(cause)}`;
      cause = cause instanceof Error ? cause.cause : undefined;
    }
    return stack;
  }

  private static clean(text: string, maxLength: number): string {
    const redacted: string = LogRedactor.redact(text);
    return redacted.length <= maxLength ? redacted : redacted.slice(0, maxLength) + TRUNCATED;
  }
}
