use chrono::{DateTime, Utc};

use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::implementation_brief::ImplementationBrief;

pub struct ImplementationBriefQueryBuilder;

impl ImplementationBriefQueryBuilder {
    pub fn build_append_implementation_brief_query(
        session_id: &str,
        implementation_brief: &ImplementationBrief,
        accepted_on: DateTime<Utc>,
    ) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "INSERT INTO implementation_briefs (session_id, task, decisions, done_when, files, accepted_on) ",
                "VALUES ($1, $2, $3, $4, $5, $6)",
            ),
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::Text(implementation_brief.task.clone()),
                SqlParameter::TextArray(implementation_brief.decisions.clone()),
                SqlParameter::Text(implementation_brief.done_when.clone()),
                SqlParameter::TextArray(implementation_brief.files.clone()),
                SqlParameter::Timestamp(accepted_on),
            ],
        }
    }

    pub fn build_count_implementation_briefs_query(session_id: &str) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "SELECT count(*) AS implementation_brief_count FROM implementation_briefs ",
                "WHERE session_id = $1",
            ),
            parameters: vec![SqlParameter::Text(session_id.to_string())],
        }
    }
}
