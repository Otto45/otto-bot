use serde::{Deserialize, Serialize};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::planned_component_document::PlannedComponentDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::work_package_document::WorkPackageDocument;

// One line of a session's plans.jsonl: a plan the harness accepted, in the order they were accepted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkPlanDocument {
    pub summary: Option<String>,
    pub assumptions: Option<Vec<String>>,
    pub components: Option<Vec<PlannedComponentDocument>>,
    pub packages: Option<Vec<WorkPackageDocument>>,
    pub accepted_on: Option<String>,
}
