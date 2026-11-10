use std::sync::Arc;

use tokio::sync::broadcast;

use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::runtime::live_run::LiveRun;

/// A session the manager is serving: the session, the channel its events go out on, its current or last run, and the
/// run before it while that run's task is still compacting.
pub struct LiveSession {
    pub session: Session,
    pub events: broadcast::Sender<SessionEvent>,
    pub run: Option<LiveRun>,
    pub compacting_run: Option<LiveRun>,
    /// Held by a run's task from start to finish, compaction included, so a run starts only once the one before it
    /// has finished compacting.
    pub work_lock: Arc<tokio::sync::Mutex<()>>,
}
