import asyncio
import sys
import threading
from types import TracebackType
from typing import Any, final

from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.logger.application_logger import ApplicationLogger


# Logs an exception nothing caught, as every other line is logged: one that ends the app, one that ends a thread, and
# one raised in an asyncio task nothing awaited. Every entry point installs it, so nothing that can fail runs without
# something to catch it. Ctrl-C (KeyboardInterrupt) is how a person stops the app, not a failure, so it isn't logged.
@final
class UncaughtExceptionLogger:
    @staticmethod
    def install() -> None:
        sys.excepthook = UncaughtExceptionLogger._log_uncaught_exception
        threading.excepthook = UncaughtExceptionLogger._log_uncaught_thread_exception

    @staticmethod
    def install_for_event_loop() -> None:
        """Called inside the running event loop, which is the one it watches."""
        asyncio.get_running_loop().set_exception_handler(UncaughtExceptionLogger._log_event_loop_exception)

    @staticmethod
    def _log_uncaught_exception(
        exception_type: type[BaseException], exception: BaseException, _traceback: TracebackType | None
    ) -> None:
        if issubclass(exception_type, KeyboardInterrupt):
            return
        ApplicationLogger.error("The app stopped on an unexpected error", ContextualData(), exception)

    @staticmethod
    def _log_uncaught_thread_exception(arguments: threading.ExceptHookArgs) -> None:
        if arguments.exc_value is None or issubclass(arguments.exc_type, KeyboardInterrupt):
            return
        thread_name: str = arguments.thread.name if arguments.thread is not None else ""
        ApplicationLogger.error(
            "A thread stopped on an unexpected error",
            ContextualData().with_value("thread_name", thread_name),
            arguments.exc_value,
        )

    # asyncio's handler is given a dict of whatever the loop knows about the failure, typed Any by asyncio itself.
    @staticmethod
    def _log_event_loop_exception(_loop: asyncio.AbstractEventLoop, context: dict[str, Any]) -> None:
        exception: object = context.get("exception")
        message: object = context.get("message")
        ApplicationLogger.error(
            "An asyncio task failed unexpectedly",
            ContextualData().with_value("event_loop_message", str(message)),
            exception if isinstance(exception, BaseException) else None,
        )
