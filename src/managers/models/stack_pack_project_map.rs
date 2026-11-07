/// How a stack pack maps a project: which files are source files (globs relative to the workspace), and the query
/// that picks each one's public items. The pack's grammar parses them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackProjectMap {
    pub source_globs: Vec<String>,
    pub query: String,
}
