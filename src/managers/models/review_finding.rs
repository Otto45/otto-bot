use crate::managers::models::finding_severity::FindingSeverity;

/// One thing the reviewer found wrong with a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewFinding {
    pub severity: FindingSeverity,
    /// The file it is in.
    pub path: String,
    pub problem: String,
    /// What would put it right.
    pub fix: String,
}
