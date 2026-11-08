use std::path::PathBuf;

use crate::managers::models::stack_pack_check::StackPackCheck;
use crate::managers::models::stack_pack_format::StackPackFormat;
use crate::managers::models::stack_pack_lint::StackPackLint;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;

/// What otto-bot knows about building apps in one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPack {
    pub name: String,
    /// The tree-sitter grammar built into otto-bot that parses the stack's source files; None for a pack that neither
    /// maps nor lints projects.
    pub language: Option<String>,
    /// A workspace is an app of this stack when its root has all of these files.
    pub detect_files: Vec<String>,
    /// The architecture's rules for the stack's apps.
    pub rules_path: PathBuf,
    pub checks: Vec<StackPackCheck>,
    /// None for a pack with no formatter.
    pub format: Option<StackPackFormat>,
    /// None for a pack that doesn't map its projects.
    pub project_map: Option<StackPackProjectMap>,
    /// None for a pack with no architecture lint.
    pub lint: Option<StackPackLint>,
}
