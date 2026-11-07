/// The RelationalDatabaseGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalDatabaseGatewayException {
    CustomerNotFound,
    InvalidPostgresCustomerRow,
    PostgresQuery,
}

impl RelationalDatabaseGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            RelationalDatabaseGatewayException::CustomerNotFound => "CustomerNotFoundException",
            RelationalDatabaseGatewayException::InvalidPostgresCustomerRow => "InvalidPostgresCustomerRowException",
            RelationalDatabaseGatewayException::PostgresQuery => "PostgresQueryException",
        }
    }
}
