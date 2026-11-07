use std::error::Error;
use std::panic::PanicHookInfo;
use std::process::ExitCode;

use clap::Parser;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
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
    // The terminal is the CLI's conversation, so only what needs looking at is logged to it (on stderr).
    ApplicationLogger::configure(LogLevel::Warning, None);
    // What reqwest and the other libraries log goes through the ApplicationLogger too, and so does any panic.
    TracingLoggingBridge::install();
    std::panic::set_hook(Box::new(UncaughtPanicLogger::log_panic));

    // `--help` and arguments that don't parse are answered by clap, which exits.
    let cli_arguments: CliArguments = CliArguments::parse();
    let exit_code: ExitCode = run(&cli_arguments);
    ApplicationLogger::close();
    exit_code
}

fn run(cli_arguments: &CliArguments) -> ExitCode {
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

    let ran: Result<(), Box<dyn Error + Send + Sync>> = runtime.block_on(CliService::run(cli_arguments));
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
