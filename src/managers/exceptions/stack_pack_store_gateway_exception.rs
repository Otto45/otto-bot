/// The StackPackStoreGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackPackStoreGatewayException {
    StackPackStoreIo,
    InvalidStackPackManifest,
    /// A file of a stack pack's template can't be copied (it isn't UTF-8 text).
    InvalidStackPackTemplate,
}

impl StackPackStoreGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            StackPackStoreGatewayException::StackPackStoreIo => "StackPackStoreIoException",
            StackPackStoreGatewayException::InvalidStackPackManifest => "InvalidStackPackManifestException",
            StackPackStoreGatewayException::InvalidStackPackTemplate => "InvalidStackPackTemplateException",
        }
    }
}
