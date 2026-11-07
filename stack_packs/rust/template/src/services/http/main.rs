use std::error::Error;
use std::panic::PanicHookInfo;
use std::process::ExitCode;

use tokio::net::TcpListener;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::logger::panic_exception::PanicException;
use crate::services::http::http_service::HttpService;
use crate::services::http::logging_bridges::tracing_logging_bridge::TracingLoggingBridge;
use crate::services::http::middleware::unexpected_error_middleware::UnexpectedErrorMiddleware;

// Where the service listens unless HOST and PORT say otherwise, as a deployment does. Kept here, outside the domain
// container, because the ConfigurationProvider serves the application behind the facade, not the processes that
// host it.
const HOST: &str = "127.0.0.1";
const PORT: u16 = 8000;

// Logs a panic nothing caught, on the main thread or another (a Tokio worker's task), as every other line is logged.
struct UncaughtPanicLogger;

impl UncaughtPanicLogger {
    fn log_panic(panic_hook_info: &PanicHookInfo<'_>) {
        // A panic in a request is the UnexpectedErrorMiddleware's to log, with the request it failed.
        if UnexpectedErrorMiddleware::take_over_panic(panic_hook_info) {
            return;
        }

        let panic_exception: PanicException = PanicException::from_panic_hook_info(panic_hook_info);
        let thread_name: String = std::thread::current().name().unwrap_or_default().to_string();
        if thread_name == "main" {
            ApplicationLogger::error(
                "The service stopped on an unexpected error",
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
    ApplicationLogger::configure(LogLevel::Info, None);
    // What hyper, sqlx and the other libraries log goes through the ApplicationLogger too, and so does any panic.
    TracingLoggingBridge::install();
    std::panic::set_hook(Box::new(UncaughtPanicLogger::log_panic));

    let exit_code: ExitCode = run();
    ApplicationLogger::close();
    exit_code
}

fn run() -> ExitCode {
    let host: String = std::env::var("HOST").unwrap_or_else(|_: std::env::VarError| HOST.to_string());
    let port: u16 = match std::env::var("PORT") {
        Err(_) => PORT,
        Ok(port) => match port.parse::<u16>() {
            Ok(port) => port,
            Err(error) => {
                ApplicationLogger::error(
                    "PORT is not a port number",
                    ContextualData::new().with("port", port),
                    Some(&error),
                );
                return ExitCode::FAILURE;
            }
        },
    };

    let runtime: tokio::runtime::Runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            ApplicationLogger::error(
                "The service's runtime could not be started",
                ContextualData::new(),
                Some(&error),
            );
            return ExitCode::FAILURE;
        }
    };

    let served: Result<(), Box<dyn Error + Send + Sync>> = runtime.block_on(async {
        let listener: TcpListener = TcpListener::bind((host.as_str(), port)).await?;
        ApplicationLogger::info(
            "The service is listening",
            ContextualData::new().with("host", host.as_str()).with("port", port),
        );
        HttpService::serve(listener, ShutdownSignal::received()).await
    });

    match served {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            match error.downcast_ref::<ApplicationBaseException>() {
                Some(exception) => ApplicationLogger::log_exception(exception, LogLevel::Error),
                None => ApplicationLogger::error(
                    "The service stopped on an unexpected error",
                    ContextualData::new(),
                    Some(error.as_ref()),
                ),
            }
            ExitCode::FAILURE
        }
    }
}

// Ctrl-C, or the SIGTERM a deployment stops the service with: the server stops taking requests, finishes the ones
// under way, and the facade is closed.
struct ShutdownSignal;

impl ShutdownSignal {
    async fn received() {
        let ctrl_c = async {
            let _ = tokio::signal::ctrl_c().await;
        };

        #[cfg(unix)]
        let terminate = async {
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(mut signal) => {
                    signal.recv().await;
                }
                Err(_) => std::future::pending::<()>().await,
            }
        };
        #[cfg(not(unix))]
        let terminate: std::future::Pending<()> = std::future::pending::<()>();

        tokio::select! {
            () = ctrl_c => {},
            () = terminate => {},
        }
    }
}
