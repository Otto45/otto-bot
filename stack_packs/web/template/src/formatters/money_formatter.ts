const LOCALE: string = "en-US";

// Shows an amount of money exactly: a response model carries it as an integer number of its currency's minor units
// (cents), which is written out as decimal text and formatted from that text, never through a float.
export class MoneyFormatter {
  /** e.g. format(123456, "USD") is "$1,234.56", and format(500, "JPY") is "¥500". */
  public static format(amountMinorUnits: number, currency: string): string {
    const numberFormat: Intl.NumberFormat = new Intl.NumberFormat(LOCALE, { style: "currency", currency });
    const fractionDigits: number = numberFormat.resolvedOptions().maximumFractionDigits ?? 0;
    const digits: string = String(Math.abs(amountMinorUnits)).padStart(fractionDigits + 1, "0");
    const wholePart: string = digits.slice(0, digits.length - fractionDigits);
    const fractionPart: string = digits.slice(digits.length - fractionDigits);
    const sign: string = amountMinorUnits < 0 ? "-" : "";
    // Decimal text by construction (digits, and a point before the minor units), which Intl formats exactly.
    const decimalText: Intl.StringNumericLiteral = (
      fractionDigits === 0 ? `${sign}${wholePart}` : `${sign}${wholePart}.${fractionPart}`
    ) as Intl.StringNumericLiteral;
    return numberFormat.format(decimalText);
  }
}
