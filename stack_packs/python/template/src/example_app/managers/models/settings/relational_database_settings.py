from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RelationalDatabaseSettings:
    connection_string: str
