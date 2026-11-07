/// How a stack pack maps a project: which tree-sitter grammar parses its source files, which files those are (globs
/// relative to the workspace), and the query that picks each file's public items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackProjectMap {
    pub language: String,
    pub source_globs: Vec<String>,
    pub query: String,
}
