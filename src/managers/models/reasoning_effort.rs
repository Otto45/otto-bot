/// How hard the model reasons before it answers. A lane's effort is set by a configuration update in its conversation,
/// never by the request's own setting, which would change the cached prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl ReasoningEffort {
    pub fn name(self) -> &'static str {
        match self {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
            ReasoningEffort::Xhigh => "xhigh",
            ReasoningEffort::Max => "max",
        }
    }

    pub fn from_name(name: &str) -> Option<ReasoningEffort> {
        match name {
            "low" => Some(ReasoningEffort::Low),
            "medium" => Some(ReasoningEffort::Medium),
            "high" => Some(ReasoningEffort::High),
            "xhigh" => Some(ReasoningEffort::Xhigh),
            "max" => Some(ReasoningEffort::Max),
            _ => None,
        }
    }
}
