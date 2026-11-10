use serde::Serialize;
use utoipa::ToSchema;

/// One of a session's events, as the `data` of a server-sent event whose `id` is its number. A run starts with
/// `run_started` and ends with exactly one of `run_completed`, `run_failed` or `run_cancelled`; the replies meant
/// for the person are the `assistant_replied` events with `to_person` set.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionEventResponse {
    RunStarted,
    AssistantReplied {
        /// orchestrator, or the lane of the agent (e.g. implementor_1).
        agent: String,
        /// Whether it is said to the person (the orchestrator's): in plain words, for a person who isn't technical.
        to_person: bool,
        text: String,
    },
    ToolUsed {
        agent: String,
        tool_name: String,
        target: String,
        succeeded: bool,
    },
    BriefAccepted {
        task: String,
        decisions: Vec<String>,
        done_when: String,
        files: Vec<String>,
    },
    BriefImplemented {
        /// done or blocked; none when the implementor finished without reporting.
        status: Option<String>,
        summary: Option<String>,
        needs: Vec<String>,
        checks_passed: bool,
        review_findings: Vec<ReviewFindingResponse>,
        fixed_after_review: bool,
    },
    LaneCompacted {
        agent: String,
        input_tokens: i64,
        output_tokens: i64,
    },
    RunCompleted {
        turns: i64,
        input_tokens: i64,
        cached_input_tokens: i64,
        output_tokens: i64,
        /// The tier the last response ran at, if the API said.
        service_tier: Option<String>,
    },
    RunFailed {
        error: String,
    },
    RunCancelled,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReviewFindingResponse {
    /// high, medium or low.
    pub severity: String,
    pub path: String,
    pub problem: String,
    pub fix: String,
}
