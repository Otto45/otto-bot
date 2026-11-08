from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class DocumentDatabaseSettings:
    connection_string: str
