use std::error::Error;
use std::path::{Path, PathBuf};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, Stdin, Stdout};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_event_record::SessionEventRecord;
use crate::managers::models::session_event_replay::SessionEventReplay;
use crate::managers::models::session_summary::SessionSummary;
use crate::managers::models::started_project::StartedProject;
use crate::managers::models::user_request::UserRequest;
use crate::otto_bot_facade::OttoBotFacade;
use crate::services::cli::exception_handlers::application_exception_report::ApplicationExceptionReport;
use crate::services::cli::mappers::session_event_mapper::SessionEventMapper;
use crate::services::cli::mappers::session_options_mapper::SessionOptionsMapper;
use crate::services::cli::mappers::session_summary_mapper::SessionSummaryMapper;
use crate::services::cli::mappers::started_project_mapper::StartedProjectMapper;
use crate::services::cli::wire_models::cli_arguments::CliArguments;
use crate::services::cli::wire_models::cli_command::CliCommand;

const GREETING: &str = "otto: what should I build? (Ctrl-C cancels a run; Ctrl-D or /exit quits)\n";
const PROMPT: &str = "> ";
const EXIT_COMMAND: &str = "/exit";

// The interactive CLI: one session, one request per line. A service sits outside the domain container and holds no
// application logic: it maps what is typed to domain models, hands them to the Domain Facade, and prints the
// session's events as they come.
pub struct CliService;

impl CliService {
    /// Creates the Domain Facade, converses until the person quits, then closes it, whether conversing failed or not.
    /// `log_path` is where the CLI logs, to say so; None when it logs to the console.
    pub async fn run(
        cli_arguments: &CliArguments,
        log_path: Option<&Path>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // e.g. invalid configuration, which fails while the facade is created
        let otto_bot_facade: OttoBotFacade = OttoBotFacade::new()?;

        let conversed: Result<(), Box<dyn Error + Send + Sync>> =
            CliService::converse(&otto_bot_facade, cli_arguments, log_path).await;
        otto_bot_facade.close().await;
        conversed
    }

    async fn converse(
        otto_bot_facade: &OttoBotFacade,
        cli_arguments: &CliArguments,
        log_path: Option<&Path>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut stdout: Stdout = tokio::io::stdout();
        let Some((session_id, workspace_directory)) =
            CliService::open_session(otto_bot_facade, cli_arguments, &mut stdout).await?
        else {
            return Ok(());
        };
        // Subscribed before anything is submitted, so no event is missed.
        // The CLI sends its requests itself, so it follows the events from now on.
        let (_, mut events): (Vec<SessionEventRecord>, broadcast::Receiver<SessionEventRecord>) =
            otto_bot_facade.subscribe(&session_id, SessionEventReplay::FromNow)?;
        let mut lines: Lines<BufReader<Stdin>> = BufReader::new(tokio::io::stdin()).lines();

        CliService::write(&mut stdout, GREETING).await?;
        CliService::write(&mut stdout, &format!("workspace: {}\n", workspace_directory.display())).await?;
        CliService::write(&mut stdout, &format!("session: {session_id}\n")).await?;
        if let Some(log_path) = log_path {
            CliService::write(&mut stdout, &format!("log: {}\n", log_path.display())).await?;
        }
        CliService::write(&mut stdout, "\n").await?;
        loop {
            CliService::write(&mut stdout, PROMPT).await?;
            let line: Option<String> = tokio::select! {
                line = lines.next_line() => line?,
                // Ctrl-C at the prompt quits, as end of input does.
                _ = tokio::signal::ctrl_c() => None,
            };

            let Some(text) = line else {
                CliService::write(&mut stdout, "\n").await?;
                return Ok(());
            };
            match text.trim() {
                "" => continue,
                EXIT_COMMAND => return Ok(()),
                _ => {}
            }

            match otto_bot_facade.submit(&session_id, &UserRequest { text }) {
                Ok(()) => CliService::follow_run(otto_bot_facade, &session_id, &mut events, &mut stdout).await?,
                Err(exception) => {
                    let message: String = ApplicationExceptionReport::report(&exception);
                    CliService::write(&mut stdout, &format!("error: {message}\n\n")).await?;
                }
            }
        }
    }

    // The session to converse in, and its workspace: a new app's, a resumed one's, or a new one in the workspace. None
    // when there is nothing to converse about (`otto sessions` has listed them).
    async fn open_session(
        otto_bot_facade: &OttoBotFacade,
        cli_arguments: &CliArguments,
        stdout: &mut Stdout,
    ) -> Result<Option<(String, PathBuf)>, Box<dyn Error + Send + Sync>> {
        let current_directory: PathBuf = std::env::current_dir()?;
        let workspace_directory: PathBuf = match (cli_arguments.command.as_ref(), cli_arguments.session.as_deref()) {
            (Some(CliCommand::Sessions { workspace }), _) => {
                let session_summaries: Vec<SessionSummary> = otto_bot_facade
                    .list_sessions(workspace.as_deref().unwrap_or(&current_directory))
                    .await?;
                CliService::write(stdout, &SessionSummaryMapper::to_list_output(&session_summaries)).await?;
                return Ok(None);
            }
            (
                Some(CliCommand::New {
                    name,
                    stack,
                    parent_directory,
                }),
                _,
            ) => {
                let started_project: StartedProject = otto_bot_facade
                    .start_project(stack, name, parent_directory.as_deref().unwrap_or(&current_directory))
                    .await?;
                CliService::write(stdout, &StartedProjectMapper::to_output(&started_project)).await?;
                started_project.workspace_directory
            }
            (None, Some(session_id)) => {
                let session_summary: SessionSummary = otto_bot_facade.resume_session(session_id).await?;
                CliService::write(stdout, &SessionSummaryMapper::to_resumed_output(&session_summary)).await?;
                return Ok(Some((
                    session_summary.session.id,
                    session_summary.session.workspace_directory,
                )));
            }
            (None, None) => cli_arguments.workspace.clone().unwrap_or(current_directory),
        };
        let session_id: String = otto_bot_facade
            .start_session(&SessionOptionsMapper::to_session_options(
                cli_arguments,
                workspace_directory.clone(),
            ))
            .await?;
        Ok(Some((session_id, workspace_directory)))
    }

    // Prints the run's events until it ends. Ctrl-C cancels the run; its end is still reported as an event.
    async fn follow_run(
        otto_bot_facade: &OttoBotFacade,
        session_id: &str,
        events: &mut broadcast::Receiver<SessionEventRecord>,
        stdout: &mut Stdout,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            tokio::select! {
                received = events.recv() => match received {
                    Ok(session_event_record) => {
                        let session_event: SessionEvent = session_event_record.event;
                        if let Some(output) = SessionEventMapper::to_output(&session_event) {
                            CliService::write(stdout, &output).await?;
                        }
                        if session_event.ends_run() {
                            return Ok(());
                        }
                    }
                    // Too far behind to see every event: the ones still to come are shown.
                    Err(RecvError::Lagged(_)) => {}
                    Err(RecvError::Closed) => return Ok(()),
                },
                _ = tokio::signal::ctrl_c() => {
                    if let Err(exception) = otto_bot_facade.cancel(session_id) {
                        let message: String = ApplicationExceptionReport::report(&exception);
                        CliService::write(stdout, &format!("error: {message}\n\n")).await?;
                    }
                }
            }
        }
    }

    async fn write(stdout: &mut Stdout, text: &str) -> std::io::Result<()> {
        stdout.write_all(text.as_bytes()).await?;
        stdout.flush().await
    }
}
