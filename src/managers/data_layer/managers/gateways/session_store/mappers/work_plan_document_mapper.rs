use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::planned_component_document::PlannedComponentDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::work_plan_document::WorkPlanDocument;
use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::work_plan::WorkPlan;

pub struct WorkPlanDocumentMapper;

impl WorkPlanDocumentMapper {
    pub fn to_work_plan_document(work_plan: &WorkPlan, accepted_on: DateTime<Utc>) -> WorkPlanDocument {
        WorkPlanDocument {
            summary: Some(work_plan.summary.clone()),
            assumptions: Some(work_plan.assumptions.clone()),
            components: Some(
                work_plan
                    .components
                    .iter()
                    .map(|planned_component: &PlannedComponent| PlannedComponentDocument {
                        kind: Some(planned_component.kind.clone()),
                        name: Some(planned_component.name.clone()),
                        within: planned_component.within.clone(),
                    })
                    .collect(),
            ),
            steps: Some(work_plan.steps.clone()),
            accepted_on: Some(accepted_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }
}
