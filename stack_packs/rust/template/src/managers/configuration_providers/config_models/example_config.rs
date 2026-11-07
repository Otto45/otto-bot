// A config model: what a configuration source read, before it is validated. Mutable, and every field can be missing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExampleConfig {
    pub greeting_prefix: Option<String>,
}
