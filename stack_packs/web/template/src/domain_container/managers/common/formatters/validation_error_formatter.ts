import type { z } from "zod";

// Turns the error that deserializing something from outside the process failed with (an HTTP body, a database row, a
// document, all read through a zod schema) into one string, naming each problem by where in the input it is. zod
// reports every problem at once; one about the input as a whole has no location, so it is named after rootName (e.g.
// "body", "row", "document"), as is an error that isn't zod's (e.g. a body that isn't JSON). The input's values are
// left out: they can be anything.
export class ValidationErrorFormatter {
  public static format(error: z.ZodError, rootName: string): string {
    return error.issues
      .map((issue: z.core.$ZodIssue): string => {
        const location: string = issue.path.map((part: PropertyKey): string => String(part)).join(".");
        return `${location === "" ? rootName : location}: ${issue.message}`;
      })
      .join("; ");
  }

  public static formatWhole(error: Error, rootName: string): string {
    return `${rootName}: ${error.message}`;
  }
}
