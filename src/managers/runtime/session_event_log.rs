use tokio::sync::broadcast;

use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_event_record::SessionEventRecord;
use crate::managers::models::session_event_replay::SessionEventReplay;

// How many events a client's channel holds when the client is behind; one further behind is told it lagged, and can
// subscribe again after the last event it saw.
const CLIENT_EVENT_CAPACITY: usize = 1024;

/// A live session's events as its clients get them: each numbered, kept with its run's (a run's start forgets the last
/// run's), and sent to every client. Behind its manager's lock, so a client subscribing gets every event once: those
/// kept, then those sent after.
pub struct SessionEventLog {
    next_id: u64,
    run_events: Vec<SessionEventRecord>,
    clients: broadcast::Sender<SessionEventRecord>,
}

impl SessionEventLog {
    /// `first_id` is above any id an earlier process gave the session's events.
    pub fn new(first_id: u64) -> Self {
        let (clients, _): (
            broadcast::Sender<SessionEventRecord>,
            broadcast::Receiver<SessionEventRecord>,
        ) = broadcast::channel(CLIENT_EVENT_CAPACITY);
        SessionEventLog {
            next_id: first_id,
            run_events: Vec::new(),
            clients,
        }
    }

    pub fn record(&mut self, event: SessionEvent) {
        if event == SessionEvent::RunStarted {
            self.run_events.clear();
        }
        let session_event_record: SessionEventRecord = SessionEventRecord {
            id: self.next_id,
            event,
        };
        self.next_id += 1;
        self.run_events.push(session_event_record.clone());
        // No client may be following.
        self.clients.send(session_event_record).ok();
    }

    /// The kept events the replay asks for, and every event from now on.
    pub fn subscribe(
        &self,
        session_event_replay: SessionEventReplay,
    ) -> (Vec<SessionEventRecord>, broadcast::Receiver<SessionEventRecord>) {
        let missed: Vec<SessionEventRecord> = match session_event_replay {
            SessionEventReplay::FromNow => Vec::new(),
            SessionEventReplay::FromRunStart => self.run_events.clone(),
            SessionEventReplay::After(id) => self
                .run_events
                .iter()
                .filter(|session_event_record: &&SessionEventRecord| session_event_record.id > id)
                .cloned()
                .collect(),
        };
        (missed, self.clients.subscribe())
    }

    pub fn count_clients(&self) -> usize {
        self.clients.receiver_count()
    }

    pub fn has_clients(&self) -> bool {
        self.clients.receiver_count() > 0
    }
}
