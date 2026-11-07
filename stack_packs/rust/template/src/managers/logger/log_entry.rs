use std::error::Error;

use chrono::{DateTime, Utc};

use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::log_level::LogLevel;

/// Where in the code an entry was logged. Rust knows the caller's file, line and column (`#[track_caller]`), but not
/// the name of its function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSource {
    pub file_path: &'static str,
    pub line_number: u32,
    pub column_number: u32,
}

/// One entry, handed to every handler while it is logged: it borrows what it was logged with rather than copying it.
#[derive(Debug, Clone, Copy)]
pub struct LogEntry<'a> {
    pub level: LogLevel,
    pub message: &'a str,
    pub context: &'a ContextualData,
    pub occurred_on: DateTime<Utc>,
    pub source: &'a LogSource,
    pub exception: Option<&'a (dyn Error + 'static)>,
}
