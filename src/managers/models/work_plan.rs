use crate::managers::models::planned_component::PlannedComponent;

/// The orchestrator's plan for a change too big to make well itself: the components the harness scaffolds, and the
/// steps the implementor takes to build the change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPlan {
    pub summary: String,
    /// What the orchestrator decided on its own where the request left it open.
    pub assumptions: Vec<String>,
    pub components: Vec<PlannedComponent>,
    /// In order; each says what to build, where, and against what.
    pub steps: Vec<String>,
}
