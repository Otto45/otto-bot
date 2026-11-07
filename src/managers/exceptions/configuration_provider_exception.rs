/// The ConfigurationProvider's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationProviderException {
    InvalidConfiguration,
    MissingConfiguration,
}

impl ConfigurationProviderException {
    pub fn name(self) -> &'static str {
        match self {
            ConfigurationProviderException::InvalidConfiguration => "InvalidConfigurationException",
            ConfigurationProviderException::MissingConfiguration => "MissingConfigurationException",
        }
    }
}
