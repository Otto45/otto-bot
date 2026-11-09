import type { LogEntry } from "#src/managers/logger/log_entry.ts";

export interface LogHandlerProtocol {
  handle(logEntry: LogEntry): void;

  close(): void;
}
