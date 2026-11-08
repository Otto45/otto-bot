from http import HTTPStatus
from typing import final

from fastapi import Request
from fastapi.responses import JSONResponse

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.logger.application_logger import ApplicationLogger
from example_app.managers.logger.log_level import LogLevel
from example_app.services.http.mappers.http_error_mapper import HttpErrorMapper
from example_app.services.http.wire_models.error_response import ErrorResponse


# What a route answers when the facade raises: FastAPI's idiom is an exception handler, so a route never catches an
# exception itself.
@final
class ApplicationExceptionHandler:
    @staticmethod
    async def handle(request: Request, exception: Exception) -> JSONResponse:
        # Registered for ApplicationBaseException only; anything else is not this handler's to answer.
        if not isinstance(exception, ApplicationBaseException):
            raise exception
        status_code: HTTPStatus = HttpErrorMapper.to_status_code(exception)

        # A client's mistake is a warning, easy to find on its own; ERROR is kept for what an engineer must look at.
        if status_code.is_server_error:
            ApplicationLogger.log_exception(exception, LogLevel.ERROR)
        else:
            ApplicationLogger.log_exception(exception, LogLevel.WARNING)

        error_response: ErrorResponse = HttpErrorMapper.to_error_response(exception, status_code)
        return JSONResponse(error_response.model_dump(mode="json"), status_code=status_code)
