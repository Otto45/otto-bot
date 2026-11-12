import type { LogEntry } from "#src/domain_container/managers/logger/log_entry.ts";

export interface LogHandlerProtocol {
  handle(logEntry: LogEntry): void;

  close(): void;
}
