/// How fast, and at what price, the model serves a request: OpenAI's processing tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceTier {
    Standard,
    Fast,
    Ultrafast,
}

impl ServiceTier {
    /// The tier a name stands for (`standard`, `fast`, `ultrafast`), as configuration and clients name it.
    pub fn from_name(name: &str) -> Option<ServiceTier> {
        match name {
            "standard" => Some(ServiceTier::Standard),
            "fast" => Some(ServiceTier::Fast),
            "ultrafast" => Some(ServiceTier::Ultrafast),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ServiceTier::Standard => "standard",
            ServiceTier::Fast => "fast",
            ServiceTier::Ultrafast => "ultrafast",
        }
    }
}
