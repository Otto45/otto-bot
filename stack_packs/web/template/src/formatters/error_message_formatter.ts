import { isNotFound } from "@tanstack/react-router";

const NOT_FOUND_MESSAGE: string = "We couldn't find that.";
const FAILED_MESSAGE: string = "Something went wrong. Please try again.";

// What a page tells the person when a server function it called fails: the message the server function was told to
// give (src/mappers/server_function_error_mapper.ts), which is written for them.
export class ErrorMessageFormatter {
  public static format(error: unknown): string {
    if (isNotFound(error)) {
      return NOT_FOUND_MESSAGE;
    }
    return error instanceof Error ? error.message : FAILED_MESSAGE;
  }
}
