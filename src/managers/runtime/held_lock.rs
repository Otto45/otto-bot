/// A lock the lock store holds for a run: on an app, while the run changes its files, or on a session, while its run
/// works on its conversation (compaction included). It is released when it is dropped, so a run that ends, is
/// cancelled or panics lets go of it, and by the operating system when the process exits, however it exits, so a lock
/// is never left behind. The lock store makes it; the manager's run holds it.
pub struct HeldLock {
    file: std::fs::File,
}

impl HeldLock {
    pub fn new(file: std::fs::File) -> Self {
        HeldLock { file }
    }
}

impl Drop for HeldLock {
    fn drop(&mut self) {
        // Closing the file would release it too; unlocking first says so. An unlock that fails is released on close.
        self.file.unlock().ok();
    }
}
