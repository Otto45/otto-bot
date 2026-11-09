use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Utc;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::agent_run::agent_run_engine::AgentRunEngine;
use crate::managers::engines::project_bootstrap::project_bootstrap_engine::ProjectBootstrapEngine;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::otto_bot_manager_exception::OttoBotManagerException;
use crate::managers::gateways::workspace::workspace_gateway::WorkspaceGateway;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::agent_run_outcome::AgentRunOutcome;
use crate::managers::models::implementation_result::ImplementationResult;
use crate::managers::models::lane_ending::LaneEnding;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_options::SessionOptions;
use crate::managers::models::started_project::StartedProject;
use crate::managers::models::user_request::UserRequest;
use crate::managers::processors::session::session_processor::SessionProcessor;
use crate::managers::runtime::live_run::LiveRun;
use crate::managers::runtime::live_session::LiveSession;
use crate::managers::service_locator::service_locator::ServiceLocator;

// How many events a session's channel holds for a client that is behind; one further behind misses the oldest.
const SESSION_EVENT_CAPACITY: usize = 1024;

/// Serves sessions: starts them, runs each request in the background, and reports what happens as SessionEvents to
/// every client that subscribes. It owns its live sessions and their runs, and its close() stops them.
pub struct OttoBotManager {
    session_processor: Arc<SessionProcessor>,
    agent_run_engine: Arc<AgentRunEngine>,
    project_bootstrap_engine: Arc<ProjectBootstrapEngine>,
    workspace_gateway: Arc<WorkspaceGateway>,
    // Shared with the Domain Facade, which owns it and closes it.
    data_facade: Arc<DataFacade>,
    live_sessions: Mutex<HashMap<String, LiveSession>>,
}

impl OttoBotManager {
    pub fn new(service_locator: &ServiceLocator, data_facade: Arc<DataFacade>) -> ApplicationResult<Self> {
        Ok(OttoBotManager {
            session_processor: service_locator.get_session_processor()?,
            agent_run_engine: service_locator.get_agent_run_engine()?,
            project_bootstrap_engine: service_locator.get_project_bootstrap_engine()?,
            workspace_gateway: service_locator.get_workspace_gateway()?,
            data_facade,
            live_sessions: Mutex::new(HashMap::new()),
        })
    }

    /// Starts a new app of the stack in `<parent_directory>/<its name>`, from the stack pack's template. A session is
    /// then started in it as in any workspace.
    pub async fn start_project(
        &self,
        stack_name: &str,
        app_name: &str,
        parent_directory: &Path,
    ) -> ApplicationResult<StartedProject> {
        self.project_bootstrap_engine
            .start_project(stack_name, app_name, parent_directory, &self.data_facade)
            .await
    }

    /// Starts a session in the client's workspace, which must be a directory, and keeps it in the session store.
    pub async fn start_session(&self, session_options: &SessionOptions) -> ApplicationResult<String> {
        let workspace_directory: PathBuf = self
            .workspace_gateway
            .resolve_workspace_directory(&session_options.workspace_directory)
            .await?;
        let session: Session = self
            .session_processor
            .create_session(session_options, workspace_directory, Utc::now());
        self.data_facade.save_session(&session).await?;

        let session_id: String = session.id.clone();
        let (events, _): (broadcast::Sender<SessionEvent>, broadcast::Receiver<SessionEvent>) =
            broadcast::channel(SESSION_EVENT_CAPACITY);

        self.lock_live_sessions().insert(
            session_id.clone(),
            LiveSession {
                session,
                events,
                run: None,
            },
        );
        Ok(session_id)
    }

    /// Starts running the request in the background and returns; what happens is reported on the session's events,
    /// so a client subscribes before it submits.
    pub fn submit(&self, session_id: &str, user_request: &UserRequest) -> ApplicationResult<()> {
        self.session_processor.check_user_request(user_request)?;

        let mut live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
        let live_session: &mut LiveSession = live_sessions
            .get_mut(session_id)
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;

        if live_session
            .run
            .as_ref()
            .is_some_and(|live_run: &LiveRun| !live_run.is_finished())
        {
            return Err(ApplicationBaseException::new(
                OttoBotManagerException::SessionBusy,
                "The session is still working on its last request",
                ContextualData::new().with("session_id", session_id),
            ));
        }

        live_session.events.send(SessionEvent::RunStarted).ok();
        let run: JoinHandle<()> = tokio::spawn(OttoBotManager::run(
            self.agent_run_engine.clone(),
            live_session.session.clone(),
            user_request.clone(),
            live_session.events.clone(),
            self.data_facade.clone(),
        ));
        live_session.run = Some(LiveRun {
            abort_handle: run.abort_handle(),
            watcher: tokio::spawn(OttoBotManager::watch(run, live_session.events.clone())),
        });
        Ok(())
    }

    pub fn subscribe(&self, session_id: &str) -> ApplicationResult<broadcast::Receiver<SessionEvent>> {
        self.lock_live_sessions()
            .get(session_id)
            .map(|live_session: &LiveSession| live_session.events.subscribe())
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))
    }

    /// Stops the session's run, if one is under way; its end is reported as `RunCancelled`.
    pub fn cancel(&self, session_id: &str) -> ApplicationResult<()> {
        let live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
        let live_session: &LiveSession = live_sessions
            .get(session_id)
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;

        if let Some(live_run) = live_session.run.as_ref() {
            live_run.abort_handle.abort();
        }
        Ok(())
    }

    /// Stops every run under way and waits until each one's end is reported.
    pub async fn close(&self) {
        let watchers: Vec<JoinHandle<()>> = self
            .lock_live_sessions()
            .drain()
            .filter_map(|(_, live_session): (String, LiveSession)| live_session.run)
            .map(|live_run: LiveRun| {
                live_run.abort_handle.abort();
                live_run.watcher
            })
            .collect();

        for watcher in watchers {
            // A watcher only reports; there is nothing left to do if it was stopped.
            watcher.await.ok();
        }
    }

    // The run's task: reports how the run ended unless it is aborted or panics, which its watcher reports instead.
    async fn run(
        agent_run_engine: Arc<AgentRunEngine>,
        session: Session,
        user_request: UserRequest,
        events: broadcast::Sender<SessionEvent>,
        data_facade: Arc<DataFacade>,
    ) {
        let ran: ApplicationResult<RunSummary> =
            OttoBotManager::run_request(&agent_run_engine, &session, &user_request, &events, &data_facade).await;
        match ran {
            Ok(run_summary) => {
                events.send(SessionEvent::RunCompleted { run_summary }).ok();
            }
            Err(exception) => {
                // Logged here, where it is caught, so the client is only told what failed.
                ApplicationLogger::log_exception(&exception, LogLevel::Error);
                events
                    .send(SessionEvent::RunFailed {
                        message: exception.message().to_string(),
                    })
                    .ok();
            }
        }
    }

    // The orchestrator runs on the request: it answers the person (a question back, when the request isn't clear yet)
    // or briefs the implementor. An accepted brief is kept and reported, built by the implementor and reviewed, and the
    // orchestrator then tells the person how it went.
    async fn run_request(
        agent_run_engine: &AgentRunEngine,
        session: &Session,
        user_request: &UserRequest,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<RunSummary> {
        let agent_run_outcome: AgentRunOutcome =
            agent_run_engine.run(session, user_request, events, data_facade).await?;
        let Some(LaneEnding::BriefAccepted(implementation_brief)) = agent_run_outcome.lane_ending else {
            return Ok(agent_run_outcome.run_summary);
        };
        data_facade
            .append_implementation_brief(&session.id, &implementation_brief)
            .await?;
        // The brief's number names its implementor's and reviewer's lanes, so each brief starts fresh.
        let brief_number: usize = data_facade.count_implementation_briefs(&session.id).await?;
        events
            .send(SessionEvent::BriefAccepted {
                implementation_brief: implementation_brief.clone(),
            })
            .ok();

        let implementation_result: ImplementationResult = agent_run_engine
            .implement_brief(
                session,
                user_request,
                &implementation_brief,
                brief_number,
                events,
                data_facade,
            )
            .await?;
        let build_summary: RunSummary = implementation_result.run_summary;
        events
            .send(SessionEvent::BriefImplemented {
                implementation_result: implementation_result.clone(),
            })
            .ok();
        let reply_summary: RunSummary = agent_run_engine
            .reply_after_build(session, &implementation_result, events, data_facade)
            .await?;
        Ok(agent_run_outcome.run_summary.plus(build_summary).plus(reply_summary))
    }

    // Reports the end of a run that could not report it itself: cancelled, or stopped by a panic (which the panic
    // hook has logged).
    async fn watch(run: JoinHandle<()>, events: broadcast::Sender<SessionEvent>) {
        match run.await {
            Ok(()) => {}
            Err(join_error) if join_error.is_cancelled() => {
                events.send(SessionEvent::RunCancelled).ok();
            }
            Err(_) => {
                events
                    .send(SessionEvent::RunFailed {
                        message: "The run stopped on an unexpected error".to_string(),
                    })
                    .ok();
            }
        }
    }

    fn to_session_not_found_exception(session_id: &str) -> ApplicationBaseException {
        ApplicationBaseException::new(
            OttoBotManagerException::SessionNotFound,
            "The session was not found",
            ContextualData::new().with("session_id", session_id),
        )
    }

    fn lock_live_sessions(&self) -> MutexGuard<'_, HashMap<String, LiveSession>> {
        self.live_sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
