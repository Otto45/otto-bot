use crate::managers::models::session_event::SessionEvent;

/// A session's event as clients get it: numbered, so a client that reconnects can say which it saw last. Numbers rise
/// through a session's events, and across restarts of the process that serves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionEventRecord {
    pub id: u64,
    pub event: SessionEvent,
}
