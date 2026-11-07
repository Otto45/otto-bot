use std::error::Error;
use std::panic::Location;
use std::sync::RwLock;

use chrono::Utc;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::handlers::console_log_handler::ConsoleLogHandler;
use crate::managers::logger::handlers::log_handler_protocol::LogHandlerProtocol;
use crate::managers::logger::log_entry::{LogEntry, LogSource};
use crate::managers::logger::log_level::LogLevel;

const DEFAULT_LEVEL: LogLevel = LogLevel::Info;

struct LoggerState {
    level: LogLevel,
    handlers: Option<Vec<Box<dyn LogHandlerProtocol>>>,
}

// The one logger of the process. Configured with defaults on first use (handlers is None until then).
static STATE: RwLock<LoggerState> = RwLock::new(LoggerState {
    level: DEFAULT_LEVEL,
    handlers: None,
});

// Exception to the architecture: ApplicationLogger is a global singleton that owns its own handlers, so any code can
// import it and log (e.g. ApplicationLogger::info(...)) without going through the ServiceLocator. Rust has no default
// arguments, so every call names its context (ContextualData::new() when there is none) and its exception.
pub struct ApplicationLogger;

impl ApplicationLogger {
    /// Called only at app boot (e.g. in main()). Without handlers (e.g. local development), logs go to the console.
    pub fn configure(level: LogLevel, handlers: Option<Vec<Box<dyn LogHandlerProtocol>>>) {
        ApplicationLogger::close();

        let mut state: std::sync::RwLockWriteGuard<'_, LoggerState> = ApplicationLogger::write_state();
        state.level = level;
        state.handlers = Some(handlers.unwrap_or_else(|| vec![Box::new(ConsoleLogHandler)]));
    }

    pub fn close() {
        let mut state: std::sync::RwLockWriteGuard<'_, LoggerState> = ApplicationLogger::write_state();
        for handler in state.handlers.take().unwrap_or_default() {
            handler.close();
        }
    }

    #[track_caller]
    pub fn debug(message: &str, context: ContextualData) {
        ApplicationLogger::log(LogLevel::Debug, message, &context, None, Location::caller());
    }

    #[track_caller]
    pub fn info(message: &str, context: ContextualData) {
        ApplicationLogger::log(LogLevel::Info, message, &context, None, Location::caller());
    }

    #[track_caller]
    pub fn warning(message: &str, context: ContextualData, exception: Option<&(dyn Error + 'static)>) {
        ApplicationLogger::log(LogLevel::Warning, message, &context, exception, Location::caller());
    }

    /// Logs what failed, with the exception that made it fail when there is one: an unexpected exception is logged
    /// this way, its message saying what was being done (e.g. "Run failed unexpectedly").
    #[track_caller]
    pub fn error(message: &str, context: ContextualData, exception: Option<&(dyn Error + 'static)>) {
        ApplicationLogger::log(LogLevel::Error, message, &context, exception, Location::caller());
    }

    /// Logs one of the application's own exceptions, with its message and context. A client's mistake (a value that
    /// is not valid, something not found) is a WARNING; ERROR is kept for what an engineer must look at.
    #[track_caller]
    pub fn log_exception(exception: &ApplicationBaseException, level: LogLevel) {
        ApplicationLogger::log(
            level,
            exception.message(),
            exception.context(),
            Some(exception),
            Location::caller(),
        );
    }

    fn log(
        level: LogLevel,
        message: &str,
        context: &ContextualData,
        exception: Option<&(dyn Error + 'static)>,
        location: &'static Location<'static>,
    ) {
        if ApplicationLogger::read_state().handlers.is_none() {
            ApplicationLogger::configure(DEFAULT_LEVEL, None);
        }

        let state: std::sync::RwLockReadGuard<'_, LoggerState> = ApplicationLogger::read_state();
        if level < state.level {
            return;
        }

        let source: LogSource = LogSource {
            file_path: location.file(),
            line_number: location.line(),
            column_number: location.column(),
        };
        let log_entry: LogEntry<'_> = LogEntry {
            level,
            message,
            context,
            occurred_on: Utc::now(),
            source: &source,
            exception,
        };

        for handler in state.handlers.iter().flatten() {
            handler.handle(&log_entry);
        }
    }

    // A handler that panicked mid-line leaves nothing half-done in the state, so a poisoned lock is still usable:
    // the logger must keep logging, above all while a panic is being reported.
    fn read_state() -> std::sync::RwLockReadGuard<'static, LoggerState> {
        STATE.read().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_state() -> std::sync::RwLockWriteGuard<'static, LoggerState> {
        STATE.write().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
