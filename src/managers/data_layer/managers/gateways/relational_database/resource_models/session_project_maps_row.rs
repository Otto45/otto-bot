// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
// The session's latest 'layer' map and its latest map of either role; both are None before its first run.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct SessionProjectMapsRow {
    pub layer_map: Option<String>,
    pub latest_map: Option<String>,
}
