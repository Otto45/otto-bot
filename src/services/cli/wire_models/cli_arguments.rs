use clap::Parser;

use crate::services::cli::wire_models::service_tier_argument::ServiceTierArgument;

/// otto: a coding agent that builds Domain Container apps.
#[derive(Debug, Clone, Parser)]
#[command(name = "otto", version, about)]
pub struct CliArguments {
    /// The OpenAI processing tier the session runs at [default: OTTO_BOT_SERVICE_TIER, or standard]
    #[arg(long, value_enum)]
    pub service_tier: Option<ServiceTierArgument>,
}
