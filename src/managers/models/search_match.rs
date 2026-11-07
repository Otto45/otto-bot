/// A line of the workspace that a search matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: i64,
    pub line: String,
}
