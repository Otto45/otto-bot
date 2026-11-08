# A value in a line of JSON the ConsoleLogHandler writes.
type JsonValue = str | int | float | bool | dict[str, JsonValue] | None
