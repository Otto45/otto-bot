use tokio::sync::broadcast;

use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::runtime::live_run::LiveRun;

/// A session the manager is serving: the session, the channel its events go out on, and its current or last run.
pub struct LiveSession {
    pub session: Session,
    pub events: broadcast::Sender<SessionEvent>,
    pub run: Option<LiveRun>,
}
