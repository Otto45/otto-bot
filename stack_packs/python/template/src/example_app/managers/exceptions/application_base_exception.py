from abc import ABC, abstractmethod
from typing import TYPE_CHECKING

from example_app.managers.exceptions.contextual_data import ContextualData

if TYPE_CHECKING:
    # Each family imports this module; annotations are evaluated lazily, so this import is only for the type checker.
    from example_app.managers.exceptions.exception_family import ExceptionFamily


# The base of every exception the application raises: a message and a context. Python keeps the stack where it was
# raised (its traceback) and the error it was raised from (`raise ... from error`), and both are logged with it.
#
# Each component's exceptions are a family: a base class that subclasses this one (e.g.
# RelationalDatabaseGatewayException), with a subclass per exception (e.g. CustomerNotFoundException). get_family()
# returns the exception as a member of ExceptionFamily, the union of every family, so a match on it that names every
# family is checked by pyright: a new family fails the type check until it is added to the union, and then every such
# match fails until it decides the new family.
class ApplicationBaseException(Exception, ABC):
    def __init__(self, message: str, context: ContextualData) -> None:
        super().__init__(message)
        self.message: str = message
        self.context: ContextualData = context

    @abstractmethod
    def get_family(self) -> ExceptionFamily: ...

    def get_name(self) -> str:
        """The name the exception is logged under (e.g. "CustomerNotFoundException")."""
        return type(self).__name__
