use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::services::cli::mappers::cli_error_mapper::CliErrorMapper;

// The facade's exception as the CLI reports it: logged once, here, where it is caught, and told to the person in
// words they can act on.
pub struct ApplicationExceptionReport;

impl ApplicationExceptionReport {
    /// Logs the exception and returns what to tell the person: a client error's own message; for anything else, only
    /// that it failed, since the details (and the stack) are in the log.
    pub fn report(exception: &ApplicationBaseException) -> String {
        if CliErrorMapper::is_client_error(exception.kind()) {
            ApplicationLogger::log_exception(exception, LogLevel::Warning);
            // What is wrong with the configuration (which setting, which environment variable) is in the exception's
            // context, which never holds a setting's value: the CLI's person is who sets it, so they are told.
            if matches!(exception.kind(), ExceptionKind::ConfigurationProvider(_)) {
                let details: Vec<String> = ["environment_variable_name", "errors"]
                    .into_iter()
                    .filter_map(|key: &str| exception.context().get(key))
                    .map(ToString::to_string)
                    .collect();
                if !details.is_empty() {
                    return format!("{}: {}", exception.message(), details.join(": "));
                }
            }
            return exception.message().to_string();
        }

        ApplicationLogger::log_exception(exception, LogLevel::Error);
        format!("{}; the details are in the log", exception.message())
    }
}
