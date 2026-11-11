use std::fs::TryLockError;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::lock_store_gateway_exception::LockStoreGatewayException;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::runtime::held_lock::HeldLock;

// One lock file per app, named after its folder's path.
const APPS_DIRECTORY_NAME: &str = "apps";
// One lock file per session, named after its id.
const SESSIONS_DIRECTORY_NAME: &str = "sessions";
const LOCK_FILE_EXTENSION: &str = "lock";
// How often a lock held elsewhere is tried again while it is waited for.
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(250);

/// Keeps the locks that keep runs apart across processes (the CLI and the HTTP API, or two of either), as the
/// operating system's file locks on files in `locks/`: one per app (`apps/<hash of its folder's path>.lock`) and one
/// per session (`sessions/<id>.lock`). The operating system releases a process's locks when it exits, however it
/// exits, so none is ever left behind and none needs cleaning up; the files themselves stay, and hold nothing. A lock
/// is held by an open file, so two runs of one process are kept apart by it too.
pub struct LockStoreGateway {
    locks_directory: PathBuf,
}

impl LockStoreGateway {
    pub fn new(locks_directory: PathBuf) -> Self {
        LockStoreGateway { locks_directory }
    }

    /// The app's lock, if no run (in this process or another) holds it: None when one does. `workspace_directory` is
    /// the app's resolved folder, so every name for it is one lock.
    pub async fn try_lock_app(&self, workspace_directory: &Path) -> ApplicationResult<Option<HeldLock>> {
        let digest: Vec<u8> = Sha256::digest(workspace_directory.as_os_str().as_encoded_bytes()).to_vec();
        let file_name: String = digest.iter().map(|byte: &u8| format!("{byte:02x}")).collect();
        let context: ContextualData = ContextualData::new().with("lock_name", file_name.as_str());
        let file: std::fs::File = self.open_lock_file(APPS_DIRECTORY_NAME, &file_name, &context).await?;
        LockStoreGateway::try_lock(file, &context)
    }

    /// The session's lock, once nothing else holds it: a run of the session in another process (still compacting its
    /// conversation, say) is waited for, and so is a cancelled run's task until it has stopped. Waiting can be cancelled
    /// with the run waiting.
    pub async fn lock_session(&self, session_id: &str) -> ApplicationResult<HeldLock> {
        let context: ContextualData = ContextualData::new().with("session_id", session_id);
        // The id names a file, so it may hold nothing that leads elsewhere; ids are lowercase letters and digits.
        let is_file_name: bool = !session_id.is_empty()
            && session_id
                .chars()
                .all(|character: char| character.is_ascii_lowercase() || character.is_ascii_digit());
        if !is_file_name {
            return Err(ApplicationBaseException::new(
                LockStoreGatewayException::InvalidLockName,
                "A session id can't name a lock",
                context,
            ));
        }

        let mut waited: bool = false;
        loop {
            let file: std::fs::File = self
                .open_lock_file(SESSIONS_DIRECTORY_NAME, session_id, &context)
                .await?;
            if let Some(held_lock) = LockStoreGateway::try_lock(file, &context)? {
                if waited {
                    ApplicationLogger::info("The session's lock is free again; its run goes on", context);
                }
                return Ok(held_lock);
            }
            if !waited {
                ApplicationLogger::info(
                    "The session's lock is held elsewhere (its last run may still be compacting); its run waits",
                    context.clone(),
                );
                waited = true;
            }
            tokio::time::sleep(LOCK_RETRY_INTERVAL).await;
        }
    }

    /// Holds nothing: each lock is released by whoever holds it.
    pub fn close(&self) {}

    async fn open_lock_file(
        &self,
        directory_name: &str,
        lock_name: &str,
        context: &ContextualData,
    ) -> ApplicationResult<std::fs::File> {
        let directory: PathBuf = self.locks_directory.join(directory_name);
        tokio::fs::create_dir_all(&directory)
            .await
            .map_err(|exception: std::io::Error| {
                LockStoreGateway::to_io_exception(exception, "creating the locks' folder", context)
            })?;
        // Opened to be locked, never written: an existing file is kept as it is.
        let file: tokio::fs::File = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(directory.join(format!("{lock_name}.{LOCK_FILE_EXTENSION}")))
            .await
            .map_err(|exception: std::io::Error| {
                LockStoreGateway::to_io_exception(exception, "opening a lock file", context)
            })?;
        Ok(file.into_std().await)
    }

    // Locks the file without waiting: a lock held elsewhere is None, and so the file is closed again.
    fn try_lock(file: std::fs::File, context: &ContextualData) -> ApplicationResult<Option<HeldLock>> {
        match file.try_lock() {
            Ok(()) => Ok(Some(HeldLock::new(file))),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(exception)) => {
                Err(LockStoreGateway::to_io_exception(exception, "taking a lock", context))
            }
        }
    }

    fn to_io_exception(
        exception: std::io::Error,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        ApplicationBaseException::new(
            LockStoreGatewayException::LockStoreIo,
            format!("The lock store failed while {operation_description}"),
            context.clone().with("error_type", format!("{:?}", exception.kind())),
        )
        .with_source(exception)
    }
}
