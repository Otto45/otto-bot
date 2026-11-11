use std::future::Future;
use std::pin::Pin;

use sqlx::Connection;
use sqlx::postgres::PgConnection;

use crate::managers::data_layer::managers::gateways::relational_database::query_builders::advisory_lock_query_builder::AdvisoryLockQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::relational_database_gateway::RelationalDatabaseGateway;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::SqlQuery;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::runtime::lock_guard_protocol::LockGuardProtocol;

/// A Postgres advisory lock, held by the connection that took it: a connection of its own, taken out of the pool, so
/// the lock is never handed to another statement. Postgres releases a session's advisory locks when its connection
/// ends, however it ends (the process exits or is killed), so a lock is never left behind; `release` unlocks it and
/// waits for Postgres to say so.
pub struct AdvisoryLockGuard {
    connection: Option<PgConnection>,
    lock_key: i64,
}

impl AdvisoryLockGuard {
    pub fn new(connection: PgConnection, lock_key: i64) -> Self {
        AdvisoryLockGuard {
            connection: Some(connection),
            lock_key,
        }
    }
}

impl LockGuardProtocol for AdvisoryLockGuard {
    fn release(mut self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let connection: Option<PgConnection> = self.connection.take();
        let lock_key: i64 = self.lock_key;
        Box::pin(async move {
            let Some(mut connection) = connection else {
                return;
            };
            let sql_query: SqlQuery = AdvisoryLockQueryBuilder::build_unlock_query(lock_key);
            // An unlock that fails still lets go: closing the connection ends its session, and its locks with it.
            if let Err(exception) = RelationalDatabaseGateway::bind(&sql_query)
                .execute(&mut connection)
                .await
            {
                ApplicationLogger::log_exception(
                    &RelationalDatabaseGateway::to_query_exception(
                        exception,
                        &sql_query,
                        "releasing a lock",
                        &ContextualData::new().with("lock_key", lock_key),
                    ),
                    LogLevel::Warning,
                );
            }
            connection.close().await.ok();
        })
    }
}

impl Drop for AdvisoryLockGuard {
    // Dropped without being released (a run cancelled or panicking): the connection is closed in the background, so
    // Postgres ends its session cleanly and frees the lock a moment later; with no runtime left to close it on, the
    // connection is dropped and Postgres frees the lock when it sees the connection has gone.
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            runtime.spawn(async move {
                connection.close().await.ok();
            });
        }
    }
}
