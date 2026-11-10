use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::task::{AbortHandle, JoinHandle};

/// A run under way: its task, which cancelling aborts, the watcher that reports how the task ended, and whether the
/// run has reported its end. A task can go on after its run ended, compacting the orchestrator's conversation.
pub struct LiveRun {
    pub abort_handle: AbortHandle,
    pub watcher: JoinHandle<()>,
    /// Set by the run's task just before it reports the run's end, so a request sent once the end is reported finds
    /// the run ended.
    pub end_reported: Arc<AtomicBool>,
}

impl LiveRun {
    /// Whether the run's task has ended, and its end has been reported.
    pub fn is_finished(&self) -> bool {
        self.watcher.is_finished()
    }

    /// Whether the run has ended, though its task may still be compacting.
    pub fn has_ended(&self) -> bool {
        self.end_reported.load(Ordering::SeqCst) || self.is_finished()
    }
}
