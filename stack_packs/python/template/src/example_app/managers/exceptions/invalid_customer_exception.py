from typing import TYPE_CHECKING, final

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException

if TYPE_CHECKING:
    from example_app.managers.exceptions.exception_family import ExceptionFamily


# A family of one. Raised for something the caller can put right, so its message is written for the person using the
# app: a service answers with it as it is. What only engineers need goes in the context, which is logged.
@final
class InvalidCustomerException(ApplicationBaseException):
    def get_family(self) -> ExceptionFamily:
        return self
