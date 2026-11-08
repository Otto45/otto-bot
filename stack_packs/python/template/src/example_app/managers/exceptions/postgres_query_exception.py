from typing import final

from example_app.managers.exceptions.relational_database_gateway_exception import RelationalDatabaseGatewayException


@final
class PostgresQueryException(RelationalDatabaseGatewayException):
    """A statement sent to Postgres failed, or Postgres could not be reached."""
