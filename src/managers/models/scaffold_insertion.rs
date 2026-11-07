/// Text a scaffold adds to a file that exists: at the start of the line holding `before` (an anchor), or at the end
/// when there is none. A template until rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldInsertion {
    pub path: String,
    pub before: Option<String>,
    pub text: String,
}
