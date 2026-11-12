/// The WorkspaceGateway's exceptions: one family of ExceptionKind. Except WorkspaceIo, each is something the agent
/// (or, for InvalidWorkspace, the client) asked for that can't be done, and its message says why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceGatewayException {
    InvalidWorkspace,
    /// A folder that was to be made new is already there.
    WorkspaceAlreadyExists,
    PathOutsideWorkspace,
    ProtectedFile,
    FileNotFound,
    FileAlreadyExists,
    NotTextFile,
    InvalidSearchPattern,
    WorkspaceIo,
}

impl WorkspaceGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            WorkspaceGatewayException::InvalidWorkspace => "InvalidWorkspaceException",
            WorkspaceGatewayException::WorkspaceAlreadyExists => "WorkspaceAlreadyExistsException",
            WorkspaceGatewayException::PathOutsideWorkspace => "PathOutsideWorkspaceException",
            WorkspaceGatewayException::ProtectedFile => "ProtectedFileException",
            WorkspaceGatewayException::FileNotFound => "FileNotFoundException",
            WorkspaceGatewayException::FileAlreadyExists => "FileAlreadyExistsException",
            WorkspaceGatewayException::NotTextFile => "NotTextFileException",
            WorkspaceGatewayException::InvalidSearchPattern => "InvalidSearchPatternException",
            WorkspaceGatewayException::WorkspaceIo => "WorkspaceIoException",
        }
    }
}
