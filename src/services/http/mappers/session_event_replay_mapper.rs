use crate::managers::models::session_event_replay::SessionEventReplay;

pub struct SessionEventReplayMapper;

impl SessionEventReplayMapper {
    /// What a client that opens the event stream gets first: the kept events after the one it saw last (`?after`, or
    /// the Last-Event-ID header its EventSource sends when it reconnects), or else the current run's from its start,
    /// so a stream opened just after a message was sent misses nothing of its run. A Last-Event-ID that is no event's
    /// number is as if there were none.
    pub fn to_session_event_replay(after: Option<u64>, last_event_id: Option<&str>) -> SessionEventReplay {
        after
            .or_else(|| last_event_id.and_then(|last_event_id: &str| last_event_id.trim().parse::<u64>().ok()))
            .map_or(SessionEventReplay::FromRunStart, SessionEventReplay::After)
    }
}
