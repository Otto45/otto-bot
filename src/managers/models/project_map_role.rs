/// What a project map kept by a session is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectMapRole {
    /// The map the session's requests open with, until it is rebuilt.
    Layer,
    /// A map a run started with, kept so the next run tells only what changed since.
    Seen,
}

impl ProjectMapRole {
    pub fn name(self) -> &'static str {
        match self {
            ProjectMapRole::Layer => "layer",
            ProjectMapRole::Seen => "seen",
        }
    }
}
