/// A file of a stack pack's template and its text, by its path relative to the template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}
