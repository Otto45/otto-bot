use std::future::Future;
use std::pin::Pin;

/// What keeps a `HeldLock` held, which the lock's store makes: an open file the operating system has locked, or a
/// database connection holding an advisory lock. Dropping it releases the lock as soon as it can; `release` lets go
/// of it and returns once the lock is free.
pub trait LockGuardProtocol: Send + Sync {
    /// Lets go of the lock, returning once another holder can take it.
    fn release(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Completes if the lock is lost while it is held (what held it is gone, so another holder can take it); never,
    /// for a lock that can't be.
    fn lost(&self) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}
