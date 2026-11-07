/// The OttoBotManager's exceptions: one family of ExceptionKind. Each is raised for something the client can put
/// right, so its message is written for the person using the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OttoBotManagerException {
    SessionNotFound,
    /// A request was submitted while the session's previous one was still running.
    SessionBusy,
}

impl OttoBotManagerException {
    pub fn name(self) -> &'static str {
        match self {
            OttoBotManagerException::SessionNotFound => "SessionNotFoundException",
            OttoBotManagerException::SessionBusy => "SessionBusyException",
        }
    }
}
