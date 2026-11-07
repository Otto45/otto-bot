#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallKind {
    Function,
    ApplyPatch,
}

impl ToolCallKind {
    pub fn name(self) -> &'static str {
        match self {
            ToolCallKind::Function => "function",
            ToolCallKind::ApplyPatch => "apply_patch",
        }
    }

    pub fn from_name(name: &str) -> Option<ToolCallKind> {
        match name {
            "function" => Some(ToolCallKind::Function),
            "apply_patch" => Some(ToolCallKind::ApplyPatch),
            _ => None,
        }
    }
}
