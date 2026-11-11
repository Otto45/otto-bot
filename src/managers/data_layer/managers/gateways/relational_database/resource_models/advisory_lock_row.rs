// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
// Whether pg_try_advisory_lock took the lock.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct AdvisoryLockRow {
    pub locked: Option<bool>,
}
