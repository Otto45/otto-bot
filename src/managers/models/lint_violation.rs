/// Where a file breaks one of the architecture's rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintViolation {
    pub rule_name: String,
    pub message: String,
    pub path: String,
    pub line_number: i64,
    /// The start of what breaks it, on one line.
    pub excerpt: String,
}
