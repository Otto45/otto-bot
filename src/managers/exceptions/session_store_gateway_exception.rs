/// The SessionStoreGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStoreGatewayException {
    SessionStoreIo,
    InvalidConversationItemDocument,
    InvalidProjectMapDocument,
    InvalidSessionDocument,
    InvalidSessionRequestDocument,
    /// A lane name that can't name a file: a bug in whatever named the lane, since names are checked before.
    InvalidLaneName,
}

impl SessionStoreGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            SessionStoreGatewayException::SessionStoreIo => "SessionStoreIoException",
            SessionStoreGatewayException::InvalidConversationItemDocument => "InvalidConversationItemDocumentException",
            SessionStoreGatewayException::InvalidProjectMapDocument => "InvalidProjectMapDocumentException",
            SessionStoreGatewayException::InvalidSessionDocument => "InvalidSessionDocumentException",
            SessionStoreGatewayException::InvalidSessionRequestDocument => "InvalidSessionRequestDocumentException",
            SessionStoreGatewayException::InvalidLaneName => "InvalidLaneNameException",
        }
    }
}
