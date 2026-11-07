/// One worker's share of a work plan: the files it owns (and only it may change), and what it is to build in them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPackage {
    /// snake_case, unique within the plan; the worker's lane is named after it.
    pub name: String,
    pub brief: String,
    pub owned_paths: Vec<String>,
}
