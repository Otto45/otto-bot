/// A function tool call, read: what the model asked the harness to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRequest {
    ReadFiles { paths: Vec<String> },
    ListFiles { directory: String },
    Search { pattern: String, path_glob: Option<String> },
    RunChecks,
}
