from typing import final

from example_app.managers.exceptions.relational_database_gateway_exception import RelationalDatabaseGatewayException


@final
class InvalidPostgresCustomerRowException(RelationalDatabaseGatewayException):
    """A customers row doesn't have what a Customer needs."""
