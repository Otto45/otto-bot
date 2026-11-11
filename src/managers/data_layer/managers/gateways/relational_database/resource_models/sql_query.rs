use chrono::{DateTime, Utc};

/// A value bound to a statement's parameter ($1, $2, ...). The gateway is private, so the compiler refuses a variant
/// no query uses: add one with the first query that binds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlParameter {
    Text(String),
    OptionalText(Option<String>),
    TextArray(Vec<String>),
    OptionalTextArray(Option<Vec<String>>),
    BigInt(i64),
    OptionalBoolean(Option<bool>),
    Timestamp(DateTime<Utc>),
}

// Values travel as bound parameters, never formatted into the statement. The statement is `&'static str`: the
// driver (sqlx) only takes a statement it can see was written in the code, so one formatted at run time does not
// compile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlQuery {
    pub statement: &'static str,
    pub parameters: Vec<SqlParameter>,
}
