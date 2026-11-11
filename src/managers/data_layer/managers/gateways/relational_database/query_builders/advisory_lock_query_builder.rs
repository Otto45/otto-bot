use std::path::Path;

use sha2::{Digest, Sha256};

use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};

pub struct AdvisoryLockQueryBuilder;

impl AdvisoryLockQueryBuilder {
    /// The key of an app's lock: its resolved folder's path, so every name for the folder is one lock, and every
    /// process that has the folder at the same path takes the same lock.
    pub fn get_app_lock_key(workspace_directory: &Path) -> i64 {
        AdvisoryLockQueryBuilder::to_lock_key(b"app:", workspace_directory.as_os_str().as_encoded_bytes())
    }

    pub fn get_session_lock_key(session_id: &str) -> i64 {
        AdvisoryLockQueryBuilder::to_lock_key(b"session:", session_id.as_bytes())
    }

    // A session-level lock: held by the connection that took it until it is unlocked or the connection ends.
    pub fn build_try_lock_query(lock_key: i64) -> SqlQuery {
        SqlQuery {
            statement: "SELECT pg_try_advisory_lock($1) AS locked",
            parameters: vec![SqlParameter::BigInt(lock_key)],
        }
    }

    pub fn build_unlock_query(lock_key: i64) -> SqlQuery {
        SqlQuery {
            statement: "SELECT pg_advisory_unlock($1)",
            parameters: vec![SqlParameter::BigInt(lock_key)],
        }
    }

    // Postgres keys advisory locks by a bigint: the first 8 bytes of the SHA-256 of the lock's kind and name, so an
    // app's and a session's lock never share a key, whatever the names.
    fn to_lock_key(kind: &[u8], name: &[u8]) -> i64 {
        let digest: Vec<u8> = Sha256::new().chain_update(kind).chain_update(name).finalize().to_vec();
        let mut key_bytes: [u8; 8] = [0; 8];
        key_bytes.copy_from_slice(&digest[..8]);
        i64::from_be_bytes(key_bytes)
    }
}
