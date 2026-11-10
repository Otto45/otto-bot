use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_event_record::SessionEventRecord;
use crate::managers::models::session_event_replay::SessionEventReplay;
use crate::managers::models::session_options::SessionOptions;
use crate::managers::models::session_summary::SessionSummary;
use crate::managers::models::started_project::StartedProject;
use crate::managers::models::transcript_entry::TranscriptEntry;
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

    /// Starts a new app of the stack (e.g. "python") in `<parent_directory>/<the app's name>`, copied from the stack
    /// pack's template and renamed, without a model. A session is then started in it as in any workspace.
    pub async fn start_project(
        &self,
        stack_name: &str,
        app_name: &str,
        parent_directory: &Path,
    ) -> ApplicationResult<StartedProject> {
        self.otto_bot_manager
            .start_project(stack_name, app_name, parent_directory)
            .await
    }

    /// Starts a session in the client's workspace and returns its id.
    pub async fn start_session(&self, session_options: &SessionOptions) -> ApplicationResult<String> {
        self.otto_bot_manager.start_session(session_options).await
    }

    /// Serves a session started before (by this client or another, in this process or an earlier one) again, as it
    /// was left, and returns it as a client lists it. A session already being served is left as it is.
    pub async fn resume_session(&self, session_id: &str) -> ApplicationResult<SessionSummary> {
        self.otto_bot_manager.resume_session(session_id).await
    }

    /// The sessions kept for the workspace, most recently active first, to pick one to resume.
    pub async fn list_sessions(&self, workspace_directory: &Path) -> ApplicationResult<Vec<SessionSummary>> {
        self.otto_bot_manager.list_sessions(workspace_directory).await
    }

    /// The session's kept events the replay asks for (none, the current run's from its start, or those after the one
    /// a client saw last), then every event from now on, each once and numbered.
    pub fn subscribe(
        &self,
        session_id: &str,
        session_event_replay: SessionEventReplay,
    ) -> ApplicationResult<(Vec<SessionEventRecord>, broadcast::Receiver<SessionEventRecord>)> {
        self.otto_bot_manager.subscribe(session_id, session_event_replay)
    }

    /// Starts a new app of the stack (e.g. "python"), kept with the other apps started by name
    /// (`OTTO_BOT_WORKSPACES_DIRECTORY`), from the stack pack's template. Clients that address apps by name (the HTTP
    /// API) start them here, and never name a path.
    pub async fn start_app(&self, stack_name: &str, app_name: &str) -> ApplicationResult<StartedProject> {
        self.otto_bot_manager.start_app(stack_name, app_name).await
    }

    /// Starts a session in the app, named by its kebab-case name (as `start_app` returned it), and returns its id.
    pub async fn start_app_session(
        &self,
        app_name: &str,
        service_tier: Option<ServiceTier>,
    ) -> ApplicationResult<String> {
        self.otto_bot_manager.start_app_session(app_name, service_tier).await
    }

    /// The app's sessions, most recently active first.
    pub async fn list_app_sessions(&self, app_name: &str) -> ApplicationResult<Vec<SessionSummary>> {
        self.otto_bot_manager.list_app_sessions(app_name).await
    }

    /// Serves the app's session (resuming it if it isn't served yet) and returns it as a client lists it. A session
    /// of another app, or of a workspace that isn't an app's, is not found.
    pub async fn resume_app_session(&self, app_name: &str, session_id: &str) -> ApplicationResult<SessionSummary> {
        self.otto_bot_manager.resume_app_session(app_name, session_id).await
    }

    /// What the person and the orchestrator said in the app's session, oldest first, in plain words.
    pub async fn get_app_session_transcript(
        &self,
        app_name: &str,
        session_id: &str,
    ) -> ApplicationResult<Vec<TranscriptEntry>> {
        self.otto_bot_manager
            .get_app_session_transcript(app_name, session_id)
            .await
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
