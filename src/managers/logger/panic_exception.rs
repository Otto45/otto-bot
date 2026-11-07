use std::any::Any;
use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;
use std::panic::PanicHookInfo;

/// A panic, as an error the ApplicationLogger can log: Rust's unexpected exceptions are panics (every error the app
/// expects is an ApplicationBaseException in a Result), and a panic hook is where one is seen with its stack.
#[derive(Debug)]
pub struct PanicException {
    message: String,
    location: String,
    backtrace: Backtrace,
}

impl PanicException {
    /// Called inside a panic hook, so the stack captured is the one that panicked.
    pub fn from_panic_hook_info(panic_hook_info: &PanicHookInfo<'_>) -> Self {
        let message: String = PanicException::to_message(panic_hook_info.payload());
        let location: String = panic_hook_info
            .location()
            .map(|location: &std::panic::Location<'_>| location.to_string())
            .unwrap_or_default();

        PanicException {
            message,
            location,
            backtrace: Backtrace::force_capture(),
        }
    }

    /// A panic caught without its hook having seen it (another hook was installed): its message, with no stack.
    pub fn from_payload(payload: &(dyn Any + Send)) -> Self {
        PanicException {
            message: PanicException::to_message(payload),
            location: String::new(),
            backtrace: Backtrace::disabled(),
        }
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl PanicException {
    // What `panic!` was given: a message, almost always.
    fn to_message(payload: &(dyn Any + Send)) -> String {
        if let Some(message) = payload.downcast_ref::<&str>() {
            return (*message).to_string();
        }
        if let Some(message) = payload.downcast_ref::<String>() {
            return message.clone();
        }
        "The panic carried no message".to_string()
    }
}

impl fmt::Display for PanicException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PanicException {}
