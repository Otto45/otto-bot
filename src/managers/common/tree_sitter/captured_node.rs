use std::ops::Range;

/// A node a tree-sitter query captured: under which capture name, where it is in the source, and on which line (from
/// 1) it starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedNode {
    pub capture_name: String,
    pub byte_range: Range<usize>,
    pub line_number: i64,
}
