use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::session::Session;

pub struct SessionQueryBuilder;

impl SessionQueryBuilder {
    // A session is saved once, when it starts; a second save of one id is refused, since rows are never rewritten.
    pub fn build_save_session_query(session: &Session) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "INSERT INTO sessions (session_id, workspace_directory, service_tier, started_on) ",
                "VALUES ($1, $2, $3, $4)",
            ),
            parameters: vec![
                SqlParameter::Text(session.id.clone()),
                SqlParameter::Text(session.workspace_directory.to_string_lossy().into_owned()),
                SqlParameter::Text(session.service_tier.name().to_string()),
                SqlParameter::Timestamp(session.started_on),
            ],
        }
    }

    pub fn build_get_session_query(session_id: &str) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "SELECT session_id, workspace_directory, service_tier, started_on ",
                "FROM sessions WHERE session_id = $1",
            ),
            parameters: vec![SqlParameter::Text(session_id.to_string())],
        }
    }

    pub fn build_get_sessions_query() -> SqlQuery {
        SqlQuery {
            statement: "SELECT session_id, workspace_directory, service_tier, started_on FROM sessions",
            parameters: Vec::new(),
        }
    }
}
