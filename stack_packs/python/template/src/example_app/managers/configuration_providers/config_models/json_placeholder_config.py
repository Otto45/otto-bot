from dataclasses import dataclass


@dataclass(slots=True)
class JsonPlaceholderConfig:
    base_url: str | None = None
    timeout_seconds: float | None = None
