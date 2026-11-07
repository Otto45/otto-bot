/// The SessionStoreGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStoreGatewayException {
    SessionStoreIo,
    InvalidConversationItemDocument,
}

impl SessionStoreGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            SessionStoreGatewayException::SessionStoreIo => "SessionStoreIoException",
            SessionStoreGatewayException::InvalidConversationItemDocument => "InvalidConversationItemDocumentException",
        }
    }
}
