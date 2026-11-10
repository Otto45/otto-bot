use std::sync::{Arc, Mutex};
use std::time::Instant;

use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::runtime::live_run::LiveRun;
use crate::managers::runtime::session_event_log::SessionEventLog;

/// A session the manager is serving: the session, the channel its runs report their events on (relayed into its
/// event log, which its clients follow), its current or last run, and the run before it while that run's task is
/// still compacting.
pub struct LiveSession {
    pub session: Session,
    pub events: broadcast::Sender<SessionEvent>,
    pub event_log: Arc<Mutex<SessionEventLog>>,
    // Numbers each event the runs send and records it in the event log; it ends once the session and its runs are gone.
    pub relay: JoinHandle<()>,
    pub run: Option<LiveRun>,
    pub compacting_run: Option<LiveRun>,
    /// Held by a run's task from start to finish, compaction included, so a run starts only once the one before it
    /// has finished compacting.
    pub work_lock: Arc<tokio::sync::Mutex<()>>,
    /// When a client last used it: a session idle long enough, with no run and no client following, is let go.
    pub last_used: Instant,
}

impl LiveSession {
    /// Whether a run of it is under way (one that has ended, but is still compacting, is not).
    pub fn is_running(&self) -> bool {
        self.run
            .as_ref()
            .is_some_and(|live_run: &LiveRun| !live_run.has_ended())
    }

    /// Whether nothing of it is running and no client follows it.
    pub fn is_idle(&self) -> bool {
        let finished = |live_run: &Option<LiveRun>| -> bool { live_run.as_ref().is_none_or(LiveRun::is_finished) };
        finished(&self.run)
            && finished(&self.compacting_run)
            && !self
                .event_log
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .has_clients()
    }
}
