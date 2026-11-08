from typing import final

from example_app.managers.exceptions.json_placeholder_gateway_exception import JsonPlaceholderGatewayException


@final
class InvalidUserProfileResponseException(JsonPlaceholderGatewayException):
    """JSON Placeholder answered with a user that doesn't have what a UserProfile needs."""
