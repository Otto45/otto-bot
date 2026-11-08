import logging
from typing import final

from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.logger.application_logger import ApplicationLogger


# Routes what the libraries the service runs log through Python's logging (uvicorn, httpx, asyncpg) to the
# ApplicationLogger, so every line the service writes has the one format. Installed on the root logger by main, before
# anything logs.
@final
class LoggingBridge(logging.Handler):
    @staticmethod
    def install() -> None:
        root_logger: logging.Logger = logging.getLogger()
        root_logger.handlers = [LoggingBridge()]
        root_logger.setLevel(logging.INFO)

    def emit(self, record: logging.LogRecord) -> None:
        context: ContextualData = (
            ContextualData()
            .with_value("logger", record.name)
            .with_value("origin", f"{record.pathname}:{record.lineno}")
        )
        exception: BaseException | None = record.exc_info[1] if record.exc_info is not None else None
        message: str = record.getMessage()

        if record.levelno >= logging.ERROR:
            ApplicationLogger.error(message, context, exception)
        elif record.levelno >= logging.WARNING:
            ApplicationLogger.warning(message, context, exception)
        elif record.levelno >= logging.INFO:
            ApplicationLogger.info(message, context)
        else:
            ApplicationLogger.debug(message, context)
