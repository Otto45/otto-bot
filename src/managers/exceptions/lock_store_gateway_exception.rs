/// The LockStoreGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockStoreGatewayException {
    LockStoreIo,
    /// A session id that can't name a lock file: a bug in whatever passed it, since ids are checked before.
    InvalidLockName,
}

impl LockStoreGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            LockStoreGatewayException::LockStoreIo => "LockStoreIoException",
            LockStoreGatewayException::InvalidLockName => "InvalidLockNameException",
        }
    }
}
