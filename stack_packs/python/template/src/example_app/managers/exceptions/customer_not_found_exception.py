from typing import final

from example_app.managers.exceptions.relational_database_gateway_exception import RelationalDatabaseGatewayException


@final
class CustomerNotFoundException(RelationalDatabaseGatewayException):
    """No customer has the id asked for: the caller can put it right, so its message is written for the person
    using the app."""
