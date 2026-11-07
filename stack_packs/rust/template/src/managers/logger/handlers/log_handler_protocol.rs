use crate::managers::logger::log_entry::LogEntry;

pub trait LogHandlerProtocol: Send + Sync {
    fn handle(&self, log_entry: &LogEntry<'_>);

    fn close(&self);
}
