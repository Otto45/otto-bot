use std::str::FromStr;

use bigdecimal::{BigDecimal, RoundingMode, ToPrimitive};

/// Money, kept exact: an amount is an integer number of its currency's minor units (cents), and what it is multiplied
/// by (an exchange rate, a percentage) is a `BigDecimal`, never a float. A `BigDecimal` has every digit it is given and
/// multiplies exactly, so a result is rounded once, at the end, to minor units, half to even (banker's rounding: half
/// a cent goes to the even cent), and no rounding is biased one way.
pub struct MoneyCalculator;

impl MoneyCalculator {
    /// The amount, in minor units, times the factor (e.g. an exchange rate), rounded to minor units; None if it doesn't
    /// fit an i64.
    pub fn multiply(amount_minor_units: i64, factor: &BigDecimal) -> Option<i64> {
        (BigDecimal::from(amount_minor_units) * factor)
            .with_scale_round(0, RoundingMode::HalfEven)
            .to_i64()
    }

    /// `percent` percent of the amount (e.g. a 10% discount: 10), in minor units, rounded to minor units; None if it
    /// doesn't fit an i64.
    pub fn take_percentage(amount_minor_units: i64, percent: &BigDecimal) -> Option<i64> {
        // Dividing by 100 only moves the decimal point, so it is exact.
        MoneyCalculator::multiply(amount_minor_units, &(percent / BigDecimal::from(100)))
    }

    /// A decimal written as text, scientific notation included (a rate from a form, a header, or a `numeric` column
    /// read as text); None if the text isn't one. A JSON body's numbers need no parsing: a `BigDecimal` field of its
    /// resource model reads one from its text (serde_json's `arbitrary_precision`).
    pub fn parse_decimal(text: &str) -> Option<BigDecimal> {
        BigDecimal::from_str(text.trim()).ok()
    }
}
