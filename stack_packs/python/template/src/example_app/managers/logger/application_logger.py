import inspect
import threading
from datetime import UTC, datetime
from types import FrameType
from typing import ClassVar, Final, final

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.logger.handlers.console_log_handler import ConsoleLogHandler
from example_app.managers.logger.handlers.log_handler_protocol import LogHandlerProtocol
from example_app.managers.logger.log_entry import LogEntry
from example_app.managers.logger.log_level import LogLevel
from example_app.managers.logger.log_source import LogSource

_DEFAULT_LEVEL: Final[LogLevel] = LogLevel.INFO


# Exception to the architecture: ApplicationLogger is a global singleton that owns its own handlers, so any code can
# import it and log (e.g. ApplicationLogger.info(...)) without going through the ServiceLocator. It configures itself
# with defaults on first use.
@final
class ApplicationLogger:
    _level: ClassVar[LogLevel] = _DEFAULT_LEVEL
    _handlers: ClassVar[list[LogHandlerProtocol] | None] = None
    _lock: ClassVar[threading.RLock] = threading.RLock()

    @classmethod
    def configure(cls, level: LogLevel = _DEFAULT_LEVEL, handlers: list[LogHandlerProtocol] | None = None) -> None:
        """Called only at app boot (e.g. in main()). Without handlers (e.g. local development), logs go to the
        console."""
        with cls._lock:
            cls.close()
            cls._level = level
            cls._handlers = handlers if handlers is not None else [ConsoleLogHandler()]

    @classmethod
    def close(cls) -> None:
        with cls._lock:
            for handler in cls._handlers or []:
                handler.close()
            cls._handlers = None

    @classmethod
    def debug(cls, message: str, context: ContextualData | None = None) -> None:
        cls._log(LogLevel.DEBUG, message, context, None)

    @classmethod
    def info(cls, message: str, context: ContextualData | None = None) -> None:
        cls._log(LogLevel.INFO, message, context, None)

    @classmethod
    def warning(
        cls, message: str, context: ContextualData | None = None, exception: BaseException | None = None
    ) -> None:
        cls._log(LogLevel.WARNING, message, context, exception)

    @classmethod
    def error(cls, message: str, context: ContextualData | None = None, exception: BaseException | None = None) -> None:
        """Logs what failed, with the exception that made it fail when there is one: an unexpected exception is logged
        this way, its message saying what was being done (e.g. "A request failed unexpectedly")."""
        cls._log(LogLevel.ERROR, message, context, exception)

    @classmethod
    def log_exception(cls, exception: ApplicationBaseException, level: LogLevel) -> None:
        """Logs one of the application's own exceptions, with its message and context. A client's mistake (a value
        that is not valid, something not found) is a WARNING; ERROR is kept for what an engineer must look at."""
        cls._log(level, exception.message, exception.context, exception)

    @classmethod
    def _log(
        cls,
        level: LogLevel,
        message: str,
        context: ContextualData | None,
        exception: BaseException | None,
    ) -> None:
        if level < cls._level:
            return

        log_entry: LogEntry = LogEntry(
            level=level,
            message=message,
            context=context if context is not None else ContextualData(),
            occurred_on=datetime.now(UTC),
            source=ApplicationLogger._get_caller_source(),
            exception=exception,
        )

        with cls._lock:
            if cls._handlers is None:
                cls.configure()
            handlers: list[LogHandlerProtocol] = list(cls._handlers or [])
        for handler in handlers:
            handler.handle(log_entry)

    @staticmethod
    def _get_caller_source() -> LogSource | None:
        # This function, _log, the public method, then the code that logged.
        frame: FrameType | None = inspect.currentframe()
        for _ in range(3):
            frame = frame.f_back if frame is not None else None
        if frame is None:
            return None
        return LogSource(
            file_path=frame.f_code.co_filename,
            line_number=frame.f_lineno,
            function_name=frame.f_code.co_qualname,
        )
