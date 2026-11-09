from typing import final

from example_app.managers.exceptions.json_placeholder_gateway_exception import JsonPlaceholderGatewayException


@final
class JsonPlaceholderUnavailableException(JsonPlaceholderGatewayException):
    """JSON Placeholder failed on its side, is limiting requests, or couldn't be reached, through every retry."""
