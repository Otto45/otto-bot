from http import HTTPStatus
from typing import final

from fastapi import Request
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse

from example_app.services.http.wire_models.error_response import ErrorResponse


# How FastAPI's own answer to a request that doesn't fit (a body that is not JSON or not the request model, a path
# parameter of the wrong type) goes over the wire: a 422 in the ErrorResponse every other error is answered with, so
# that callers handle one shape of error. It says where each problem is, never the value sent.
@final
class RequestValidationExceptionHandler:
    @staticmethod
    async def handle(request: Request, exception: Exception) -> JSONResponse:
        if not isinstance(exception, RequestValidationError):
            raise exception
        # FastAPI describes each problem as a dict of its own making (typed Any): where it is, and what is wrong.
        problems: list[str] = [
            f"{'.'.join(str(part) for part in error['loc'])}: {error['msg']}" for error in exception.errors()
        ]
        error_response: ErrorResponse = ErrorResponse(error="; ".join(problems))
        return JSONResponse(error_response.model_dump(mode="json"), status_code=HTTPStatus.UNPROCESSABLE_CONTENT)
