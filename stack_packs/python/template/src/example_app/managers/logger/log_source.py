from dataclasses import dataclass


# Where in the code an entry was logged: the code that called the ApplicationLogger.
@dataclass(frozen=True, slots=True)
class LogSource:
    file_path: str
    line_number: int
    function_name: str
