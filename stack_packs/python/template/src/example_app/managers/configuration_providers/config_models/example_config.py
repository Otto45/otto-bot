from dataclasses import dataclass


# A config model: what a configuration source read, before it is validated. Every field can be missing, so the
# validator can say what is wrong with it.
@dataclass(slots=True)
class ExampleConfig:
    greeting_prefix: str | None = None
