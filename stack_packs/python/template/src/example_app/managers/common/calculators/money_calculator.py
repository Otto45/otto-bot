import json
from decimal import ROUND_HALF_EVEN, Context, Decimal, InvalidOperation
from typing import Final, final

# Wide enough that multiplying an amount by a rate is exact before the one rounding to minor units; the default
# context's 28 digits would round a large amount times a long rate halfway through.
_EXACT: Final[Context] = Context(prec=100, rounding=ROUND_HALF_EVEN)
_ONE: Final[Decimal] = Decimal(1)
_ONE_HUNDRED: Final[Decimal] = Decimal(100)


@final
class MoneyCalculator:
    """Money, kept exact: an amount is an int number of its currency's minor units (cents), and what it is multiplied
    by (an exchange rate, a percentage) is a Decimal, never a float. A result is rounded once, at the end, to minor
    units, half to even (banker's rounding: half a cent goes to the even cent), so no rounding is biased one way."""

    @staticmethod
    def multiply(amount_minor_units: int, factor: Decimal) -> int:
        """The amount, in minor units, times the factor (e.g. an exchange rate), rounded to minor units."""
        product: Decimal = _EXACT.multiply(Decimal(amount_minor_units), factor)
        return int(product.quantize(_ONE, rounding=ROUND_HALF_EVEN, context=_EXACT))

    @staticmethod
    def take_percentage(amount_minor_units: int, percent: Decimal) -> int:
        """percent percent of the amount (e.g. a 10% discount: Decimal(10)), in minor units, rounded to minor units."""
        return MoneyCalculator.multiply(amount_minor_units, _EXACT.divide(percent, _ONE_HUNDRED))

    @staticmethod
    def parse_decimal(text: str) -> Decimal | None:
        """A decimal written as text (a rate from a form or a header); None if the text isn't one."""
        try:
            parsed: Decimal = Decimal(text.strip())
        except InvalidOperation:
            return None
        return parsed if parsed.is_finite() else None

    @staticmethod
    def parse_json(body: str | bytes) -> object:
        """A JSON body with every fractional number read as a Decimal from its text, for a resource model's
        model_validate: model_validate_json would read it as a float first and lose digits. Raises ValueError (as
        json.JSONDecodeError) for a body that isn't JSON."""
        return json.loads(body, parse_float=Decimal)
