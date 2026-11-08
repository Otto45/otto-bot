use std::path::PathBuf;

use clap::Subcommand;

/// What `otto` is asked to do before it converses; without one, it converses in the workspace.
#[derive(Debug, Clone, Subcommand)]
pub enum CliCommand {
    /// Starts a new app from a stack's template (copied and renamed, its examples kept), then converses in it
    New {
        /// The app's name (e.g. orders-app); its folder is named after it
        name: String,

        /// The stack the app is built in (e.g. rust, python)
        #[arg(long)]
        stack: String,

        /// The folder the app's folder is made in [default: the current directory]
        #[arg(long = "in", value_name = "DIRECTORY")]
        parent_directory: Option<PathBuf>,
    },
}
