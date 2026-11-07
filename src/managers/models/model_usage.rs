/// The tokens model responses used. Cached input tokens (read from the cache) and cache write tokens (written to it,
/// billed at a premium) are both part of the input tokens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelUsage {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub cache_write_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
}

impl ModelUsage {
    /// This usage and another, together.
    pub fn plus(self, other: ModelUsage) -> ModelUsage {
        ModelUsage {
            input_tokens: self.input_tokens + other.input_tokens,
            cached_input_tokens: self.cached_input_tokens + other.cached_input_tokens,
            cache_write_tokens: self.cache_write_tokens + other.cache_write_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            reasoning_tokens: self.reasoning_tokens + other.reasoning_tokens,
        }
    }
}
