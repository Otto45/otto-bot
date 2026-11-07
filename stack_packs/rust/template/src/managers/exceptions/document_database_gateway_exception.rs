/// The DocumentDatabaseGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentDatabaseGatewayException {
    InvalidAuditEventDocument,
}

impl DocumentDatabaseGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            DocumentDatabaseGatewayException::InvalidAuditEventDocument => "InvalidAuditEventDocumentException",
        }
    }
}
