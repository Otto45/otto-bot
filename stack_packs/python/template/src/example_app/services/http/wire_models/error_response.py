from pydantic import BaseModel


# Every error the service answers with, in one shape, so callers handle one shape of error.
class ErrorResponse(BaseModel):
    error: str
