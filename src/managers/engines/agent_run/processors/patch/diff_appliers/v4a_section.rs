use crate::managers::engines::agent_run::processors::patch::diff_appliers::v4a_chunk::V4aChunk;

/// One section of a V4A update diff, read: the lines it expects to find (context and deleted lines, in order), its
/// changes (each chunk's orig_index relative to the start of that context), where reading stopped, and whether the
/// section is anchored at the end of the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V4aSection {
    pub next_context: Vec<String>,
    pub section_chunks: Vec<V4aChunk>,
    pub end_index: usize,
    pub eof: bool,
}
