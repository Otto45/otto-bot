from typing import final

from example_app.managers.exceptions.json_placeholder_gateway_exception import JsonPlaceholderGatewayException


@final
class JsonPlaceholderRequestException(JsonPlaceholderGatewayException):
    """A request to JSON Placeholder failed: it could not be sent, timed out, or was answered with an error status."""
