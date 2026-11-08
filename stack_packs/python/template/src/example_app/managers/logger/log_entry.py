from dataclasses import dataclass
from datetime import datetime

from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.logger.log_level import LogLevel
from example_app.managers.logger.log_source import LogSource


# One entry, handed to every handler.
@dataclass(frozen=True, slots=True)
class LogEntry:
    level: LogLevel
    message: str
    context: ContextualData
    occurred_on: datetime
    source: LogSource | None
    exception: BaseException | None
