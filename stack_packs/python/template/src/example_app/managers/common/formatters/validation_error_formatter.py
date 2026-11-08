from typing import final

from pydantic import ValidationError
from pydantic_core import ErrorDetails


# Turns the error that deserializing something from outside the process failed with (an HTTP body, a database row, a
# document, all read through pydantic) into one string, naming each problem by where in the input it is. pydantic
# reports every problem at once; one about the input as a whole (e.g. a body that isn't JSON) has no location, so it is
# named after root_name (e.g. "body", "row", "document"). The input's values are left out: they can be anything.
@final
class ValidationErrorFormatter:
    @staticmethod
    def format(error: ValidationError, root_name: str) -> str:
        return "; ".join(
            ValidationErrorFormatter._format_error_details(error_details, root_name)
            for error_details in error.errors(include_url=False, include_input=False)
        )

    @staticmethod
    def _format_error_details(error_details: ErrorDetails, root_name: str) -> str:
        location: str = ".".join(str(part) for part in error_details["loc"])
        return f"{location or root_name}: {error_details['msg']}"
