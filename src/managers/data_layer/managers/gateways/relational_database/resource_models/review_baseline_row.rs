// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it. `files` is JSON text: an array of ReviewBaselineFileResource.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct ReviewBaselineRow {
    pub reviewed: Option<bool>,
    pub files: Option<String>,
}
