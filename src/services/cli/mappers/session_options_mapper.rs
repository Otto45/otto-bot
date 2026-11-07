use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_options::SessionOptions;
use crate::services::cli::wire_models::cli_arguments::CliArguments;
use crate::services::cli::wire_models::service_tier_argument::ServiceTierArgument;

pub struct SessionOptionsMapper;

impl SessionOptionsMapper {
    pub fn to_session_options(cli_arguments: &CliArguments) -> SessionOptions {
        SessionOptions {
            service_tier: cli_arguments.service_tier.map(SessionOptionsMapper::to_service_tier),
        }
    }

    fn to_service_tier(service_tier_argument: ServiceTierArgument) -> ServiceTier {
        match service_tier_argument {
            ServiceTierArgument::Standard => ServiceTier::Standard,
            ServiceTierArgument::Fast => ServiceTier::Fast,
            ServiceTierArgument::Ultrafast => ServiceTier::Ultrafast,
        }
    }
}
