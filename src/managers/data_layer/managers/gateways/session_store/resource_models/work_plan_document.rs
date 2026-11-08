use serde::{Deserialize, Serialize};

// One line of a session's plans.jsonl: a plan the harness accepted, in the order they were accepted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkPlanDocument {
    pub summary: Option<String>,
    pub assumptions: Option<Vec<String>>,
    pub steps: Option<Vec<String>>,
    pub accepted_on: Option<String>,
}
