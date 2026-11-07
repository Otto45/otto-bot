/// A check every change to an app of the stack passes: a command run from the app's root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackCheck {
    pub name: String,
    /// The program, then its arguments. Empty for a check the pack keeps a place for but doesn't run yet.
    pub command: Vec<String>,
}
