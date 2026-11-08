from typing import final

from example_app.managers.exceptions.json_placeholder_gateway_exception import JsonPlaceholderGatewayException


@final
class UserProfileNotFoundException(JsonPlaceholderGatewayException):
    """No user profile has the id asked for: the caller can put it right, so its message is written for the
    person using the app."""
