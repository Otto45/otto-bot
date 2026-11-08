use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::work_plan::WorkPlan;

/// A function tool call, read: what the model asked the harness to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolRequest {
    ReadFiles {
        paths: Vec<String>,
    },
    ListFiles {
        directory: String,
    },
    Search {
        pattern: String,
        path_glob: Option<String>,
    },
    RunChecks,
    Scaffold {
        kind: String,
        name: String,
        within: Option<String>,
    },
    SubmitPlan {
        work_plan: WorkPlan,
    },
    Report {
        agent_report: AgentReport,
    },
}
