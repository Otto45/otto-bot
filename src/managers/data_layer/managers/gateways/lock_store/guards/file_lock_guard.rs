use std::future::Future;
use std::pin::Pin;

use crate::managers::runtime::lock_guard_protocol::LockGuardProtocol;

/// A lock file the operating system has locked for this process: released when it is dropped (closing the file
/// would release it too), and by the operating system when the process exits.
pub struct FileLockGuard {
    file: std::fs::File,
}

impl FileLockGuard {
    pub fn new(file: std::fs::File) -> Self {
        FileLockGuard { file }
    }
}

impl LockGuardProtocol for FileLockGuard {
    // Dropping it unlocks the file there and then, so the lock is free once this returns.
    fn release(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        drop(self);
        Box::pin(async {})
    }

    // The operating system holds the lock for as long as this process has the file open.
    fn lost(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(std::future::pending())
    }
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        // Closing the file would release it too; unlocking first says so. An unlock that fails is released on close.
        self.file.unlock().ok();
    }
}
