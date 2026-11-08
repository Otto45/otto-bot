from http import HTTPStatus
from typing import final

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.exceptions.configuration_provider_exception import ConfigurationProviderException
from example_app.managers.exceptions.customer_not_found_exception import CustomerNotFoundException
from example_app.managers.exceptions.document_database_gateway_exception import DocumentDatabaseGatewayException
from example_app.managers.exceptions.invalid_customer_exception import InvalidCustomerException
from example_app.managers.exceptions.json_placeholder_gateway_exception import JsonPlaceholderGatewayException
from example_app.managers.exceptions.relational_database_gateway_exception import RelationalDatabaseGatewayException
from example_app.managers.exceptions.user_profile_not_found_exception import UserProfileNotFoundException
from example_app.services.http.wire_models.error_response import ErrorResponse


# Translates the domain container's exceptions into HTTP errors. Malformed requests never get here: FastAPI answers
# those itself (see RequestValidationExceptionHandler).
@final
class HttpErrorMapper:
    @staticmethod
    def to_status_code(exception: ApplicationBaseException) -> HTTPStatus:
        # Most specific first: the not-found exceptions are also gateway exceptions. Every family is named, with no
        # `case _:`, so a new one fails the type check until it is given a status here.
        match exception.get_family():
            case InvalidCustomerException():
                return HTTPStatus.BAD_REQUEST
            case CustomerNotFoundException() | UserProfileNotFoundException():
                return HTTPStatus.NOT_FOUND
            case JsonPlaceholderGatewayException():
                return HTTPStatus.BAD_GATEWAY
            case (
                ConfigurationProviderException()
                | RelationalDatabaseGatewayException()
                | DocumentDatabaseGatewayException()
            ):
                return HTTPStatus.INTERNAL_SERVER_ERROR

    @staticmethod
    def to_error_response(exception: ApplicationBaseException, status_code: HTTPStatus) -> ErrorResponse:
        # A client error answers with the exception's own message: those exceptions are the ones listed above as 4xx,
        # and their messages are written for the person using the app. Every other message and every context describes
        # internals (tables, SQL statements, upstream services) that callers must not see; they are logged instead.
        if not status_code.is_server_error:
            return ErrorResponse(error=exception.message)
        if status_code == HTTPStatus.BAD_GATEWAY:
            return ErrorResponse(error="An upstream service failed")
        return ErrorResponse(error="The request could not be completed")
