import json
import sys
import threading
import traceback
from typing import Final, final

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.exceptions.context_value import ContextValue
from example_app.managers.logger.handlers.json_value import JsonValue
from example_app.managers.logger.handlers.log_handler_protocol import LogHandlerProtocol
from example_app.managers.logger.log_entry import LogEntry
from example_app.managers.logger.log_level import LogLevel
from example_app.managers.logger.log_redactor import LogRedactor

_LOGGER_NAME: Final[str] = "example_app"
# The longest a message and a stack may be: an exception that quotes a whole response or file must not make a line that
# fills the log.
_MAX_MESSAGE_LENGTH: Final[int] = 4_000
_MAX_STACK_LENGTH: Final[int] = 16_000
_TRUNCATED: Final[str] = " [truncated]"


# Writes each log entry to the console (stderr) as one line of JSON, with an exception's stack inside it, so a log store
# (CloudWatch) keeps each entry, stack and all, as one event whose fields it can search and count. What must never be
# logged is taken out first (LogRedactor).
@final
class ConsoleLogHandler(LogHandlerProtocol):
    def __init__(self) -> None:
        # Keeps lines written from several threads at once from interleaving.
        self._lock: threading.Lock = threading.Lock()

    def handle(self, log_entry: LogEntry) -> None:
        line: str = json.dumps(ConsoleLogHandler._to_console_line(log_entry), ensure_ascii=False)
        with self._lock:
            # A line that can't be written has nowhere else to go, so it is dropped.
            try:
                sys.stderr.write(line + "\n")
                sys.stderr.flush()
            except OSError, ValueError:
                pass

    def close(self) -> None:
        # Holds nothing to release: the process owns stderr.
        pass

    @staticmethod
    def _to_console_line(log_entry: LogEntry) -> dict[str, JsonValue]:
        context: dict[str, JsonValue] = {
            key: ConsoleLogHandler._clean_value(value) for key, value in log_entry.context.to_dict().items()
        }
        console_line: dict[str, JsonValue] = {
            "time": log_entry.occurred_on.isoformat(timespec="milliseconds").replace("+00:00", "Z"),
            "level": log_entry.level.name,
            "logger": _LOGGER_NAME,
            "message": ConsoleLogHandler._clean(log_entry.message, _MAX_MESSAGE_LENGTH),
            "context": context,
        }
        if log_entry.source is not None:
            console_line["source"] = {
                "file": log_entry.source.file_path,
                "line": log_entry.source.line_number,
                "function": log_entry.source.function_name,
            }
        if log_entry.exception is not None:
            error: dict[str, JsonValue] = {
                "type": ConsoleLogHandler._type_name(log_entry.exception),
                "message": ConsoleLogHandler._clean(str(log_entry.exception), _MAX_MESSAGE_LENGTH),
            }
            # Only what an engineer must look at carries its stack: a client's mistake, a warning, needs none.
            if log_entry.level >= LogLevel.ERROR:
                error["stack"] = ConsoleLogHandler._clean(
                    ConsoleLogHandler._stack(log_entry.exception), _MAX_STACK_LENGTH
                )
            console_line["error"] = error
        return console_line

    @staticmethod
    def _type_name(exception: BaseException) -> str:
        if isinstance(exception, ApplicationBaseException):
            return exception.get_name()
        return type(exception).__name__

    @staticmethod
    def _stack(exception: BaseException) -> str:
        # The traceback where the exception was raised, then each exception it was raised from or while handling.
        return "".join(traceback.format_exception(exception)).rstrip()

    @staticmethod
    def _clean_value(value: ContextValue) -> JsonValue:
        if isinstance(value, str):
            return ConsoleLogHandler._clean(value, _MAX_MESSAGE_LENGTH)
        return value

    @staticmethod
    def _clean(text: str, max_length: int) -> str:
        redacted: str = LogRedactor.redact(text)
        if len(redacted) <= max_length:
            return redacted
        return redacted[:max_length] + _TRUNCATED
