from dataclasses import dataclass


@dataclass(slots=True)
class DocumentDatabaseConfig:
    connection_string: str | None = None
