use std::backtrace::Backtrace;
use std::error::Error;
use std::fmt;

use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;

/// What every fallible operation of the domain container returns.
pub type ApplicationResult<T> = Result<T, ApplicationBaseException>;

/// The application's one exception type. Which exception it is lives in its kind (see ExceptionKind); the message,
/// the context and the error it was raised from (`with_source`) are the same for every kind.
///
/// It keeps the stack where it was raised: Rust errors carry none of their own, so one is captured here, and ERROR
/// logs it.
#[derive(Debug)]
pub struct ApplicationBaseException {
    kind: ExceptionKind,
    message: String,
    context: ContextualData,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
    // Boxed, so the Err side of every ApplicationResult stays small.
    backtrace: Box<Backtrace>,
}

impl ApplicationBaseException {
    pub fn new(kind: impl Into<ExceptionKind>, message: impl Into<String>, context: ContextualData) -> Self {
        ApplicationBaseException {
            kind: kind.into(),
            message: message.into(),
            context,
            source: None,
            backtrace: Box::new(Backtrace::force_capture()),
        }
    }

    /// The error this exception was raised from, which is logged with it.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn kind(&self) -> ExceptionKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn context(&self) -> &ContextualData {
        &self.context
    }

    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for ApplicationBaseException {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ApplicationBaseException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source: &(dyn Error + Send + Sync + 'static)| source as &(dyn Error + 'static))
    }
}
