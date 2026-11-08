from collections.abc import Mapping
from typing import final

from example_app.managers.exceptions.context_value import ContextValue


# What an exception says about where it happened, under snake_case keys, for the engineers who read the logs. It never
# holds anything sensitive: no bound parameter values, no credentials. It never changes once made: with_value returns a
# new one, so a context is built in one expression: `ContextualData().with_value("customer_id", customer_id)`.
@final
class ContextualData:
    __slots__ = ("_values",)

    def __init__(self, values: Mapping[str, ContextValue] | None = None) -> None:
        self._values: dict[str, ContextValue] = dict(values or {})

    def with_value(self, key: str, value: ContextValue) -> ContextualData:
        values: dict[str, ContextValue] = dict(self._values)
        values[key] = value
        return ContextualData(values)

    def get(self, key: str) -> ContextValue | None:
        return self._values.get(key)

    def to_dict(self) -> dict[str, ContextValue]:
        """The values, sorted by key, so the same context is always written the same way."""
        return dict(sorted(self._values.items()))

    def __repr__(self) -> str:
        return f"ContextualData({self.to_dict()!r})"
