use axum::response::sse::Event;

use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::review::Review;
use crate::managers::models::review_finding::ReviewFinding;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_event::SessionEvent;
use crate::managers::models::session_event_record::SessionEventRecord;
use crate::services::http::wire_models::session_event_response::{ReviewFindingResponse, SessionEventResponse};

// The session's own agent, which the domain leaves unnamed in its events.
const ORCHESTRATOR: &str = "orchestrator";

pub struct SessionEventResponseMapper;

impl SessionEventResponseMapper {
    /// The event as a server-sent event: its number as the id, the wire model as JSON data.
    pub fn to_server_sent_event(session_event_record: SessionEventRecord) -> Event {
        let session_event_response: SessionEventResponse =
            SessionEventResponseMapper::to_session_event_response(session_event_record.event);
        Event::default()
            .id(session_event_record.id.to_string())
            .data(serde_json::to_string(&session_event_response).expect("an event response serializes to JSON"))
    }

    pub fn to_session_event_response(session_event: SessionEvent) -> SessionEventResponse {
        match session_event {
            SessionEvent::RunStarted => SessionEventResponse::RunStarted,
            SessionEvent::AssistantReplied { agent, text } => SessionEventResponse::AssistantReplied {
                to_person: agent.is_none(),
                agent: agent.unwrap_or_else(|| ORCHESTRATOR.to_string()),
                text,
            },
            SessionEvent::ToolUsed {
                agent,
                tool_name,
                target,
                succeeded,
            } => SessionEventResponse::ToolUsed {
                agent: agent.unwrap_or_else(|| ORCHESTRATOR.to_string()),
                tool_name,
                target,
                succeeded,
            },
            SessionEvent::BriefAccepted { implementation_brief } => SessionEventResponse::BriefAccepted {
                task: implementation_brief.task,
                decisions: implementation_brief.decisions,
                done_when: implementation_brief.done_when,
                files: implementation_brief.files,
            },
            SessionEvent::BriefImplemented { implementation_result } => SessionEventResponse::BriefImplemented {
                status: implementation_result
                    .report
                    .as_ref()
                    .map(|agent_report: &AgentReport| agent_report.status.name().to_string()),
                summary: implementation_result
                    .report
                    .as_ref()
                    .map(|agent_report: &AgentReport| agent_report.summary.clone()),
                needs: implementation_result
                    .report
                    .map_or_else(Vec::new, |agent_report: AgentReport| agent_report.needs),
                checks_passed: implementation_result.checks_passed,
                review_findings: implementation_result.review.map_or_else(Vec::new, |review: Review| {
                    review
                        .findings
                        .into_iter()
                        .map(SessionEventResponseMapper::to_review_finding_response)
                        .collect()
                }),
                fixed_after_review: implementation_result.fixed_after_review,
            },
            SessionEvent::LaneCompacted { agent, usage } => SessionEventResponse::LaneCompacted {
                agent: agent.unwrap_or_else(|| ORCHESTRATOR.to_string()),
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
            },
            SessionEvent::RunCompleted { run_summary } => SessionEventResponse::RunCompleted {
                turns: run_summary.turns,
                input_tokens: run_summary.usage.input_tokens,
                cached_input_tokens: run_summary.usage.cached_input_tokens,
                output_tokens: run_summary.usage.output_tokens,
                service_tier: run_summary
                    .service_tier
                    .map(|service_tier: ServiceTier| service_tier.name().to_string()),
            },
            // The run's exception has been logged; its message can describe internals, so it isn't sent.
            SessionEvent::RunFailed { .. } => SessionEventResponse::RunFailed {
                error: "The request could not be completed".to_string(),
            },
            SessionEvent::RunCancelled => SessionEventResponse::RunCancelled,
        }
    }

    fn to_review_finding_response(review_finding: ReviewFinding) -> ReviewFindingResponse {
        ReviewFindingResponse {
            severity: review_finding.severity.name().to_string(),
            path: review_finding.path,
            problem: review_finding.problem,
            fix: review_finding.fix,
        }
    }
}
