/// The Postgres database otto-bot keeps its sessions and their locks in. Its connection string holds the database's
/// password, so it is never logged or put in an exception's context.
#[derive(Clone, PartialEq, Eq)]
pub struct RelationalDatabaseSettings {
    pub connection_string: String,
}

// Written by hand so that a settings value printed with `{:?}` never shows the password.
impl std::fmt::Debug for RelationalDatabaseSettings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalDatabaseSettings")
            .field("connection_string", &"<redacted>")
            .finish()
    }
}
