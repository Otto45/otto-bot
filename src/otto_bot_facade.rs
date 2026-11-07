use std::sync::Arc;

use tokio::sync::broadcast;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_options::SessionOptions;
use crate::managers::models::user_request::UserRequest;
use crate::managers::otto_bot_manager::OttoBotManager;
use crate::managers::service_locator::service_locator::ServiceLocator;

/// The Domain Facade: the domain container's public entry point, the same for every client.
///
/// A client starts a session, subscribes to its events, then submits requests: each runs in the background, on its
/// own, and what happens is reported as SessionEvents to every subscriber. Rust has no `async with` (a value can't
/// await anything while it is dropped), so whoever creates a facade calls `close().await` when done with it, on every
/// path, the way the CLI does.
pub struct OttoBotFacade {
    service_locator: Arc<ServiceLocator>,
    data_facade: Arc<DataFacade>,
    otto_bot_manager: OttoBotManager,
}

impl OttoBotFacade {
    /// Creates every dependency, so invalid configuration fails here. Runs are Tokio tasks, so a facade is created
    /// and used inside a Tokio runtime.
    pub fn new() -> ApplicationResult<Self> {
        OttoBotFacade::with_service_locator(Arc::new(ServiceLocator::new()))
    }

    pub fn with_service_locator(service_locator: Arc<ServiceLocator>) -> ApplicationResult<Self> {
        let data_facade: Arc<DataFacade> = Arc::new(DataFacade::new(&service_locator)?);
        let otto_bot_manager: OttoBotManager = OttoBotManager::new(&service_locator, data_facade.clone())?;

        Ok(OttoBotFacade {
            service_locator,
            data_facade,
            otto_bot_manager,
        })
    }

    /// Starts a session in the client's workspace and returns its id.
    pub async fn start_session(&self, session_options: &SessionOptions) -> ApplicationResult<String> {
        self.otto_bot_manager.start_session(session_options).await
    }

    /// The session's events from now on. Subscribe before submitting, so no event of the run is missed.
    pub fn subscribe(&self, session_id: &str) -> ApplicationResult<broadcast::Receiver<SessionEvent>> {
        self.otto_bot_manager.subscribe(session_id)
    }

    /// Starts running the request in the background. Fails if the session is still running its last one.
    pub fn submit(&self, session_id: &str, user_request: &UserRequest) -> ApplicationResult<()> {
        self.otto_bot_manager.submit(session_id, user_request)
    }

    /// Stops the session's run, if one is under way.
    pub fn cancel(&self, session_id: &str) -> ApplicationResult<()> {
        self.otto_bot_manager.cancel(session_id)
    }

    pub async fn close(&self) {
        self.otto_bot_manager.close().await;
        self.data_facade.close();
        // Last: the locator closes the singletons (gateways) the manager and the DataFacade were using.
        self.service_locator.close().await;
    }
}
