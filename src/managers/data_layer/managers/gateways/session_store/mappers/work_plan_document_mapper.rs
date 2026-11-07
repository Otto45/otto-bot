use chrono::{DateTime, SecondsFormat, Utc};

use crate::managers::data_layer::managers::gateways::session_store::resource_models::planned_component_document::PlannedComponentDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::work_package_document::WorkPackageDocument;
use crate::managers::data_layer::managers::gateways::session_store::resource_models::work_plan_document::WorkPlanDocument;
use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::work_package::WorkPackage;
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
            packages: Some(
                work_plan
                    .packages
                    .iter()
                    .map(|work_package: &WorkPackage| WorkPackageDocument {
                        name: Some(work_package.name.clone()),
                        brief: Some(work_package.brief.clone()),
                        owned_paths: Some(work_package.owned_paths.clone()),
                    })
                    .collect(),
            ),
            accepted_on: Some(accepted_on.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }
}
