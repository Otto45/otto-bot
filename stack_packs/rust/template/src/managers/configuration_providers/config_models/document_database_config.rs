#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentDatabaseConfig {
    pub connection_string: Option<String>,
}
