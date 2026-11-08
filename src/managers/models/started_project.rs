use std::path::PathBuf;

use crate::managers::models::app_name::AppName;

/// An app started from a stack pack's template: where it is, how many files it began with, and whether the pack's
/// formatters ran over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartedProject {
    pub stack_name: String,
    pub app_name: AppName,
    pub workspace_directory: PathBuf,
    pub file_count: usize,
    /// Whether every one of the pack's formatters ran and succeeded on it.
    pub formatted: bool,
}
