use std::path::PathBuf;

use clap::Parser;

use crate::services::cli::wire_models::cli_command::CliCommand;
use crate::services::cli::wire_models::service_tier_argument::ServiceTierArgument;

/// otto: a coding agent that builds Domain Container apps.
#[derive(Debug, Clone, Parser)]
#[command(name = "otto", version, about, args_conflicts_with_subcommands = true)]
pub struct CliArguments {
    #[command(subcommand)]
    pub command: Option<CliCommand>,

    /// The directory otto works in; it reads and writes nothing outside it [default: the current directory]
    #[arg(long)]
    pub workspace: Option<PathBuf>,

    /// Resumes a session started before (`otto sessions` lists them), in its workspace and at its tier
    #[arg(long, value_name = "ID", conflicts_with_all = ["workspace", "service_tier"])]
    pub session: Option<String>,

    /// The OpenAI processing tier the session runs at [default: OTTO_BOT_SERVICE_TIER, or standard]
    #[arg(long, value_enum, global = true)]
    pub service_tier: Option<ServiceTierArgument>,
}
