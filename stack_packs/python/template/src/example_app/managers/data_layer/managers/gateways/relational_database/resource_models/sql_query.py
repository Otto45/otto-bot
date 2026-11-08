from dataclasses import dataclass
from typing import LiteralString

from example_app.managers.data_layer.managers.gateways.relational_database.resource_models.sql_parameter import (
    SqlParameter,
)


# Values travel as bound parameters, never formatted into the statement. The statement is a LiteralString, so pyright
# refuses one built from a value at run time (an f-string or a concatenation with a variable): only a statement written
# in the code type-checks.
@dataclass(frozen=True, slots=True)
class SqlQuery:
    statement: LiteralString
    parameters: tuple[SqlParameter, ...]
