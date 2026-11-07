use tokio::task::{AbortHandle, JoinHandle};

/// A run under way: its task, which cancelling aborts, and the watcher that reports how the task ended.
pub struct LiveRun {
    pub abort_handle: AbortHandle,
    pub watcher: JoinHandle<()>,
}

impl LiveRun {
    /// Whether the run has ended and its end has been reported.
    pub fn is_finished(&self) -> bool {
        self.watcher.is_finished()
    }
}
