// The app's locale and time zone. A page is rendered on the server and again in the browser, and the two must render
// the same text (or React throws the server's away), so a date is never formatted in whatever locale or time zone the
// machine has.
const LOCALE: string = "en-US";
const TIME_ZONE: string = "UTC";

// Formats what a response model carries for the person to read, in the browser and on the server alike.
export class DateTimeFormatter {
  /** A moment (ISO 8601 text, as a response model carries it), e.g. "Mar 4, 2026, 2:05 PM". */
  public static format(isoText: string): string {
    return new Intl.DateTimeFormat(LOCALE, { dateStyle: "medium", timeStyle: "short", timeZone: TIME_ZONE }).format(
      new Date(isoText),
    );
  }
}
