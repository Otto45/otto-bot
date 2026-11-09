import asyncio
import random
from http import HTTPStatus
from typing import Final, final

import httpx
from pydantic import ValidationError

from example_app.managers.common.formatters.validation_error_formatter import ValidationErrorFormatter
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_user_profile_response_exception import (
    InvalidUserProfileResponseException,
)
from example_app.managers.exceptions.json_placeholder_request_exception import JsonPlaceholderRequestException
from example_app.managers.exceptions.json_placeholder_unavailable_exception import (
    JsonPlaceholderUnavailableException,
)
from example_app.managers.exceptions.user_profile_not_found_exception import UserProfileNotFoundException
from example_app.managers.gateways.json_placeholder.mappers.user_profile_mapper import UserProfileMapper
from example_app.managers.gateways.json_placeholder.resource_models.user_resource import UserResource
from example_app.managers.gateways.json_placeholder.validators.validator_user_resource import ValidatorUserResource
from example_app.managers.logger.application_logger import ApplicationLogger
from example_app.managers.models.user_profile import UserProfile

# How long the gateway waits before sending a failed request again: doubling from the first, up to the longest, with up
# to a quarter of a second added at random so requests that failed together aren't sent together. The longest the
# service's own Retry-After is waited for; asking for longer is treated as this.
_FIRST_RETRY_DELAY_SECONDS: Final[float] = 1
_LONGEST_RETRY_DELAY_SECONDS: Final[float] = 20
_MAX_RETRY_JITTER_SECONDS: Final[float] = 0.25
_LONGEST_RETRY_AFTER_SECONDS: Final[float] = 60
# The statuses that mean JSON Placeholder failed on its side or is limiting requests for now.
_UNAVAILABLE_STATUSES: Final[frozenset[int]] = frozenset(
    {
        HTTPStatus.TOO_MANY_REQUESTS,
        HTTPStatus.INTERNAL_SERVER_ERROR,
        HTTPStatus.BAD_GATEWAY,
        HTTPStatus.SERVICE_UNAVAILABLE,
        HTTPStatus.GATEWAY_TIMEOUT,
    }
)


@final
class JsonPlaceholderGateway:
    def __init__(self, http_client: httpx.AsyncClient, max_retries: int) -> None:
        """Given its client, built by a seam with the service's base URL and the configured timeout, and how many times
        a request that failed for a reason that may pass is sent again."""
        self._http_client: httpx.AsyncClient = http_client
        self._max_retries: int = max_retries

    async def get_user_profile(self, user_id: int) -> UserProfile:
        context: ContextualData = ContextualData().with_value("user_id", user_id)

        response: httpx.Response = await self._get(f"/users/{user_id}", "retrieving a user profile", context)
        if response.status_code == HTTPStatus.NOT_FOUND:
            raise UserProfileNotFoundException("The user profile was not found", context)
        if not response.is_success:
            raise JsonPlaceholderRequestException(
                "JSON Placeholder request failed while retrieving a user profile",
                context.with_value("error_type", "HTTPStatusError").with_value("status_code", response.status_code),
            )

        user_resource: UserResource = JsonPlaceholderGateway._deserialize_user_resource(response.content, context)
        ValidatorUserResource.validate_user_resource(user_resource)
        return UserProfileMapper.to_user_profile(user_resource)

    async def close(self) -> None:
        """Closes the client's pooled connections."""
        await self._http_client.aclose()

    async def _get(self, path: str, operation_description: str, context: ContextualData) -> httpx.Response:
        """A GET's response. A GET is safe to send twice, so one that fails for a reason that may pass (a connection
        that couldn't be made or timed out, an _UNAVAILABLE_STATUSES answer) is sent again, up to max_retries times,
        after a growing wait or the one the service asks for; the last failure is a JsonPlaceholderUnavailableException.
        """
        retry: int = 0
        while True:
            try:
                return await self._get_once(path, operation_description, context)
            except JsonPlaceholderUnavailableException as exception:
                if retry >= self._max_retries:
                    raise
                retry += 1
                delay_seconds: float = JsonPlaceholderGateway._get_retry_delay_seconds(retry, exception)
                ApplicationLogger.warning(
                    "JSON Placeholder failed a request; it is sent again",
                    exception.context.with_value("retry", retry).with_value("delay_seconds", delay_seconds),
                    exception,
                )
                await asyncio.sleep(delay_seconds)

    async def _get_once(self, path: str, operation_description: str, context: ContextualData) -> httpx.Response:
        try:
            response: httpx.Response = await self._http_client.get(path)
        except (httpx.ConnectError, httpx.TimeoutException) as error:
            # A request that couldn't reach the service, or timed out, may well get through if sent again.
            raise JsonPlaceholderUnavailableException(
                f"JSON Placeholder is unavailable while {operation_description}",
                context.with_value("error_type", type(error).__name__),
            ) from error
        except httpx.HTTPError as error:
            raise JsonPlaceholderRequestException(
                f"JSON Placeholder request failed while {operation_description}",
                context.with_value("error_type", type(error).__name__),
            ) from error

        if response.status_code in _UNAVAILABLE_STATUSES:
            exception_context: ContextualData = context.with_value("error_type", "HTTPStatusError").with_value(
                "status_code", response.status_code
            )
            # How long the service asks to be left before the request is sent again, in whole seconds.
            retry_after: str = response.headers.get("retry-after", "").strip()
            if retry_after.isdigit():
                exception_context: ContextualData = exception_context.with_value(
                    "retry_after_seconds", int(retry_after)
                )
            raise JsonPlaceholderUnavailableException(
                f"JSON Placeholder is unavailable while {operation_description}", exception_context
            )
        return response

    @staticmethod
    def _get_retry_delay_seconds(retry: int, exception: JsonPlaceholderUnavailableException) -> float:
        """The retry-th wait: what the service asked for (Retry-After, up to _LONGEST_RETRY_AFTER_SECONDS), or doubling
        from _FIRST_RETRY_DELAY_SECONDS up to _LONGEST_RETRY_DELAY_SECONDS, whichever is longer, with jitter."""
        backoff: float = min(_FIRST_RETRY_DELAY_SECONDS * 2 ** (retry - 1), _LONGEST_RETRY_DELAY_SECONDS)
        retry_after: object = exception.context.get("retry_after_seconds")
        asked: float = min(float(retry_after), _LONGEST_RETRY_AFTER_SECONDS) if isinstance(retry_after, int) else 0
        return max(backoff, asked) + random.uniform(0, _MAX_RETRY_JITTER_SECONDS)

    @staticmethod
    def _deserialize_user_resource(body: bytes, context: ContextualData) -> UserResource:
        try:
            return UserResource.model_validate_json(body)
        except ValidationError as error:
            raise InvalidUserProfileResponseException(
                "User profile response could not be deserialized",
                context.with_value("errors", ValidationErrorFormatter.format(error, "body")),
            ) from error
