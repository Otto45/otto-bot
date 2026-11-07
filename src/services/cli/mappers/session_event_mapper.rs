use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_event::SessionEvent;

// What the terminal shows for each event.
pub struct SessionEventMapper;

impl SessionEventMapper {
    /// The text to print for an event, if it shows anything.
    pub fn to_output(session_event: &SessionEvent) -> Option<String> {
        match session_event {
            SessionEvent::RunStarted | SessionEvent::RunCompleted => None,
            SessionEvent::ModelResponded {
                text,
                usage,
                service_tier,
            } => Some(format!(
                "\n{text}\n\n{}\n\n",
                SessionEventMapper::to_usage_line(usage, *service_tier)
            )),
            SessionEvent::RunFailed { message } => Some(format!("\nerror: {message}\n\n")),
            SessionEvent::RunCancelled => Some("\ncancelled\n\n".to_string()),
        }
    }

    fn to_usage_line(usage: &ModelUsage, service_tier: Option<ServiceTier>) -> String {
        format!(
            "[{} in ({} cached) · {} out ({} reasoning) · {} tier]",
            usage.input_tokens,
            usage.cached_input_tokens,
            usage.output_tokens,
            usage.reasoning_tokens,
            service_tier.map_or("unknown", ServiceTier::name)
        )
    }
}
