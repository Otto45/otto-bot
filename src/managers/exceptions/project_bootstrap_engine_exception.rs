/// The ProjectBootstrapEngine's exceptions: one family of ExceptionKind. Each is raised for something the client can
/// put right, so its message is written for the person using the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectBootstrapEngineException {
    /// The app's name can't be made into the names the template's are replaced with.
    InvalidAppName,
    /// No stack pack has the stack's name.
    UnknownStack,
    /// The stack's pack has no template to start an app from.
    StackWithoutTemplate,
}

impl ProjectBootstrapEngineException {
    pub fn name(self) -> &'static str {
        match self {
            ProjectBootstrapEngineException::InvalidAppName => "InvalidAppNameException",
            ProjectBootstrapEngineException::UnknownStack => "UnknownStackException",
            ProjectBootstrapEngineException::StackWithoutTemplate => "StackWithoutTemplateException",
        }
    }
}
