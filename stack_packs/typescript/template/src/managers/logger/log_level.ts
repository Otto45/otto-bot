// How much a log entry matters; entries below the configured level are dropped. A const object and its values' type,
// as TypeScript's enums aren't plain JavaScript (Node runs the code with its types stripped).
export const LogLevel = { Debug: 10, Info: 20, Warning: 30, Error: 40 } as const;

export type LogLevel = (typeof LogLevel)[keyof typeof LogLevel];
