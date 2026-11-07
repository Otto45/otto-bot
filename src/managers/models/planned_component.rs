/// A component a work plan has the harness scaffold before the workers start: one of the stack's scaffold kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedComponent {
    pub kind: String,
    pub name: String,
    /// The component it lives within, for a kind that lives within one.
    pub within: Option<String>,
}
