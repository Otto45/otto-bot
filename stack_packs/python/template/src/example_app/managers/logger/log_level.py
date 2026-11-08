from enum import IntEnum


# How much a log entry matters; entries below the configured level are dropped.
class LogLevel(IntEnum):
    DEBUG = 10
    INFO = 20
    WARNING = 30
    ERROR = 40
