use std::future::Future;
use std::pin::Pin;

use crate::managers::runtime::lock_guard_protocol::LockGuardProtocol;

/// A lock the lock's store holds for a run: on an app, while the run changes its files, or on a session, while its
/// run works on its conversation (compaction included). The store decides what holds it (an operating system's file
/// lock, or a Postgres advisory lock), so its holder never learns which. `release` lets go of it and returns once it
/// is free, so a request sent as soon as a run's end is seen finds the lock free; if it is only dropped (a run that
/// is cancelled or panics) it is let go of as soon as it can be, and when the process exits, however it exits, so a
/// lock is never left behind. The store makes it; the manager's run holds it.
pub struct HeldLock {
    guard: Option<Box<dyn LockGuardProtocol>>,
}

impl HeldLock {
    pub fn new(guard: Box<dyn LockGuardProtocol>) -> Self {
        HeldLock { guard: Some(guard) }
    }

    /// Completes if the lock is lost while it is held (a database connection holding it ended), so its holder can
    /// stop what the lock was protecting; never, for a lock that is kept until it is let go of.
    pub fn lost(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        match self.guard.as_ref() {
            Some(guard) => guard.lost(),
            None => Box::pin(std::future::pending()),
        }
    }

    /// Lets go of the lock, returning once another holder can take it.
    pub async fn release(mut self) {
        if let Some(guard) = self.guard.take() {
            guard.release().await;
        }
    }
}
