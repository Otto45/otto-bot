from http import HTTPStatus
from typing import final

from fastapi.responses import JSONResponse
from starlette.types import ASGIApp, Receive, Scope, Send

from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.logger.application_logger import ApplicationLogger
from example_app.services.http.middleware.response_start_tracker import ResponseStartTracker
from example_app.services.http.wire_models.error_response import ErrorResponse


# Logs, with its stack and the request it failed, any exception a request raised that the exception handlers don't
# answer (the app's own are answered by ApplicationExceptionHandler), and answers it generically so nothing of the
# app's internals leaks. It is logged once, here, where it is caught: it goes no further, so the server doesn't log it
# too. An exception raised after the response has started is past answering: it goes on to the server, which ends that
# connection and logs it through the LoggingBridge.
@final
class UnexpectedErrorMiddleware:
    def __init__(self, application: ASGIApp) -> None:
        self._application: ASGIApp = application

    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        if scope["type"] != "http":
            await self._application(scope, receive, send)
            return

        response_start_tracker: ResponseStartTracker = ResponseStartTracker(send)
        try:
            await self._application(scope, receive, response_start_tracker.send)
        except Exception as exception:
            if response_start_tracker.started:
                raise
            ApplicationLogger.error(
                "A request failed unexpectedly",
                ContextualData().with_value("method", str(scope["method"])).with_value("path", str(scope["path"])),
                exception,
            )
            error_response: ErrorResponse = ErrorResponse(error="The request could not be completed")
            response: JSONResponse = JSONResponse(
                error_response.model_dump(mode="json"), status_code=HTTPStatus.INTERNAL_SERVER_ERROR
            )
            await response(scope, receive, send)
