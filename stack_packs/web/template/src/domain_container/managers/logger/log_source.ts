// Where in the code an entry was logged: the code that called the ApplicationLogger.
export interface LogSource {
  readonly filePath: string;
  readonly lineNumber: number;
  readonly columnNumber: number;
  readonly functionName: string;
}
