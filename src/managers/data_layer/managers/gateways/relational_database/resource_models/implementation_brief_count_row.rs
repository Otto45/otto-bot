// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct ImplementationBriefCountRow {
    pub implementation_brief_count: Option<i64>,
}
