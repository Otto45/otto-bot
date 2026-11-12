use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;

use crate::managers::configuration_providers::configuration_provider::ConfigurationProvider;
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
use crate::managers::models::run_record::RunRecord;
use crate::managers::models::run_status::RunStatus;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session::Session;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_event_record::SessionEventRecord;
use crate::managers::models::session_event_replay::SessionEventReplay;
use crate::managers::models::session_options::SessionOptions;
use crate::managers::models::session_summary::SessionSummary;
use crate::managers::models::started_project::StartedProject;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::transcript_speaker::TranscriptSpeaker;
use crate::managers::models::user_request::UserRequest;
use crate::managers::processors::session::session_processor::SessionProcessor;
use crate::managers::runtime::held_lock::HeldLock;
use crate::managers::runtime::live_run::LiveRun;
use crate::managers::runtime::live_session::LiveSession;
use crate::managers::runtime::session_event_log::SessionEventLog;
use crate::managers::service_locator::service_locator::ServiceLocator;

// How many events a session's channel holds for its relay to number; the relay keeps up, so it never fills.
const SESSION_EVENT_CAPACITY: usize = 1024;
// How often sessions left idle are looked for.
const IDLE_SWEEP_INTERVAL: Duration = Duration::from_secs(60);
// What a client following a session is told of a run that stopped with its process, when the session is next served.
const INTERRUPTED_RUN_MESSAGE: &str = "The session's last request was stopped before it finished";

type LiveSessions = Arc<Mutex<HashMap<String, LiveSession>>>;

/// Serves sessions: starts them, runs each request in the background, and reports what happens as SessionEvents to
/// every client that subscribes. It owns its live sessions and their runs, and its close() stops them.
pub struct OttoBotManager {
    session_processor: Arc<SessionProcessor>,
    agent_run_engine: Arc<AgentRunEngine>,
    project_bootstrap_engine: Arc<ProjectBootstrapEngine>,
    workspace_gateway: Arc<WorkspaceGateway>,
    // Shared with the Domain Facade, which owns it and closes it.
    data_facade: Arc<DataFacade>,
    live_sessions: LiveSessions,
    // Where apps started by name are kept, each in a folder named after it.
    workspaces_directory: PathBuf,
    // Lets go of sessions idle for the configured time; stopped on close.
    idle_sweeper: JoinHandle<()>,
    // Set when the manager closes: the runs it then stops are left with no end on record, so whichever process next
    // serves their sessions reports them stopped.
    closing: Arc<AtomicBool>,
}

impl OttoBotManager {
    /// Created inside a Tokio runtime: it starts the loop that lets go of idle sessions.
    pub fn new(service_locator: &ServiceLocator, data_facade: Arc<DataFacade>) -> ApplicationResult<Self> {
        let configuration_provider: Arc<ConfigurationProvider> = service_locator.get_configuration_provider()?;
        let idle_session_minutes: u64 =
            u64::try_from(configuration_provider.get_agent_settings().idle_session_minutes).unwrap_or(u64::MAX);
        let live_sessions: LiveSessions = Arc::new(Mutex::new(HashMap::new()));
        Ok(OttoBotManager {
            session_processor: service_locator.get_session_processor()?,
            agent_run_engine: service_locator.get_agent_run_engine()?,
            project_bootstrap_engine: service_locator.get_project_bootstrap_engine()?,
            workspace_gateway: service_locator.get_workspace_gateway()?,
            data_facade,
            idle_sweeper: tokio::spawn(OttoBotManager::sweep_idle_sessions(
                live_sessions.clone(),
                service_locator.get_agent_run_engine()?,
                Duration::from_secs(idle_session_minutes.saturating_mul(60)),
            )),
            live_sessions,
            closing: Arc::new(AtomicBool::new(false)),
            workspaces_directory: configuration_provider
                .get_storage_settings()
                .workspaces_directory
                .clone(),
        })
    }

    /// Starts a new app of the stack, kept with the others started by name (in a folder named after it), from the
    /// stack pack's template.
    pub async fn start_app(&self, stack_name: &str, app_name: &str) -> ApplicationResult<StartedProject> {
        let kebab_name: String = self.project_bootstrap_engine.to_app_name(app_name)?.kebab;
        let app_directory: PathBuf = self.workspaces_directory.join(&kebab_name);
        if self
            .workspace_gateway
            .resolve_workspace_directory(&app_directory)
            .await
            .is_ok()
        {
            return Err(ApplicationBaseException::new(
                OttoBotManagerException::AppAlreadyExists,
                format!("An app named {kebab_name} already exists"),
                ContextualData::new().with("app_name", app_name),
            ));
        }
        self.start_project(stack_name, app_name, &self.workspaces_directory)
            .await
    }

    /// Starts a session in the app (named by its kebab-case name, as `start_app` returned it).
    pub async fn start_app_session(
        &self,
        app_name: &str,
        service_tier: Option<ServiceTier>,
    ) -> ApplicationResult<String> {
        let workspace_directory: PathBuf = self.find_app(app_name).await?;
        self.start_session(&SessionOptions {
            workspace_directory,
            service_tier,
        })
        .await
    }

    /// The app's sessions, most recently active first.
    pub async fn list_app_sessions(&self, app_name: &str) -> ApplicationResult<Vec<SessionSummary>> {
        let workspace_directory: PathBuf = self.find_app(app_name).await?;
        self.list_sessions(&workspace_directory).await
    }

    /// Serves the app's session again, as `resume_session` does. A session of another app, or outside where apps
    /// are kept, is not found.
    pub async fn resume_app_session(&self, app_name: &str, session_id: &str) -> ApplicationResult<SessionSummary> {
        let session: Session = self.get_app_session(app_name, session_id).await?;
        self.serve_session(session).await
    }

    /// What the person and the orchestrator said in the app's session, oldest first.
    pub async fn get_app_session_transcript(
        &self,
        app_name: &str,
        session_id: &str,
    ) -> ApplicationResult<Vec<TranscriptEntry>> {
        let session: Session = self.get_app_session(app_name, session_id).await?;
        self.data_facade.get_transcript(&session.id).await
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
        self.lock_live_sessions()
            .insert(session_id.clone(), OttoBotManager::to_live_session(session));
        Ok(session_id)
    }

    /// Serves a session kept in the session store again, as it was left: its orchestrator's conversation, its project
    /// map and its briefs carry on. A session already being served is left as it is.
    pub async fn resume_session(&self, session_id: &str) -> ApplicationResult<SessionSummary> {
        let session: Session = self
            .data_facade
            .get_session(session_id)
            .await?
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;
        self.serve_session(session).await
    }

    // The kept session served, unless it already is, and summarized.
    async fn serve_session(&self, session: Session) -> ApplicationResult<SessionSummary> {
        // Its workspace must still be there.
        self.workspace_gateway
            .resolve_workspace_directory(&session.workspace_directory)
            .await?;
        let transcript_entries: Vec<TranscriptEntry> = self.data_facade.get_transcript(&session.id).await?;
        // A session not served here yet may have been left mid-run by a process that stopped.
        let is_served: bool = self.lock_live_sessions().contains_key(&session.id);
        let run_was_interrupted: bool = !is_served && self.end_interrupted_run(&session).await?;

        {
            let mut live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
            let live_session: &mut LiveSession = live_sessions
                .entry(session.id.clone())
                .or_insert_with(|| OttoBotManager::to_live_session(session.clone()));
            live_session.last_used = Instant::now();
            // Recorded in the log itself, not sent as a run's event: it is for a client that asks what it missed (it
            // was following the run when the process stopped), not for one following the session's next run.
            if run_was_interrupted {
                live_session
                    .event_log
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .record(SessionEvent::RunFailed {
                        message: INTERRUPTED_RUN_MESSAGE.to_string(),
                    });
            }
        }
        Ok(self.session_processor.summarize_session(session, &transcript_entries))
    }

    // Ends, on record, a run of the session that stopped with its process: its last run has no end, and nothing is
    // running in its app any more (a run under way holds its app's lock, in whichever process). Whether it did. A
    // run of another session of the app under way leaves it for the next time the session is served.
    async fn end_interrupted_run(&self, session: &Session) -> ApplicationResult<bool> {
        let run_records: Vec<RunRecord> = self.data_facade.get_run_records(&session.id).await?;
        if !self.session_processor.has_unended_run(&run_records) {
            return Ok(false);
        }
        let Some(app_lock) = self.data_facade.try_lock_app(&session.workspace_directory).await? else {
            return Ok(false);
        };
        let recorded: ApplicationResult<()> = self
            .data_facade
            .append_run_record(
                &session.id,
                &RunRecord {
                    status: RunStatus::Failed,
                    recorded_on: Utc::now(),
                },
            )
            .await;
        app_lock.release().await;
        recorded?;
        Ok(true)
    }

    // The app's folder, by the app's kebab-case name: a name in any other form, or with no folder, is no app's.
    async fn find_app(&self, app_name: &str) -> ApplicationResult<PathBuf> {
        let to_app_not_found_exception = || -> ApplicationBaseException {
            ApplicationBaseException::new(
                OttoBotManagerException::AppNotFound,
                format!("There is no app named {app_name}"),
                ContextualData::new().with("app_name", app_name),
            )
        };
        let kebab_name: String = self
            .project_bootstrap_engine
            .to_app_name(app_name)
            .map_err(|_: ApplicationBaseException| to_app_not_found_exception())?
            .kebab;
        if kebab_name != app_name {
            return Err(to_app_not_found_exception());
        }
        self.workspace_gateway
            .resolve_workspace_directory(&self.workspaces_directory.join(&kebab_name))
            .await
            .map_err(|_: ApplicationBaseException| to_app_not_found_exception())
    }

    // The app's session, kept in the session store, in the app's folder.
    async fn get_app_session(&self, app_name: &str, session_id: &str) -> ApplicationResult<Session> {
        let workspace_directory: PathBuf = self.find_app(app_name).await?;
        self.data_facade
            .get_session(session_id)
            .await?
            .filter(|session: &Session| session.workspace_directory == workspace_directory)
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))
    }

    /// The sessions kept for the workspace, most recently active first.
    pub async fn list_sessions(&self, workspace_directory: &Path) -> ApplicationResult<Vec<SessionSummary>> {
        let workspace_directory: PathBuf = self
            .workspace_gateway
            .resolve_workspace_directory(workspace_directory)
            .await?;
        let sessions: Vec<Session> = self.data_facade.get_sessions().await?;

        let mut session_summaries: Vec<SessionSummary> = Vec::new();
        for session in sessions {
            if session.workspace_directory != workspace_directory {
                continue;
            }
            let transcript_entries: Vec<TranscriptEntry> = self.data_facade.get_transcript(&session.id).await?;
            session_summaries.push(self.session_processor.summarize_session(session, &transcript_entries));
        }
        self.session_processor.sort_session_summaries(&mut session_summaries);
        Ok(session_summaries)
    }

    /// Starts running the request in the background and returns; what happens is reported on the session's events,
    /// so a client subscribes before it submits.
    pub async fn submit(&self, session_id: &str, user_request: &UserRequest) -> ApplicationResult<()> {
        self.session_processor.check_user_request(user_request)?;

        let workspace_directory: PathBuf = {
            let mut live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
            let live_session: &mut LiveSession = live_sessions
                .get_mut(session_id)
                .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;
            live_session.last_used = Instant::now();
            OttoBotManager::check_not_running(live_session, session_id)?;
            live_session.session.workspace_directory.clone()
        };
        // One run per app, whichever process runs it: the lock is held until the run has ended.
        let Some(app_lock) = self.data_facade.try_lock_app(&workspace_directory).await? else {
            return Err(ApplicationBaseException::new(
                OttoBotManagerException::AppBusy,
                "The app is busy with another request; send this once it is done",
                ContextualData::new().with("session_id", session_id),
            ));
        };

        let mut live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
        let live_session: &mut LiveSession = live_sessions
            .get_mut(session_id)
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;
        // A run of it may have started while the lock was taken (and ended, or it would hold the lock).
        OttoBotManager::check_not_running(live_session, session_id)?;

        // The last run's task may still be compacting: it is kept, to be stopped on close, and the new run waits for it.
        if let Some(last_run) = live_session.run.take()
            && !last_run.is_finished()
        {
            live_session.compacting_run = Some(last_run);
        }

        live_session.events.send(SessionEvent::RunStarted).ok();
        let end_reported: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let run: JoinHandle<()> = tokio::spawn(OttoBotManager::run(
            self.agent_run_engine.clone(),
            self.session_processor.clone(),
            live_session.session.clone(),
            user_request.clone(),
            Utc::now(),
            live_session.events.clone(),
            self.data_facade.clone(),
            live_session.work_lock.clone(),
            app_lock,
            end_reported.clone(),
        ));
        live_session.run = Some(LiveRun {
            abort_handle: run.abort_handle(),
            watcher: tokio::spawn(OttoBotManager::watch(
                run,
                live_session.session.id.clone(),
                live_session.events.clone(),
                self.data_facade.clone(),
                end_reported.clone(),
                self.closing.clone(),
            )),
            end_reported,
        });
        Ok(())
    }

    /// The session's kept events the replay asks for, then every event from now on, each once.
    pub fn subscribe(
        &self,
        session_id: &str,
        session_event_replay: SessionEventReplay,
    ) -> ApplicationResult<(Vec<SessionEventRecord>, broadcast::Receiver<SessionEventRecord>)> {
        let mut live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
        let live_session: &mut LiveSession = live_sessions
            .get_mut(session_id)
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;
        live_session.last_used = Instant::now();
        Ok(live_session
            .event_log
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .subscribe(session_event_replay))
    }

    /// Stops the session's run, if one is under way; its end is reported as `RunCancelled`. A run that has ended is
    /// left to finish compacting.
    pub fn cancel(&self, session_id: &str) -> ApplicationResult<()> {
        let live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> = self.lock_live_sessions();
        let live_session: &LiveSession = live_sessions
            .get(session_id)
            .ok_or_else(|| OttoBotManager::to_session_not_found_exception(session_id))?;

        if let Some(live_run) = live_session.run.as_ref()
            && !live_run.has_ended()
        {
            live_run.abort_handle.abort();
        }
        Ok(())
    }

    /// Stops every run under way, and every compaction, and waits until each one's end is reported.
    pub async fn close(&self) {
        self.closing.store(true, Ordering::SeqCst);
        self.idle_sweeper.abort();
        let watchers: Vec<JoinHandle<()>> = self
            .lock_live_sessions()
            .drain()
            .flat_map(|(_, live_session): (String, LiveSession)| {
                // A relay ends on its own once the session's runs have; stopped here so nothing outlives the facade.
                live_session.relay.abort();
                [live_session.run, live_session.compacting_run]
            })
            .flatten()
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

    // The run's task: waits for the session's last run to finish compacting, runs the request, and reports how the run
    // ended, unless it is aborted or panics, which its watcher reports instead. A run that completed then compacts the
    // orchestrator's conversation if it has grown long: the person reads the reply meanwhile.
    #[allow(clippy::too_many_arguments)]
    async fn run(
        agent_run_engine: Arc<AgentRunEngine>,
        session_processor: Arc<SessionProcessor>,
        session: Session,
        user_request: UserRequest,
        submitted_on: DateTime<Utc>,
        events: broadcast::Sender<SessionEvent>,
        data_facade: Arc<DataFacade>,
        work_lock: Arc<tokio::sync::Mutex<()>>,
        app_lock: HeldLock,
        end_reported: Arc<AtomicBool>,
    ) {
        // The session's last run in this process, then in any other, may still be compacting: the run waits for it.
        let _working: tokio::sync::OwnedMutexGuard<()> = work_lock.lock_owned().await;
        let ran: ApplicationResult<(RunSummary, i64, HeldLock)> = async {
            let session_lock: HeldLock = data_facade.lock_session(&session.id).await?;
            let (run_summary, orchestrator_input_tokens): (RunSummary, i64) = OttoBotManager::run_request(
                &agent_run_engine,
                &session_processor,
                &session,
                &user_request,
                submitted_on,
                &events,
                &data_facade,
            )
            .await?;
            Ok((run_summary, orchestrator_input_tokens, session_lock))
        }
        .await;
        // How the run ended is kept before the app's lock is let go of, so no one finds the app free and the run
        // without an end.
        OttoBotManager::record_run_end(
            &data_facade,
            &session.id,
            if ran.is_ok() {
                RunStatus::Completed
            } else {
                RunStatus::Failed
            },
        )
        .await;
        // The app's files are done with, so another session's request may run while this one compacts; let go before
        // the end is reported, so a request sent as soon as it is seen finds the app free (a lock kept in a database
        // is free only once the database says so, so this waits for it).
        app_lock.release().await;
        // Marked before it is reported, so a request sent as soon as the end is seen finds the run ended.
        end_reported.store(true, Ordering::SeqCst);
        // The session's lock is kept through the compaction.
        let (orchestrator_input_tokens, session_lock): (i64, HeldLock) = match ran {
            Ok((run_summary, orchestrator_input_tokens, session_lock)) => {
                events.send(SessionEvent::RunCompleted { run_summary }).ok();
                (orchestrator_input_tokens, session_lock)
            }
            Err(exception) => {
                // Logged here, where it is caught, so the client is only told what failed.
                ApplicationLogger::log_exception(&exception, LogLevel::Error);
                events
                    .send(SessionEvent::RunFailed {
                        message: exception.message().to_string(),
                    })
                    .ok();
                return;
            }
        };

        let compacted: ApplicationResult<bool> = agent_run_engine
            .compact_orchestrator(&session, orchestrator_input_tokens, &events, &data_facade)
            .await;
        // The run has ended: a compaction that failed is only logged, and tried again after the next run.
        if let Err(exception) = compacted {
            ApplicationLogger::error(
                "The orchestrator's conversation could not be compacted",
                ContextualData::new().with("session_id", session.id.as_str()),
                Some(&exception),
            );
        }
        // Compacted or not, the session is free for its next run, here or in another process.
        session_lock.release().await;
    }

    // The orchestrator runs on the request: it answers the person (a question back, when the request isn't clear yet)
    // or briefs the implementor. An accepted brief is kept and reported, built by the implementor and reviewed, and the
    // orchestrator then tells the person how it went. The run's summary, and how many input tokens the orchestrator's
    // last request had.
    async fn run_request(
        agent_run_engine: &AgentRunEngine,
        session_processor: &SessionProcessor,
        session: &Session,
        user_request: &UserRequest,
        submitted_on: DateTime<Utc>,
        events: &broadcast::Sender<SessionEvent>,
        data_facade: &DataFacade,
    ) -> ApplicationResult<(RunSummary, i64)> {
        data_facade
            .append_transcript_entry(
                &session.id,
                &TranscriptEntry {
                    speaker: TranscriptSpeaker::Person,
                    text: user_request.text.clone(),
                    said_on: submitted_on,
                },
            )
            .await?;
        // A build briefed in an earlier run may have ended without the orchestrator being told: it is told now.
        let run_records: Vec<RunRecord> = data_facade.get_run_records(&session.id).await?;
        let unfinished_build: bool = session_processor.has_unfinished_build(&run_records);
        OttoBotManager::record_run(data_facade, &session.id, RunStatus::Started).await?;
        let agent_run_outcome: AgentRunOutcome = agent_run_engine
            .run(session, user_request, unfinished_build, events, data_facade)
            .await?;
        let Some(LaneEnding::BriefAccepted(implementation_brief)) = agent_run_outcome.lane_ending else {
            return Ok((agent_run_outcome.run_summary, agent_run_outcome.last_input_tokens));
        };
        data_facade
            .append_implementation_brief(&session.id, &implementation_brief)
            .await?;
        // The brief's number names its implementor's and reviewer's lanes, so each brief starts fresh.
        let brief_number: usize = data_facade.count_implementation_briefs(&session.id).await?;
        OttoBotManager::record_run(data_facade, &session.id, RunStatus::Building).await?;
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
        OttoBotManager::record_run(data_facade, &session.id, RunStatus::Built).await?;
        let build_summary: RunSummary = implementation_result.run_summary;
        events
            .send(SessionEvent::BriefImplemented {
                implementation_result: implementation_result.clone(),
            })
            .ok();
        let reply_outcome: AgentRunOutcome = agent_run_engine
            .reply_after_build(session, &implementation_result, events, data_facade)
            .await?;
        Ok((
            agent_run_outcome
                .run_summary
                .plus(build_summary)
                .plus(reply_outcome.run_summary),
            reply_outcome.last_input_tokens,
        ))
    }

    // Keeps a step the session's run got to.
    async fn record_run(data_facade: &DataFacade, session_id: &str, run_status: RunStatus) -> ApplicationResult<()> {
        data_facade
            .append_run_record(
                session_id,
                &RunRecord {
                    status: run_status,
                    recorded_on: Utc::now(),
                },
            )
            .await
    }

    // Keeps how the session's run ended. The run has ended whether or not this is kept, so a failure is only logged:
    // the run is then taken for one stopped with its process when its session is next served.
    async fn record_run_end(data_facade: &DataFacade, session_id: &str, run_status: RunStatus) {
        if let Err(exception) = OttoBotManager::record_run(data_facade, session_id, run_status).await {
            ApplicationLogger::error(
                "How a run ended could not be kept",
                ContextualData::new()
                    .with("session_id", session_id)
                    .with("run_status", run_status.name()),
                Some(&exception),
            );
        }
    }

    // Reports the end of a run that could not report it itself: cancelled, or stopped by a panic (which the panic
    // hook has logged), and keeps it on record. A task stopped after its run reported its end (while compacting) has
    // nothing to report. A run stopped because the manager is closing is left with no end on record: no client is
    // told now, so whichever process next serves its session reports it stopped.
    async fn watch(
        run: JoinHandle<()>,
        session_id: String,
        events: broadcast::Sender<SessionEvent>,
        data_facade: Arc<DataFacade>,
        end_reported: Arc<AtomicBool>,
        closing: Arc<AtomicBool>,
    ) {
        let joined: Result<(), tokio::task::JoinError> = run.await;
        if end_reported.load(Ordering::SeqCst) {
            return;
        }
        match joined {
            Ok(()) => {}
            Err(join_error) if join_error.is_cancelled() => {
                if !closing.load(Ordering::SeqCst) {
                    OttoBotManager::record_run_end(&data_facade, &session_id, RunStatus::Cancelled).await;
                }
                events.send(SessionEvent::RunCancelled).ok();
            }
            Err(_) => {
                OttoBotManager::record_run_end(&data_facade, &session_id, RunStatus::Failed).await;
                events
                    .send(SessionEvent::RunFailed {
                        message: "The run stopped on an unexpected error".to_string(),
                    })
                    .ok();
            }
        }
    }

    // A session served from now on, with a channel for its runs' events, relayed into its event log, and no run yet.
    // Its events are numbered from the time it is served (in thousandths of a millisecond), above any number an
    // earlier process gave them.
    fn to_live_session(session: Session) -> LiveSession {
        let (events, relayed): (broadcast::Sender<SessionEvent>, broadcast::Receiver<SessionEvent>) =
            broadcast::channel(SESSION_EVENT_CAPACITY);
        let first_event_id: u64 = u64::try_from(Utc::now().timestamp_millis())
            .unwrap_or(0)
            .saturating_mul(1000);
        let event_log: Arc<Mutex<SessionEventLog>> = Arc::new(Mutex::new(SessionEventLog::new(first_event_id)));
        LiveSession {
            relay: tokio::spawn(OttoBotManager::relay(relayed, event_log.clone(), session.id.clone())),
            session,
            events,
            event_log,
            run: None,
            compacting_run: None,
            work_lock: Arc::new(tokio::sync::Mutex::new(())),
            last_used: Instant::now(),
        }
    }

    // Records each event the session's runs send in its event log, until the session and its runs are gone.
    async fn relay(
        mut relayed: broadcast::Receiver<SessionEvent>,
        event_log: Arc<Mutex<SessionEventLog>>,
        session_id: String,
    ) {
        loop {
            match relayed.recv().await {
                Ok(session_event) => event_log
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .record(session_event),
                Err(RecvError::Lagged(missed_count)) => ApplicationLogger::warning(
                    "A session's events came faster than they could be recorded; some were lost",
                    ContextualData::new()
                        .with("session_id", session_id.as_str())
                        .with("missed_count", i64::try_from(missed_count).unwrap_or(i64::MAX)),
                    None,
                ),
                Err(RecvError::Closed) => return,
            }
        }
    }

    // Every minute, lets go of the sessions left idle for `idle_session_timeout`: nothing running, no client following.
    // A session let go is resumed from the session store when it is next used.
    async fn sweep_idle_sessions(
        live_sessions: LiveSessions,
        agent_run_engine: Arc<AgentRunEngine>,
        idle_session_timeout: Duration,
    ) {
        let mut interval: tokio::time::Interval = tokio::time::interval(IDLE_SWEEP_INTERVAL);
        loop {
            interval.tick().await;
            let let_go: Vec<LiveSession> = {
                let mut live_sessions: MutexGuard<'_, HashMap<String, LiveSession>> =
                    live_sessions.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                let idle_ids: Vec<String> = live_sessions
                    .iter()
                    .filter(|(_, live_session): &(&String, &LiveSession)| {
                        live_session.is_idle() && live_session.last_used.elapsed() >= idle_session_timeout
                    })
                    .map(|(session_id, _): (&String, &LiveSession)| session_id.clone())
                    .collect();
                idle_ids
                    .iter()
                    .filter_map(|session_id: &String| live_sessions.remove(session_id))
                    .collect()
            };
            for live_session in let_go {
                // What the model's gateway keeps for the session (its lanes' connections) is let go of with it.
                agent_run_engine.release_session(&live_session.session.id).await;
                ApplicationLogger::info(
                    "An idle session was let go",
                    ContextualData::new().with("session_id", live_session.session.id.as_str()),
                );
            }
        }
    }

    // A session runs one request at a time.
    fn check_not_running(live_session: &LiveSession, session_id: &str) -> ApplicationResult<()> {
        if live_session.is_running() {
            return Err(ApplicationBaseException::new(
                OttoBotManagerException::SessionBusy,
                "The session is still working on its last request",
                ContextualData::new().with("session_id", session_id),
            ));
        }
        Ok(())
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
