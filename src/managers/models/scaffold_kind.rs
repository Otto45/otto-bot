use crate::managers::models::scaffold_file::ScaffoldFile;
use crate::managers::models::scaffold_insertion::ScaffoldInsertion;

/// A kind of component a stack pack can scaffold: the files it creates and the insertions that wire it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldKind {
    pub name: String,
    pub description: String,
    /// For a kind that lives inside another component (an engine's processor): a file that component has, which must
    /// exist (a template using {within}). None for a kind that stands on its own.
    pub within_path: Option<String>,
    pub files: Vec<ScaffoldFile>,
    pub insertions: Vec<ScaffoldInsertion>,
}
