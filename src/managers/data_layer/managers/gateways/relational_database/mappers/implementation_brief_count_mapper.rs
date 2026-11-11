use crate::managers::data_layer::managers::gateways::relational_database::resource_models::implementation_brief_count_row::ImplementationBriefCountRow;

pub struct ImplementationBriefCountMapper;

impl ImplementationBriefCountMapper {
    /// Expects a row that has already passed ValidatorImplementationBriefCountRow: each `expect` states a checked
    /// fact, and fails only if the validator is wrong.
    pub fn to_implementation_brief_count(implementation_brief_count_row: ImplementationBriefCountRow) -> usize {
        let implementation_brief_count: i64 = implementation_brief_count_row
            .implementation_brief_count
            .expect("validated by ValidatorImplementationBriefCountRow");
        usize::try_from(implementation_brief_count).expect("validated by ValidatorImplementationBriefCountRow")
    }
}
