#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchOperationKind {
    CreateFile,
    UpdateFile,
    DeleteFile,
}

impl PatchOperationKind {
    pub fn name(self) -> &'static str {
        match self {
            PatchOperationKind::CreateFile => "create_file",
            PatchOperationKind::UpdateFile => "update_file",
            PatchOperationKind::DeleteFile => "delete_file",
        }
    }
}
