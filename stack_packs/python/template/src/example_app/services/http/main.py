import asyncio
import os
import signal
from types import FrameType
from typing import Final

from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.logger.application_logger import ApplicationLogger
from example_app.managers.logger.log_level import LogLevel
from example_app.managers.logger.uncaught_exception_logger import UncaughtExceptionLogger
from example_app.services.http.http_service import HttpService
from example_app.services.http.logging_bridges.logging_bridge import LoggingBridge

# Where the service listens unless HOST and PORT say otherwise, as a deployment does. Kept here, outside the domain
# container, because the ConfigurationProvider serves the application behind the facade, not the processes that host
# it.
_HOST: Final[str] = "127.0.0.1"
_PORT: Final[int] = 8000


# Entry point (example-app-http): logger, logging bridge, uncaught exceptions, HOST and PORT, serve.
def main() -> int:
    ApplicationLogger.configure(LogLevel.INFO)
    # What uvicorn, httpx and the other libraries log goes through the ApplicationLogger too, and so does any exception
    # nothing caught.
    LoggingBridge.install()
    UncaughtExceptionLogger.install()
    try:
        return _run()
    finally:
        ApplicationLogger.close()


def _run() -> int:
    host: str = os.environ.get("HOST", _HOST)
    port_text: str = os.environ.get("PORT", str(_PORT))
    try:
        port: int = int(port_text)
    except ValueError as error:
        ApplicationLogger.error("PORT is not a port number", ContextualData().with_value("port", port_text), error)
        return 1

    # uvicorn handles Ctrl-C and SIGTERM while it serves (it stops taking requests and finishes the ones under way),
    # then hands each signal it caught back to the handler it found. This one does nothing, so the facade is closed
    # after the server stops, rather than the process ending before it is.
    signal.signal(signal.SIGINT, _ignore_signal)
    signal.signal(signal.SIGTERM, _ignore_signal)

    try:
        asyncio.run(HttpService.serve(host, port))
        return 0
    except ApplicationBaseException as exception:
        ApplicationLogger.log_exception(exception, LogLevel.ERROR)
        return 1
    except Exception as exception:
        ApplicationLogger.error("The service stopped on an unexpected error", None, exception)
        return 1


def _ignore_signal(signal_number: int, frame: FrameType | None) -> None:
    pass
