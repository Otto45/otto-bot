import { Decimal } from "decimal.js";

// Wide enough that multiplying an amount by a rate is exact before the one rounding to minor units; decimal.js rounds
// every operation to 20 significant digits by default, which would round a large amount times a long rate halfway.
const EXACT: typeof Decimal = Decimal.clone({ precision: 100, rounding: Decimal.ROUND_HALF_EVEN });

// Money, kept exact: an amount is an integer number of its currency's minor units (cents), a safe integer, and what it
// is multiplied by (an exchange rate, a percentage) is a decimal.js Decimal, never a number. A result is rounded once,
// at the end, to minor units, half to even (banker's rounding: half a cent goes to the even cent), so no rounding is
// biased one way.
export class MoneyCalculator {
  /**
   * The amount, in minor units, times the factor (e.g. an exchange rate), rounded to minor units. A result beyond a
   * safe integer (an amount near 90 trillion dollars, which validation rules out first) is a bug: a RangeError.
   */
  public static multiply(amountMinorUnits: number, factor: Decimal): number {
    const rounded: number = new EXACT(amountMinorUnits)
      .times(factor)
      .toDecimalPlaces(0, Decimal.ROUND_HALF_EVEN)
      .toNumber();
    if (!Number.isSafeInteger(rounded)) {
      throw new RangeError("an amount in minor units is beyond a safe integer");
    }
    return rounded;
  }

  /** percent percent of the amount (e.g. a 10% discount: 10), in minor units, rounded to minor units. */
  public static takePercentage(amountMinorUnits: number, percent: Decimal): number {
    return MoneyCalculator.multiply(amountMinorUnits, new EXACT(percent).dividedBy(100));
  }

  /** A decimal written as text (a rate from a form, a header, or a numeric column, which pg reads as text); undefined if the text isn't one. */
  public static parseDecimal(text: string): Decimal | undefined {
    try {
      const parsed: Decimal = new Decimal(text.trim());
      return parsed.isFinite() ? parsed : undefined;
    } catch {
      return undefined;
    }
  }

  /**
   * A JSON body with every number that isn't a safe integer read as a Decimal from its text, so its resource model's
   * schema takes a decimal as z.instanceof(Decimal): JSON.parse alone reads it as a number and loses digits. Throws a
   * SyntaxError for a body that isn't JSON.
   */
  public static parseJson(body: string): unknown {
    return JSON.parse(body, (_key: string, value: unknown, context?: { source?: string }): unknown =>
      typeof value === "number" && !Number.isSafeInteger(value) && context?.source !== undefined
        ? new Decimal(context.source)
        : value,
    );
  }
}
