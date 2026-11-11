use chrono::{DateTime, Utc};

use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::project_map_role::ProjectMapRole;

pub struct ProjectMapQueryBuilder;

impl ProjectMapQueryBuilder {
    pub fn build_append_project_map_query(
        session_id: &str,
        project_map: &str,
        project_map_role: ProjectMapRole,
        built_on: DateTime<Utc>,
    ) -> SqlQuery {
        SqlQuery {
            statement: "INSERT INTO project_maps (session_id, project_map, role, built_on) VALUES ($1, $2, $3, $4)",
            parameters: vec![
                SqlParameter::Text(session_id.to_string()),
                SqlParameter::Text(project_map.to_string()),
                SqlParameter::Text(project_map_role.name().to_string()),
                SqlParameter::Timestamp(built_on),
            ],
        }
    }

    // Only the two maps the session uses are read, not every map it kept: the latest 'layer' one (its requests open
    // with it) and the latest of either role (the one the last run started with). One row, whatever the session has.
    pub fn build_get_session_project_maps_query(session_id: &str) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "SELECT ",
                "(SELECT project_map FROM project_maps WHERE session_id = $1 AND role = 'layer' ",
                "ORDER BY project_map_id DESC LIMIT 1) AS layer_map, ",
                "(SELECT project_map FROM project_maps WHERE session_id = $1 ",
                "ORDER BY project_map_id DESC LIMIT 1) AS latest_map",
            ),
            parameters: vec![SqlParameter::Text(session_id.to_string())],
        }
    }
}
