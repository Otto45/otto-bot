// A row as Postgres returns it: every column optional, so that any row deserializes and the validator can say
// what is wrong with it.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct CustomerRow {
    pub customer_id: Option<String>,
    pub full_name: Option<String>,
    pub email_address: Option<String>,
}
