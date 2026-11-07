/// The StackPackStoreGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackPackStoreGatewayException {
    StackPackStoreIo,
    InvalidStackPackManifest,
}

impl StackPackStoreGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            StackPackStoreGatewayException::StackPackStoreIo => "StackPackStoreIoException",
            StackPackStoreGatewayException::InvalidStackPackManifest => "InvalidStackPackManifestException",
        }
    }
}
