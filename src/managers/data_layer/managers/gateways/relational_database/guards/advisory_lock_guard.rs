use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use sqlx::Connection;
use sqlx::postgres::PgConnection;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::managers::data_layer::managers::gateways::relational_database::query_builders::advisory_lock_query_builder::AdvisoryLockQueryBuilder;
use crate::managers::data_layer::managers::gateways::relational_database::relational_database_gateway::RelationalDatabaseGateway;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::SqlQuery;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::runtime::lock_guard_protocol::LockGuardProtocol;

// How often a held lock's connection is asked whether it is still there, and how long it has to answer. Asking also
// keeps the connection in use, so nothing between otto-bot and the database closes it for being idle.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);
// How long letting go of a lock may take: a connection that no longer answers has lost its lock already.
const RELEASE_TIMEOUT: Duration = Duration::from_secs(5);

type LockConnection = Arc<tokio::sync::Mutex<Option<PgConnection>>>;

/// A Postgres advisory lock, held by the connection that took it: a connection of its own, taken out of the pool, so
/// the lock is never handed to another statement. Postgres releases a session's advisory locks when its connection
/// ends, however it ends (the process exits or is killed), so a lock is never left behind; `release` unlocks it and
/// waits for Postgres to say so. The same holds the other way: a connection that ends while the lock is held (the
/// database restarted, something between closed it) loses the lock, which a heartbeat on the connection finds out,
/// and `lost` tells.
pub struct AdvisoryLockGuard {
    connection: LockConnection,
    lock_key: i64,
    heartbeat: JoinHandle<()>,
    lost: watch::Receiver<bool>,
}

impl AdvisoryLockGuard {
    /// Made inside a Tokio runtime: it starts the lock's heartbeat.
    pub fn new(connection: PgConnection, lock_key: i64) -> Self {
        let connection: LockConnection = Arc::new(tokio::sync::Mutex::new(Some(connection)));
        let (lost_sender, lost): (watch::Sender<bool>, watch::Receiver<bool>) = watch::channel(false);
        AdvisoryLockGuard {
            heartbeat: tokio::spawn(AdvisoryLockGuard::beat(connection.clone(), lock_key, lost_sender)),
            connection,
            lock_key,
            lost,
        }
    }

    // Asks the connection, every so often, whether it is still there, until the lock is let go of; a connection that
    // isn't has lost its lock, which is logged and told.
    async fn beat(connection: LockConnection, lock_key: i64, lost_sender: watch::Sender<bool>) {
        loop {
            tokio::time::sleep(HEARTBEAT_INTERVAL).await;
            let mut held_connection: tokio::sync::MutexGuard<'_, Option<PgConnection>> = connection.lock().await;
            let Some(held_connection) = held_connection.as_mut() else {
                return;
            };
            let is_alive: bool = matches!(
                tokio::time::timeout(HEARTBEAT_TIMEOUT, held_connection.ping()).await,
                Ok(Ok(()))
            );
            if !is_alive {
                ApplicationLogger::error(
                    "A lock's database connection was lost, and the lock with it",
                    ContextualData::new().with("lock_key", lock_key),
                    None,
                );
                lost_sender.send(true).ok();
                return;
            }
        }
    }

    // Unlocks, then closes the connection. An unlock that fails still lets go: closing the connection ends its
    // session, and its locks with it.
    async fn unlock(mut connection: PgConnection, lock_key: i64) {
        let sql_query: SqlQuery = AdvisoryLockQueryBuilder::build_unlock_query(lock_key);
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
    }
}

impl LockGuardProtocol for AdvisoryLockGuard {
    fn release(self: Box<Self>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.heartbeat.abort();
        let connection: LockConnection = self.connection.clone();
        let lock_key: i64 = self.lock_key;
        Box::pin(async move {
            let Some(connection) = connection.lock().await.take() else {
                return;
            };
            // A connection that doesn't answer in time is dropped: it has lost its lock already, or loses it now.
            if tokio::time::timeout(RELEASE_TIMEOUT, AdvisoryLockGuard::unlock(connection, lock_key))
                .await
                .is_err()
            {
                ApplicationLogger::warning(
                    "A lock's database connection did not answer while the lock was released; it was dropped",
                    ContextualData::new().with("lock_key", lock_key),
                    None,
                );
            }
        })
    }

    fn lost(&self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut lost: watch::Receiver<bool> = self.lost.clone();
        Box::pin(async move {
            // A heartbeat that ended without losing the lock (the lock was let go of) never tells.
            if lost.wait_for(|lost: &bool| *lost).await.is_err() {
                std::future::pending::<()>().await;
            }
        })
    }
}

impl Drop for AdvisoryLockGuard {
    // Dropped without being released (a run cancelled or panicking), or after it: a connection still held is closed
    // in the background, so Postgres ends its session cleanly and frees the lock a moment later; with no runtime left
    // to close it on, the connection is dropped and Postgres frees the lock when it sees the connection has gone.
    fn drop(&mut self) {
        self.heartbeat.abort();
        let connection: LockConnection = self.connection.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                if let Some(connection) = connection.lock().await.take() {
                    connection.close().await.ok();
                }
            });
        }
    }
}
