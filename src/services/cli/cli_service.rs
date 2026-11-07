use std::error::Error;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines, Stdin, Stdout};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::user_request::UserRequest;
use crate::otto_bot_facade::OttoBotFacade;
use crate::services::cli::exception_handlers::application_exception_report::ApplicationExceptionReport;
use crate::services::cli::mappers::session_event_mapper::SessionEventMapper;
use crate::services::cli::mappers::session_options_mapper::SessionOptionsMapper;
use crate::services::cli::wire_models::cli_arguments::CliArguments;

const GREETING: &str = "otto: what should I build? (Ctrl-C cancels a run; Ctrl-D or /exit quits)\n\n";
const PROMPT: &str = "> ";
const EXIT_COMMAND: &str = "/exit";

// The interactive CLI: one session, one request per line. A service sits outside the domain container and holds no
// application logic: it maps what is typed to domain models, hands them to the Domain Facade, and prints the
// session's events as they come.
pub struct CliService;

impl CliService {
    /// Creates the Domain Facade, converses until the person quits, then closes it, whether conversing failed or not.
    pub async fn run(cli_arguments: &CliArguments) -> Result<(), Box<dyn Error + Send + Sync>> {
        // e.g. invalid configuration, which fails while the facade is created
        let otto_bot_facade: OttoBotFacade = OttoBotFacade::new()?;

        let conversed: Result<(), Box<dyn Error + Send + Sync>> =
            CliService::converse(&otto_bot_facade, cli_arguments).await;
        otto_bot_facade.close().await;
        conversed
    }

    async fn converse(
        otto_bot_facade: &OttoBotFacade,
        cli_arguments: &CliArguments,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let session_id: String =
            otto_bot_facade.start_session(&SessionOptionsMapper::to_session_options(cli_arguments));
        // Subscribed before anything is submitted, so no event is missed.
        let mut events: broadcast::Receiver<SessionEvent> = otto_bot_facade.subscribe(&session_id)?;
        let mut lines: Lines<BufReader<Stdin>> = BufReader::new(tokio::io::stdin()).lines();
        let mut stdout: Stdout = tokio::io::stdout();

        CliService::write(&mut stdout, GREETING).await?;
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

    // Prints the run's events until it ends. Ctrl-C cancels the run; its end is still reported as an event.
    async fn follow_run(
        otto_bot_facade: &OttoBotFacade,
        session_id: &str,
        events: &mut broadcast::Receiver<SessionEvent>,
        stdout: &mut Stdout,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            tokio::select! {
                received = events.recv() => match received {
                    Ok(session_event) => {
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
