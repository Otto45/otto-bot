/// The OttoBotManager's exceptions: one family of ExceptionKind. Each but `AppLockLost` is raised for something the
/// client can put right, so its message is written for the person using the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OttoBotManagerException {
    SessionNotFound,
    /// A request was submitted while the session's previous one was still running.
    SessionBusy,
    /// No app of the name is kept where apps started by name are.
    AppNotFound,
    /// An app of the name is already kept there.
    AppAlreadyExists,
    /// A request was submitted while another session of the same app (workspace) was running one: two runs would
    /// change the same files.
    AppBusy,
    /// The app's lock was lost while a run held it (the database connection holding it ended), so the run was
    /// stopped: another run could have started on the same files.
    AppLockLost,
}

impl OttoBotManagerException {
    pub fn name(self) -> &'static str {
        match self {
            OttoBotManagerException::SessionNotFound => "SessionNotFoundException",
            OttoBotManagerException::SessionBusy => "SessionBusyException",
            OttoBotManagerException::AppNotFound => "AppNotFoundException",
            OttoBotManagerException::AppAlreadyExists => "AppAlreadyExistsException",
            OttoBotManagerException::AppBusy => "AppBusyException",
            OttoBotManagerException::AppLockLost => "AppLockLostException",
        }
    }
}
