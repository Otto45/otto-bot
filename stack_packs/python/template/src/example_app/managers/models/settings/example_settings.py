from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ExampleSettings:
    greeting_prefix: str
