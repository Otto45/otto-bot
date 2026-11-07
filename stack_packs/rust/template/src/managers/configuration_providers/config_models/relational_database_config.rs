#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelationalDatabaseConfig {
    pub connection_string: Option<String>,
}
