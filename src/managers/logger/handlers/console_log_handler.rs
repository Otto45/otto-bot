use std::error::Error;
use std::io::Write;

use chrono::SecondsFormat;
use serde::Serialize;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::contextual_data::{ContextValue, ContextualData};
use crate::managers::logger::handlers::log_handler_protocol::LogHandlerProtocol;
use crate::managers::logger::log_entry::LogEntry;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::logger::log_redactor::LogRedactor;
use crate::managers::logger::panic_exception::PanicException;

const LOGGER_NAME: &str = "otto_bot";
// The longest a message and a stack may be: an error that quotes a whole response or file must not make a line that
// fills the log.
const MAX_MESSAGE_LENGTH: usize = 4_000;
const MAX_STACK_LENGTH: usize = 16_000;
const TRUNCATED: &str = " [truncated]";

// One line, its fields in this order.
#[derive(Serialize)]
struct ConsoleLine {
    time: String,
    level: &'static str,
    logger: &'static str,
    message: String,
    context: ContextualData,
    source: ConsoleSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ConsoleError>,
}

#[derive(Serialize)]
struct ConsoleSource {
    file: &'static str,
    line: u32,
    column: u32,
}

#[derive(Serialize)]
struct ConsoleError {
    #[serde(rename = "type")]
    type_name: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stack: Option<String>,
}

// Writes each log entry to the console (stderr) as one line of JSON, with an exception's stack inside it, so a log
// store (CloudWatch) keeps each entry, stack and all, as one event whose fields it can search and count. What must
// never be logged is taken out first (LogRedactor).
pub struct ConsoleLogHandler;

impl LogHandlerProtocol for ConsoleLogHandler {
    fn handle(&self, log_entry: &LogEntry<'_>) {
        let console_line: ConsoleLine = ConsoleLogHandler::to_console_line(log_entry);
        let line: String = serde_json::to_string(&console_line).expect("a log line serializes to JSON");

        // Locking stderr keeps lines written from several threads at once from interleaving. A line that can't be
        // written has nowhere else to go, so it is dropped.
        let mut stderr: std::io::StderrLock<'static> = std::io::stderr().lock();
        let _ = writeln!(stderr, "{line}");
        let _ = stderr.flush();
    }

    fn close(&self) {
        // Holds nothing to release: the process owns stderr.
    }
}

impl ConsoleLogHandler {
    fn to_console_line(log_entry: &LogEntry<'_>) -> ConsoleLine {
        let mut context: ContextualData = ContextualData::new();
        for (key, value) in log_entry.context.iter() {
            let cleaned_value: ContextValue = match value {
                ContextValue::String(text) => ContextValue::String(ConsoleLogHandler::clean(text, MAX_MESSAGE_LENGTH)),
                other => other.clone(),
            };
            context.insert(key, cleaned_value);
        }

        ConsoleLine {
            time: log_entry.occurred_on.to_rfc3339_opts(SecondsFormat::Millis, true),
            level: log_entry.level.name(),
            logger: LOGGER_NAME,
            message: ConsoleLogHandler::clean(log_entry.message, MAX_MESSAGE_LENGTH),
            context,
            source: ConsoleSource {
                file: log_entry.source.file_path,
                line: log_entry.source.line_number,
                column: log_entry.source.column_number,
            },
            error: log_entry
                .exception
                .map(|exception: &(dyn Error + 'static)| ConsoleError {
                    type_name: ConsoleLogHandler::type_name(exception),
                    message: ConsoleLogHandler::clean(&exception.to_string(), MAX_MESSAGE_LENGTH),
                    // Only what an engineer must look at carries its stack: a client's mistake, a warning, needs none.
                    stack: (log_entry.level >= LogLevel::Error)
                        .then(|| ConsoleLogHandler::clean(&ConsoleLogHandler::stack(exception), MAX_STACK_LENGTH)),
                }),
        }
    }

    // Rust keeps no type names at run time, so the types this app logs are recognized by downcasting.
    fn type_name(exception: &(dyn Error + 'static)) -> &'static str {
        if let Some(application_exception) = exception.downcast_ref::<ApplicationBaseException>() {
            return application_exception.kind().name();
        }
        if exception.is::<PanicException>() {
            return "PanicException";
        }
        "Error"
    }

    // The stack where the exception was raised, then each error it was raised from.
    fn stack(exception: &(dyn Error + 'static)) -> String {
        let mut stack: String = format!("{}: {exception}\n", ConsoleLogHandler::type_name(exception));
        if let Some(application_exception) = exception.downcast_ref::<ApplicationBaseException>() {
            stack.push_str(&application_exception.backtrace().to_string());
        } else if let Some(panic_exception) = exception.downcast_ref::<PanicException>() {
            stack.push_str(&format!(
                "at {}\n{}",
                panic_exception.location(),
                panic_exception.backtrace()
            ));
        }

        let mut source: Option<&(dyn Error + 'static)> = exception.source();
        while let Some(cause) = source {
            stack.push_str(&format!("\nCaused by: {cause}"));
            source = cause.source();
        }
        stack
    }

    fn clean(text: &str, max_length: usize) -> String {
        let redacted: String = LogRedactor::redact(text);
        match redacted.char_indices().nth(max_length) {
            None => redacted,
            Some((byte_index, _)) => format!("{}{TRUNCATED}", &redacted[..byte_index]),
        }
    }
}
