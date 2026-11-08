from dataclasses import dataclass


@dataclass(slots=True)
class RelationalDatabaseConfig:
    connection_string: str | None = None
