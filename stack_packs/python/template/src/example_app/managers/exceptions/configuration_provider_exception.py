from typing import TYPE_CHECKING

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException

if TYPE_CHECKING:
    from example_app.managers.exceptions.exception_family import ExceptionFamily


class ConfigurationProviderException(ApplicationBaseException):
    """The ConfigurationProvider's exceptions: one family. A new exception is a subclass."""

    def get_family(self) -> ExceptionFamily:
        return self
