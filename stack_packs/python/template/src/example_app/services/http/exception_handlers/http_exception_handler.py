from typing import final

from fastapi import Request
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException

from example_app.services.http.wire_models.error_response import ErrorResponse


# How FastAPI's own HTTP errors (a path that is no route's, a method its route doesn't accept) go over the wire: their
# status, message and headers (a 405's Allow), in the ErrorResponse every other error is answered with, so that callers
# handle one shape of error rather than FastAPI's {"detail": ...}.
@final
class HttpExceptionHandler:
    @staticmethod
    async def handle(request: Request, exception: Exception) -> JSONResponse:
        if not isinstance(exception, HTTPException):
            raise exception
        error_response: ErrorResponse = ErrorResponse(error=exception.detail)
        return JSONResponse(
            error_response.model_dump(mode="json"), status_code=exception.status_code, headers=exception.headers
        )
