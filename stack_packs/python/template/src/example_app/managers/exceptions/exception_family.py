from example_app.managers.exceptions.configuration_provider_exception import ConfigurationProviderException
from example_app.managers.exceptions.document_database_gateway_exception import DocumentDatabaseGatewayException
from example_app.managers.exceptions.invalid_customer_exception import InvalidCustomerException
from example_app.managers.exceptions.json_placeholder_gateway_exception import JsonPlaceholderGatewayException
from example_app.managers.exceptions.relational_database_gateway_exception import RelationalDatabaseGatewayException

# Every family of the application's exceptions. A match that decides something for every exception (e.g.
# HttpErrorMapper.to_status_code) matches ApplicationBaseException.get_family() and names each family here, with no
# `case _:`, so pyright (reportMatchNotExhaustive) fails it until a new family is decided there.
type ExceptionFamily = (
    ConfigurationProviderException
    | RelationalDatabaseGatewayException
    | DocumentDatabaseGatewayException
    | JsonPlaceholderGatewayException
    | InvalidCustomerException
)
