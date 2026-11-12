use chrono::{DateTime, Utc};

use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};

pub struct ReviewBaselineQueryBuilder;

impl ReviewBaselineQueryBuilder {
    /// `files` is the baseline's files as JSON text; None says everything the session built has been reviewed.
    pub fn build_append_review_baseline_query(
        session_id: &str,
        files: Option<String>,
        recorded_on: DateTime<Utc>,
    ) -> SqlQuery {
        SqlQuery {
            statement: "INSERT INTO review_baselines (session_id, reviewed, files, recorded_on) VALUES ($1, $2, $3, $4)",
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::OptionalBoolean(Some(files.is_none())),
                SqlParameter::OptionalText(files),
                SqlParameter::Timestamp(recorded_on),
            ],
        }
    }

    // Only the latest row is used: the ones before it are baselines no longer reviewed against.
    pub fn build_get_review_baseline_query(session_id: &str) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "SELECT reviewed, files FROM review_baselines ",
                "WHERE session_id = $1 ORDER BY review_baseline_id DESC LIMIT 1",
            ),
            parameters: vec![SqlParameter::Text(session_id.to_string())],
        }
    }
}
