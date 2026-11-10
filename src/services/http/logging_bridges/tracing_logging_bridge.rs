use std::fmt::Debug;

use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::{Context, SubscriberExt};

use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;

// Routes what the libraries the service runs log through `tracing` (hyper, sqlx, reqwest) to the ApplicationLogger,
// so every line the service writes has the one format. Installed as the process's tracing subscriber by main.
pub struct TracingLoggingBridge;

impl TracingLoggingBridge {
    pub fn install() {
        let subscriber = tracing_subscriber::registry().with(TracingLoggingBridge.with_filter(LevelFilter::INFO));
        // Only the first subscriber of a process is installed; main installs this one before anything logs.
        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}

impl<S: Subscriber> Layer<S> for TracingLoggingBridge {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        let mut message_visitor: MessageVisitor = MessageVisitor { message: String::new() };
        event.record(&mut message_visitor);

        let metadata: &tracing::Metadata<'_> = event.metadata();
        let context: ContextualData = ContextualData::new().with("logger", metadata.target()).with(
            "origin",
            format!(
                "{}:{}",
                metadata.file().unwrap_or("unknown"),
                metadata.line().unwrap_or_default()
            ),
        );

        match *metadata.level() {
            Level::ERROR => ApplicationLogger::error(&message_visitor.message, context, None),
            Level::WARN => ApplicationLogger::warning(&message_visitor.message, context, None),
            Level::INFO => ApplicationLogger::info(&message_visitor.message, context),
            _ => ApplicationLogger::debug(&message_visitor.message, context),
        }
    }
}

// Keeps an event's message; its other fields can quote anything (a statement, a URL), so they are not logged.
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.message = format!("{value:?}");
        }
    }
}
