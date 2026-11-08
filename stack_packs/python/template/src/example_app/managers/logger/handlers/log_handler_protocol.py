from typing import Protocol

from example_app.managers.logger.log_entry import LogEntry


class LogHandlerProtocol(Protocol):
    def handle(self, log_entry: LogEntry) -> None: ...

    def close(self) -> None: ...
