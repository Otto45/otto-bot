use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::run_record::RunRecord;

pub struct RunRecordQueryBuilder;

impl RunRecordQueryBuilder {
    pub fn build_append_run_record_query(session_id: &str, run_record: &RunRecord) -> SqlQuery {
        SqlQuery {
            statement: "INSERT INTO run_records (session_id, status, recorded_on) VALUES ($1, $2, $3)",
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::Text(run_record.status.name().to_string()),
                SqlParameter::Timestamp(run_record.recorded_on),
            ],
        }
    }

    // Oldest first: the identity column numbers them in the order they were recorded.
    pub fn build_get_run_records_query(session_id: &str) -> SqlQuery {
        SqlQuery {
            statement: "SELECT status, recorded_on FROM run_records WHERE session_id = $1 ORDER BY run_record_id",
            parameters: vec![SqlParameter::Text(session_id.to_string())],
        }
    }
}
