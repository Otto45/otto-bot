use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::implementation_result::ImplementationResult;
use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::work_plan::WorkPlan;

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
            SessionEvent::PlanAccepted { work_plan } => Some(SessionEventMapper::to_plan_output(work_plan)),
            SessionEvent::PlanImplemented { implementation_result } => {
                Some(SessionEventMapper::to_implementation_output(implementation_result))
            }
            SessionEvent::RunCompleted { run_summary } => {
                Some(format!("\n{}\n\n", SessionEventMapper::to_summary_line(run_summary)))
            }
            SessionEvent::RunFailed { message } => Some(format!("\nerror: {message}\n\n")),
            SessionEvent::RunCancelled => Some("\ncancelled\n\n".to_string()),
        }
    }

    fn to_plan_output(work_plan: &WorkPlan) -> String {
        let mut output: String = format!("\nplan: {}\n", work_plan.summary.trim());
        for assumption in &work_plan.assumptions {
            output.push_str(&format!("  assumed: {assumption}\n"));
        }
        for (index, step) in work_plan.steps.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", index + 1, step.trim()));
        }
        output
    }

    fn to_implementation_output(implementation_result: &ImplementationResult) -> String {
        let checks: &str = if implementation_result.checks_passed {
            "checks pass"
        } else {
            "checks fail"
        };
        let mut output: String = match implementation_result.report.as_ref() {
            Some(agent_report) => format!(
                "\nimplementor: {} ({checks}): {}\n",
                agent_report.status.name(),
                agent_report.summary.trim()
            ),
            None => format!("\nimplementor: ended without a report ({checks})\n"),
        };
        for need in implementation_result
            .report
            .iter()
            .flat_map(|agent_report: &AgentReport| agent_report.needs.iter())
        {
            output.push_str(&format!("  needs: {need}\n"));
        }
        if let Some(review) = implementation_result.review.as_ref() {
            output.push_str(&format!("reviewer: {}\n", review.summary.trim()));
            for finding in &review.findings {
                output.push_str(&format!(
                    "  [{}] {}: {}\n",
                    finding.severity.name(),
                    finding.path,
                    finding.problem.trim()
                ));
            }
            if implementation_result.fixed_after_review {
                output.push_str("  (the implementor was sent the high-severity findings to fix)\n");
            }
        }
        output
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
