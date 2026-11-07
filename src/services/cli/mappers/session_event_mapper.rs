use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_event::SessionEvent;

// What the terminal shows for each event.
pub struct SessionEventMapper;

impl SessionEventMapper {
    /// The text to print for an event, if it shows anything.
    pub fn to_output(session_event: &SessionEvent) -> Option<String> {
        match session_event {
            SessionEvent::RunStarted => None,
            SessionEvent::AssistantReplied { agent, text } => Some(format!(
                "\n{}{}\n",
                SessionEventMapper::to_agent_prefix(agent.as_deref()),
                text.trim_end()
            )),
            SessionEvent::ToolUsed {
                agent,
                tool_name,
                target,
                succeeded,
            } => {
                let mark: &str = if *succeeded { "·" } else { "✗" };
                Some(format!(
                    "  {mark} {}{tool_name} {target}\n",
                    SessionEventMapper::to_agent_prefix(agent.as_deref())
                ))
            }
            SessionEvent::RunCompleted { run_summary } => {
                Some(format!("\n{}\n\n", SessionEventMapper::to_summary_line(run_summary)))
            }
            SessionEvent::RunFailed { message } => Some(format!("\nerror: {message}\n\n")),
            SessionEvent::RunCancelled => Some("\ncancelled\n\n".to_string()),
        }
    }

    // The session's own agent is unnamed; any other is named before what it did.
    fn to_agent_prefix(agent: Option<&str>) -> String {
        agent.map_or_else(String::new, |agent: &str| format!("[{agent}] "))
    }

    fn to_summary_line(run_summary: &RunSummary) -> String {
        format!(
            "[{} {} · {} in ({} cached, {} written to cache) · {} out ({} reasoning) · {} tier]",
            run_summary.turns,
            if run_summary.turns == 1 { "turn" } else { "turns" },
            run_summary.usage.input_tokens,
            run_summary.usage.cached_input_tokens,
            run_summary.usage.cache_write_tokens,
            run_summary.usage.output_tokens,
            run_summary.usage.reasoning_tokens,
            run_summary.service_tier.map_or("unknown", ServiceTier::name)
        )
    }
}
