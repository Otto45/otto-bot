/// How a stack's apps divide among agents: which files only the orchestrator writes, and which a worker may own. A work
/// plan is checked against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackLayout {
    /// The wiring and the contracts (module files, the service locator, the facade, the managers, the models,
    /// exceptions and settings): only the orchestrator writes them, so no two workers need the same file.
    pub orchestrator_globs: Vec<String>,
    /// Where a worker's files may be (processors, engines, gateways, data gateways, services), unless `orchestrator_globs`
    /// claims them.
    pub worker_globs: Vec<String>,
}
