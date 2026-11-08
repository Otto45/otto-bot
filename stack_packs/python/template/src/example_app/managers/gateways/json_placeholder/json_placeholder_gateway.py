from http import HTTPStatus
from typing import final

import httpx
from pydantic import ValidationError

from example_app.managers.common.formatters.validation_error_formatter import ValidationErrorFormatter
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_user_profile_response_exception import (
    InvalidUserProfileResponseException,
)
from example_app.managers.exceptions.json_placeholder_request_exception import JsonPlaceholderRequestException
from example_app.managers.exceptions.user_profile_not_found_exception import UserProfileNotFoundException
from example_app.managers.gateways.json_placeholder.mappers.user_profile_mapper import UserProfileMapper
from example_app.managers.gateways.json_placeholder.resource_models.user_resource import UserResource
from example_app.managers.gateways.json_placeholder.validators.validator_user_resource import ValidatorUserResource
from example_app.managers.models.user_profile import UserProfile


@final
class JsonPlaceholderGateway:
    def __init__(self, http_client: httpx.AsyncClient) -> None:
        """Given its client, built by a seam with the service's base URL and the configured timeout."""
        self._http_client: httpx.AsyncClient = http_client

    async def get_user_profile(self, user_id: int) -> UserProfile:
        context: ContextualData = ContextualData().with_value("user_id", user_id)

        try:
            response: httpx.Response = await self._http_client.get(f"/users/{user_id}")
            if response.status_code == HTTPStatus.NOT_FOUND:
                raise UserProfileNotFoundException("The user profile was not found", context)
            response.raise_for_status()
        except httpx.HTTPError as error:
            raise JsonPlaceholderGateway._to_request_exception(error, "retrieving a user profile", context) from error

        user_resource: UserResource = JsonPlaceholderGateway._deserialize_user_resource(response.content, context)
        ValidatorUserResource.validate_user_resource(user_resource)
        return UserProfileMapper.to_user_profile(user_resource)

    async def close(self) -> None:
        """Closes the client's pooled connections."""
        await self._http_client.aclose()

    @staticmethod
    def _deserialize_user_resource(body: bytes, context: ContextualData) -> UserResource:
        try:
            return UserResource.model_validate_json(body)
        except ValidationError as error:
            raise InvalidUserProfileResponseException(
                "User profile response could not be deserialized",
                context.with_value("errors", ValidationErrorFormatter.format(error, "body")),
            ) from error

    @staticmethod
    def _to_request_exception(
        http_error: httpx.HTTPError, operation_description: str, context: ContextualData
    ) -> JsonPlaceholderRequestException:
        exception_context: ContextualData = context.with_value("error_type", type(http_error).__name__)
        if isinstance(http_error, httpx.HTTPStatusError):
            exception_context = exception_context.with_value("status_code", http_error.response.status_code)

        return JsonPlaceholderRequestException(
            f"JSON Placeholder request failed while {operation_description}", exception_context
        )
