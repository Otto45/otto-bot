use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::command_runner_gateway_exception::CommandRunnerGatewayException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::command_result::CommandResult;

// The environment variables a command the harness runs inherits: what a toolchain needs to find itself, its caches,
// the network and the user's locale. Everything else is withheld (the model's API key, otto-bot's own settings, the
// service's token, a database's connection string, a cloud's credentials), because a check runs code the agent
// wrote, and what that code prints is sent to the model.
const INHERITED_ENVIRONMENT_VARIABLES: [&str; 22] = [
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "TERM",
    "LANG",
    "TZ",
    "TMPDIR",
    "TEMP",
    "TMP",
    "VIRTUAL_ENV",
    "RUSTFLAGS",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "no_proxy",
    "SYSTEMROOT",
];
// And those whose names start with one of these: a toolchain's own settings.
const INHERITED_ENVIRONMENT_VARIABLE_PREFIXES: [&str; 10] = [
    "LC_",
    "XDG_",
    "CARGO_",
    "RUSTUP_",
    "UV_",
    "PNPM_",
    "NPM_CONFIG_",
    "npm_config_",
    "NODE_",
    "COREPACK_",
];
// But none whose name says it holds a secret (a registry's token among a toolchain's settings, say).
const SECRET_NAME_PARTS: [&str; 5] = ["TOKEN", "SECRET", "PASSWORD", "CREDENTIAL", "_KEY"];

/// Runs commands (a stack pack's checks) in the workspace, with no input, collecting what they write, in an
/// environment that holds only what a toolchain needs.
pub struct CommandRunnerGateway;

impl Default for CommandRunnerGateway {
    fn default() -> Self {
        CommandRunnerGateway::new()
    }
}

impl CommandRunnerGateway {
    pub fn new() -> Self {
        CommandRunnerGateway
    }

    /// Runs `command` (the program, then its arguments) in `working_directory`, stopping it after `timeout`. A
    /// command that runs and fails is a result, not an exception; one that can't be started is an exception.
    pub async fn run(
        &self,
        working_directory: &Path,
        command: &[String],
        timeout: Duration,
    ) -> ApplicationResult<CommandResult> {
        let context: ContextualData = ContextualData::new().with("command", command.join(" "));
        let Some((program, arguments)) = command.split_first() else {
            return Err(ApplicationBaseException::new(
                CommandRunnerGatewayException::CommandStart,
                "The command is empty",
                context,
            ));
        };

        let mut process: tokio::process::Command = tokio::process::Command::new(program);
        process
            .args(arguments)
            .current_dir(working_directory)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // A run that is cancelled, or a command that takes too long, takes its process with it.
            .kill_on_drop(true);
        process.env_clear();
        for (name, value) in std::env::vars_os() {
            if CommandRunnerGateway::is_inherited(&name.to_string_lossy()) {
                process.env(name, value);
            }
        }

        let child: tokio::process::Child = process.spawn().map_err(|exception: std::io::Error| {
            ApplicationBaseException::new(
                CommandRunnerGatewayException::CommandStart,
                format!("{program} could not be started"),
                context.clone().with("error_type", format!("{:?}", exception.kind())),
            )
            .with_source(exception)
        })?;

        match tokio::time::timeout(timeout, child.wait_with_output()).await {
            // Dropping the wait dropped the process, which kill_on_drop stopped.
            Err(_) => Ok(CommandResult {
                exit_code: None,
                timed_out: true,
                output: String::new(),
            }),
            Ok(Err(exception)) => Err(ApplicationBaseException::new(
                CommandRunnerGatewayException::CommandStart,
                format!("{program} failed while running"),
                context.with("error_type", format!("{:?}", exception.kind())),
            )
            .with_source(exception)),
            Ok(Ok(output)) => Ok(CommandRunnerGateway::to_command_result(&output)),
        }
    }

    /// Holds nothing between commands, so there is nothing to close.
    pub fn close(&self) {}

    fn is_inherited(environment_variable_name: &str) -> bool {
        let named: bool = INHERITED_ENVIRONMENT_VARIABLES.contains(&environment_variable_name)
            || INHERITED_ENVIRONMENT_VARIABLE_PREFIXES
                .iter()
                .any(|prefix: &&str| environment_variable_name.starts_with(prefix));
        let upper_name: String = environment_variable_name.to_ascii_uppercase();
        named
            && !SECRET_NAME_PARTS
                .iter()
                .any(|secret_name_part: &&str| upper_name.contains(secret_name_part))
    }

    fn to_command_result(output: &Output) -> CommandResult {
        let mut combined_output: String = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
        if !combined_output.is_empty() && !stderr.is_empty() && !combined_output.ends_with('\n') {
            combined_output.push('\n');
        }
        combined_output.push_str(&stderr);
        CommandResult {
            exit_code: output.status.code(),
            timed_out: false,
            output: combined_output,
        }
    }
}
