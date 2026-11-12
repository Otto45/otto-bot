use crate::managers::data_layer::managers::gateways::relational_database::resource_models::run_record_row::RunRecordRow;
use crate::managers::models::run_record::RunRecord;
use crate::managers::models::run_status::RunStatus;

pub struct RunRecordMapper;

impl RunRecordMapper {
    /// Expects a row that has already passed ValidatorRunRecordRow: each `expect` states a checked fact, and fails
    /// only if the validator is wrong.
    pub fn to_run_record(run_record_row: RunRecordRow) -> RunRecord {
        let status: String = run_record_row.status.expect("validated by ValidatorRunRecordRow");
        RunRecord {
            status: RunStatus::from_name(&status).expect("validated by ValidatorRunRecordRow"),
            recorded_on: run_record_row.recorded_on.expect("validated by ValidatorRunRecordRow"),
        }
    }
}
