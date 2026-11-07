/// One change a V4A hunk makes: at `orig_index` (a line of the original), `del_lines` are replaced by `ins_lines`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V4aChunk {
    pub orig_index: usize,
    pub del_lines: Vec<String>,
    pub ins_lines: Vec<String>,
}
