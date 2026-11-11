use std::sync::Arc;

use crate::managers::data_layer::managers::gateways::lock_store::lock_store_gateway::LockStoreGateway;
use crate::managers::data_layer::managers::gateways::relational_database::relational_database_gateway::RelationalDatabaseGateway;
use crate::managers::data_layer::managers::gateways::session_store::session_store_gateway::SessionStoreGateway;

/// Where the DataManager keeps the sessions and their locks, which configuration decides: in folders and file locks
/// under otto-bot's home folder, or in a Postgres database (`OTTO_BOT_RELATIONAL_DATABASE_CONNECTION_STRING`), which
/// processes on several machines can share. Only the gateways of the one configured are built.
pub enum SessionStorage {
    Files {
        session_store_gateway: Arc<SessionStoreGateway>,
        lock_store_gateway: Arc<LockStoreGateway>,
    },
    RelationalDatabase {
        relational_database_gateway: Arc<RelationalDatabaseGateway>,
    },
}
