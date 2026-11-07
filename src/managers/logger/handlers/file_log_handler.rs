use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::managers::logger::handlers::console_log_handler::ConsoleLogHandler;
use crate::managers::logger::handlers::log_handler_protocol::LogHandlerProtocol;
use crate::managers::logger::log_entry::LogEntry;

// Appends each log entry to a file, as the same one line of JSON the ConsoleLogHandler writes. For a service whose
// terminal is its conversation (the CLI), so log lines don't interleave with it.
pub struct FileLogHandler {
    path: PathBuf,
    // None once closed.
    file: Mutex<Option<File>>,
}

impl FileLogHandler {
    /// Opens the file for appending, creating it and its folder if needed.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)?;
        }
        let file: File = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(FileLogHandler {
            path: path.to_path_buf(),
            file: Mutex::new(Some(file)),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn lock(&self) -> MutexGuard<'_, Option<File>> {
        self.file.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl LogHandlerProtocol for FileLogHandler {
    fn handle(&self, log_entry: &LogEntry<'_>) {
        let line: String = ConsoleLogHandler::to_json_line(log_entry);

        // The lock keeps lines written from several threads at once from interleaving. A line that can't be written
        // has nowhere else to go, so it is dropped.
        if let Some(file) = self.lock().as_mut() {
            let _ = writeln!(file, "{line}");
        }
    }

    fn close(&self) {
        if let Some(mut file) = self.lock().take() {
            let _ = file.flush();
        }
    }
}
