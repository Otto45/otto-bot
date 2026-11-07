use crate::managers::models::patch_operation_kind::PatchOperationKind;

/// One file operation the model asked for through apply_patch. `diff` is a V4A diff: a new file's lines for
/// create_file, the hunks to apply for update_file, and None for delete_file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchOperation {
    pub kind: PatchOperationKind,
    pub path: String,
    pub diff: Option<String>,
}
