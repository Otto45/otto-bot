use std::path::PathBuf;

use crate::managers::models::stack_pack_check::StackPackCheck;

/// What otto-bot knows about building apps in one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPack {
    pub name: String,
    /// A workspace is an app of this stack when its root has all of these files.
    pub detect_files: Vec<String>,
    /// The architecture's rules for the stack's apps.
    pub rules_path: PathBuf,
    pub checks: Vec<StackPackCheck>,
}
