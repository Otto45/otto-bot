/// The orchestrator's plan for a change too big to make well itself: what the change is, and the steps the
/// implementor takes to build it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPlan {
    pub summary: String,
    /// What the orchestrator decided on its own where the request left it open.
    pub assumptions: Vec<String>,
    /// In order; each says what to build, where, and against what.
    pub steps: Vec<String>,
}
