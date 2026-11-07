use crate::managers::models::cache_lifetime::CacheLifetime;

/// One layer of the instructions that open every request, cached on its own. Layers go from what the most requests
/// share (the stack's) to what the fewest do, so a request reuses the longest prefix it shares with any before it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptLayer {
    pub text: String,
    pub cache_lifetime: CacheLifetime,
}
