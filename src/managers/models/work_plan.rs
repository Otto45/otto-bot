use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::work_package::WorkPackage;

/// The orchestrator's plan for a change too big to make alone: the components to scaffold, and the packages workers build at
/// the same time, each in files no other package touches. The orchestrator writes the contracts (models, exceptions,
/// signatures) between the two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPlan {
    pub summary: String,
    /// What the orchestrator decided on its own where the request left it open.
    pub assumptions: Vec<String>,
    pub components: Vec<PlannedComponent>,
    pub packages: Vec<WorkPackage>,
}
