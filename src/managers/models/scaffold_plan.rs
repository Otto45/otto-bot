use crate::managers::models::scaffold_file::ScaffoldFile;
use crate::managers::models::scaffold_insertion::ScaffoldInsertion;

/// A scaffold rendered for one component: the files to create, the insertions to make, and the file the component it
/// lives within must have (if it lives within one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldPlan {
    pub within_path: Option<String>,
    pub created_files: Vec<ScaffoldFile>,
    pub insertions: Vec<ScaffoldInsertion>,
}
