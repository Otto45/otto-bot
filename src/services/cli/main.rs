use std::error::Error;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::handlers::file_log_handler::FileLogHandler;
use crate::managers::logger::handlers::log_handler_protocol::LogHandlerProtocol;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::logger::panic_exception::PanicException;
use crate::services::cli::cli_service::CliService;
use crate::services::cli::exception_handlers::application_exception_report::ApplicationExceptionReport;
use crate::services::cli::logging_bridges::tracing_logging_bridge::TracingLoggingBridge;
use crate::services::cli::wire_models::cli_arguments::CliArguments;

// Logs a panic nothing caught, on the main thread or another (a Tokio worker's task), as every other line is logged.
struct UncaughtPanicLogger;

impl UncaughtPanicLogger {
    fn log_panic(panic_hook_info: &PanicHookInfo<'_>) {
        let panic_exception: PanicException = PanicException::from_panic_hook_info(panic_hook_info);
        let thread_name: String = std::thread::current().name().unwrap_or_default().to_string();
        if thread_name == "main" {
            ApplicationLogger::error(
                "The CLI stopped on an unexpected error",
                ContextualData::new(),
                Some(&panic_exception),
            );
        } else {
            ApplicationLogger::error(
                "A thread stopped on an unexpected error",
                ContextualData::new().with("thread_name", thread_name),
                Some(&panic_exception),
            );
        }
    }
}

pub fn main() -> ExitCode {
    // The terminal is the CLI's conversation, so the log goes to a file; to the console only if the file can't be
    // opened, and then only what needs looking at.
    let log_path: Option<PathBuf> = configure_logger();
    // What reqwest and the other libraries log goes through the ApplicationLogger too, and so does any panic.
    TracingLoggingBridge::install();
    std::panic::set_hook(Box::new(UncaughtPanicLogger::log_panic));

    // `--help` and arguments that don't parse are answered by clap, which exits.
    let cli_arguments: CliArguments = CliArguments::parse();
    let exit_code: ExitCode = run(&cli_arguments, log_path);
    ApplicationLogger::close();
    exit_code
}

// Where the CLI logs, like where a service listens, is the entry point's to decide: the log is open before anything
// else runs, configuration included. It is otto-bot's home folder's logs/otto.log.
fn configure_logger() -> Option<PathBuf> {
    let home_directory: Option<PathBuf> = std::env::var_os("OTTO_BOT_HOME").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME")
            .map(|user_home_directory: std::ffi::OsString| PathBuf::from(user_home_directory).join(".otto-bot"))
    });
    let Some(home_directory) = home_directory else {
        ApplicationLogger::configure(LogLevel::Warning, None);
        return None;
    };
    let log_path: PathBuf = home_directory.join("logs").join("otto.log");

    match FileLogHandler::open(&log_path) {
        Ok(file_log_handler) => {
            let handlers: Vec<Box<dyn LogHandlerProtocol>> = vec![Box::new(file_log_handler)];
            ApplicationLogger::configure(LogLevel::Info, Some(handlers));
            Some(log_path)
        }
        Err(_) => {
            ApplicationLogger::configure(LogLevel::Warning, None);
            None
        }
    }
}

fn run(cli_arguments: &CliArguments, log_path: Option<PathBuf>) -> ExitCode {
    let runtime: tokio::runtime::Runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            ApplicationLogger::error(
                "The CLI's runtime could not be started",
                ContextualData::new(),
                Some(&error),
            );
            return ExitCode::FAILURE;
        }
    };

    let ran: Result<(), Box<dyn Error + Send + Sync>> =
        runtime.block_on(CliService::run(cli_arguments, log_path.as_deref()));
    // A read of the terminal can't be cancelled: left to end with the runtime, it would keep the CLI from exiting
    // (after Ctrl-C at the prompt, say) until Enter is pressed. Everything else has ended: the facade is closed.
    runtime.shutdown_background();
    match ran {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            match error.downcast_ref::<ApplicationBaseException>() {
                Some(exception) => {
                    let message: String = ApplicationExceptionReport::report(exception);
                    eprintln!("otto: {message}");
                }
                None => ApplicationLogger::error(
                    "The CLI stopped on an unexpected error",
                    ContextualData::new(),
                    Some(error.as_ref()),
                ),
            }
            ExitCode::FAILURE
        }
    }
}
