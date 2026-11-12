use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

/// A command's process group while the command runs: the command and every process it started (a compiler under a
/// build tool, the app's own binary). Dropped while the command is still running (it took too long, or its run was
/// cancelled), it stops them all, so none goes on changing the workspace after its run has moved on. Once the
/// command has ended on its own, `release` lets go of the group without stopping anything: its id may be another
/// group's by then.
pub struct ProcessGroupGuard {
    process_group_id: Option<Pid>,
}

impl ProcessGroupGuard {
    /// `process_id` is the command's, which leads its own process group; None if it had already ended.
    pub fn new(process_id: Option<u32>) -> Self {
        ProcessGroupGuard {
            process_group_id: process_id
                .and_then(|process_id: u32| i32::try_from(process_id).ok())
                .map(Pid::from_raw),
        }
    }

    pub fn release(mut self) {
        self.process_group_id = None;
    }
}

impl Drop for ProcessGroupGuard {
    fn drop(&mut self) {
        if let Some(process_group_id) = self.process_group_id.take() {
            // A group that is already gone has nothing to stop.
            killpg(process_group_id, Signal::SIGKILL).ok();
        }
    }
}
