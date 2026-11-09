import type { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import type { LogLevel } from "#src/managers/logger/log_level.ts";
import type { LogSource } from "#src/managers/logger/log_source.ts";

// One entry, handed to every handler.
export interface LogEntry {
  readonly level: LogLevel;
  readonly message: string;
  readonly context: ContextualData;
  readonly occurredOn: Date;
  readonly source: LogSource | undefined;
  readonly exception: Error | undefined;
}
