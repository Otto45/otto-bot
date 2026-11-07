use sqlx::postgres::{PgArguments, PgPool, PgRow};
use sqlx::{FromRow, Postgres};

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::data_layer::managers::gateways::relational_database::mappers::customer_mapper::CustomerMapper;
use crate::managers::data_layer::managers::gateways::relational_database::query_builders::customer_query_builder::CustomerQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::customer_row::CustomerRow;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::data_layer::managers::gateways::relational_database::validators::validator_customer_row::ValidatorCustomerRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::managers::models::customer::Customer;

type BoundQuery<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;

pub struct RelationalDatabaseGateway {
    // The ServiceLocator builds the pool without connecting (its methods are synchronous); the pool opens its first
    // connection when the gateway first sends a statement, and opens it only once, however many statements are sent
    // at the same time. The schema is not the gateway's: it is migrated outside the application (migrations/,
    // applied by dbmate).
    pool: PgPool,
}

impl RelationalDatabaseGateway {
    pub fn new(pool: PgPool) -> Self {
        RelationalDatabaseGateway { pool }
    }

    pub async fn get_customer(&self, customer_id: &str) -> ApplicationResult<Customer> {
        let context: ContextualData = ContextualData::new().with("customer_id", customer_id);
        let sql_query: SqlQuery = CustomerQueryBuilder::build_get_customer_query(customer_id);

        let rows: Vec<PgRow> = RelationalDatabaseGateway::bind(&sql_query)
            .fetch_all(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(exception, &sql_query, "retrieving a customer", &context)
            })?;

        let Some(row) = rows.first() else {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::CustomerNotFound,
                "The customer was not found",
                context,
            ));
        };

        let customer_row: CustomerRow = CustomerRow::from_row(row).map_err(|exception: sqlx::Error| {
            let errors: String = ValidationErrorFormatter::format_row(&exception, "row");
            ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresCustomerRow,
                "Customer row from the Postgres customers table could not be deserialized",
                context.clone().with("errors", errors),
            )
            .with_source(exception)
        })?;

        ValidatorCustomerRow::validate_customer_row(&customer_row)?;
        Ok(CustomerMapper::to_customer(customer_row))
    }

    pub async fn save_customer(&self, customer: &Customer) -> ApplicationResult<()> {
        let context: ContextualData = ContextualData::new().with("customer_id", customer.id.as_str());
        let sql_query: SqlQuery = CustomerQueryBuilder::build_save_customer_query(customer);

        RelationalDatabaseGateway::bind(&sql_query)
            .execute(&self.pool)
            .await
            .map_err(|exception: sqlx::Error| {
                RelationalDatabaseGateway::to_query_exception(exception, &sql_query, "saving a customer", &context)
            })?;
        Ok(())
    }

    /// Closes the pool's connections. A pool that never connected has none, and closing it does not connect.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    fn bind(sql_query: &SqlQuery) -> BoundQuery<'_> {
        let mut query: BoundQuery<'_> = sqlx::query(sql_query.statement);
        for parameter in &sql_query.parameters {
            query = match parameter {
                SqlParameter::Text(value) => query.bind(value.as_str()),
            };
        }
        query
    }

    fn to_query_exception(
        driver_error: sqlx::Error,
        sql_query: &SqlQuery,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        // Parameter values stay out of the context: they can hold customer data.
        let mut exception_context: ContextualData = context
            .clone()
            .with(
                "error_type",
                RelationalDatabaseGateway::describe_error_type(&driver_error),
            )
            .with("postgres_error_message", driver_error.to_string())
            .with("sql_statement", sql_query.statement);

        // Only set when the error comes from Postgres itself (e.g. not for a refused connection): its SQLSTATE code.
        if let Some(postgres_error_code) = driver_error
            .as_database_error()
            .and_then(|database_error: &dyn sqlx::error::DatabaseError| database_error.code())
        {
            exception_context.insert("postgres_error_code", postgres_error_code.into_owned());
        }

        ApplicationBaseException::new(
            RelationalDatabaseGatewayException::PostgresQuery,
            format!("Postgres query failed while {operation_description}"),
            exception_context,
        )
        .with_source(driver_error)
    }

    // sqlx has one error type; what kind of failure it was is which of its variants it is.
    fn describe_error_type(driver_error: &sqlx::Error) -> &'static str {
        match driver_error {
            sqlx::Error::Configuration(_) => "ConfigurationError",
            sqlx::Error::Database(_) => "DatabaseError",
            sqlx::Error::Io(_) => "IoError",
            sqlx::Error::Tls(_) => "TlsError",
            sqlx::Error::Protocol(_) => "ProtocolError",
            sqlx::Error::PoolTimedOut => "PoolTimedOutError",
            sqlx::Error::PoolClosed => "PoolClosedError",
            sqlx::Error::WorkerCrashed => "WorkerCrashedError",
            _ => "Error",
        }
    }
}
