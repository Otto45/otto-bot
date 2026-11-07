/// The tokens one model response used. Cached input tokens are part of the input tokens, billed at the cached rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelUsage {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
}
