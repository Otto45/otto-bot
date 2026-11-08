use crate::managers::models::review_finding::ReviewFinding;

/// The reviewer's verdict on a change: what it found wrong, most severe first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Review {
    pub summary: String,
    pub findings: Vec<ReviewFinding>,
}
