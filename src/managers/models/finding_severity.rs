/// How much a review finding matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingSeverity {
    /// Must be fixed before the change ships: a security hole, a broken architecture rule, a bug.
    High,
    /// Worth fixing, but the change can ship without it.
    Medium,
    Low,
}

impl FindingSeverity {
    pub fn name(self) -> &'static str {
        match self {
            FindingSeverity::High => "high",
            FindingSeverity::Medium => "medium",
            FindingSeverity::Low => "low",
        }
    }

    pub fn from_name(name: &str) -> Option<FindingSeverity> {
        match name {
            "high" => Some(FindingSeverity::High),
            "medium" => Some(FindingSeverity::Medium),
            "low" => Some(FindingSeverity::Low),
            _ => None,
        }
    }
}
