from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class JsonPlaceholderSettings:
    base_url: str
    timeout_seconds: float
