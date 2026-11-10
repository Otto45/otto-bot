use chrono::{DateTime, Utc};

use crate::managers::models::session_summary::SessionSummary;

// How much of a session's first request a list shows, in characters.
const FIRST_REQUEST_LENGTH: usize = 72;

// What the terminal shows for sessions: the one resumed, and a workspace's list.
pub struct SessionSummaryMapper;

impl SessionSummaryMapper {
    pub fn to_resumed_output(session_summary: &SessionSummary) -> String {
        format!(
            "resumed session {} (started {}, {}, last active {})\n",
            session_summary.session.id,
            SessionSummaryMapper::to_time(session_summary.session.started_on),
            SessionSummaryMapper::to_request_count(session_summary.request_count),
            SessionSummaryMapper::to_time(session_summary.last_active_on),
        )
    }

    pub fn to_list_output(session_summaries: &[SessionSummary]) -> String {
        if session_summaries.is_empty() {
            return "no sessions in this workspace yet\n".to_string();
        }
        let mut output: String = String::new();
        for session_summary in session_summaries {
            output.push_str(&format!(
                "{}  last active {}  {}\n    {}\n",
                session_summary.session.id,
                SessionSummaryMapper::to_time(session_summary.last_active_on),
                SessionSummaryMapper::to_request_count(session_summary.request_count),
                session_summary
                    .first_request
                    .as_deref()
                    .map_or("(no request yet)".to_string(), SessionSummaryMapper::to_excerpt),
            ));
        }
        output.push_str("\nresume one with: otto --session <id>\n");
        output
    }

    fn to_time(time: DateTime<Utc>) -> String {
        time.format("%Y-%m-%d %H:%M UTC").to_string()
    }

    fn to_request_count(request_count: usize) -> String {
        match request_count {
            1 => "1 request".to_string(),
            _ => format!("{request_count} requests"),
        }
    }

    // The request's first line, cut short.
    fn to_excerpt(text: &str) -> String {
        let first_line: &str = text.trim().lines().next().unwrap_or_default();
        if first_line.chars().count() <= FIRST_REQUEST_LENGTH && !text.trim().contains('\n') {
            return first_line.to_string();
        }
        let excerpt: String = first_line.chars().take(FIRST_REQUEST_LENGTH).collect();
        format!("{}…", excerpt.trim_end())
    }
}
