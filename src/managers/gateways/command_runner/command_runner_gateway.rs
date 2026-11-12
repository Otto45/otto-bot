use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{ChildStderr, ChildStdout};

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::command_runner_gateway_exception::CommandRunnerGatewayException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::gateways::command_runner::guards::process_group_guard::ProcessGroupGuard;
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

// How much of what a command writes to each of stdout and stderr is kept: its end.
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const READ_CHUNK_BYTES: usize = 16 * 1024;

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
            // The command leads a process group of its own, so that it and everything it starts can be stopped
            // together (ProcessGroupGuard) when it takes too long or its run is cancelled.
            .process_group(0)
            .kill_on_drop(true);
        process.env_clear();
        for (name, value) in std::env::vars_os() {
            if CommandRunnerGateway::is_inherited(&name.to_string_lossy()) {
                process.env(name, value);
            }
        }

        let mut child: tokio::process::Child = process.spawn().map_err(|exception: std::io::Error| {
            ApplicationBaseException::new(
                CommandRunnerGatewayException::CommandStart,
                format!("{program} could not be started"),
                context.clone().with("error_type", format!("{:?}", exception.kind())),
            )
            .with_source(exception)
        })?;
        // Stops the command and everything it started if this is dropped before the command has ended.
        let process_group_guard: ProcessGroupGuard = ProcessGroupGuard::new(child.id());
        let stdout: Option<ChildStdout> = child.stdout.take();
        let stderr: Option<ChildStderr> = child.stderr.take();

        // Kept outside what is timed, so what the command wrote before it was stopped is still there.
        let mut stdout_end: Vec<u8> = Vec::new();
        let mut stderr_end: Vec<u8> = Vec::new();
        let waited: Result<std::io::Result<ExitStatus>, tokio::time::error::Elapsed> =
            tokio::time::timeout(timeout, async {
                let (exit_status, (), ()): (std::io::Result<ExitStatus>, (), ()) = tokio::join!(
                    child.wait(),
                    CommandRunnerGateway::read_end(stdout, &mut stdout_end),
                    CommandRunnerGateway::read_end(stderr, &mut stderr_end),
                );
                exit_status
            })
            .await;
        let output: String = CommandRunnerGateway::to_output(&stdout_end, &stderr_end);

        match waited {
            // Dropping the guard stops the command's processes.
            Err(_) => Ok(CommandResult {
                exit_code: None,
                timed_out: true,
                output,
            }),
            Ok(Err(exception)) => Err(ApplicationBaseException::new(
                CommandRunnerGatewayException::CommandStart,
                format!("{program} failed while running"),
                context.with("error_type", format!("{:?}", exception.kind())),
            )
            .with_source(exception)),
            Ok(Ok(exit_status)) => {
                process_group_guard.release();
                Ok(CommandResult {
                    exit_code: exit_status.code(),
                    timed_out: false,
                    output,
                })
            }
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

    // Reads what the command writes until it closes the stream, keeping its last MAX_OUTPUT_BYTES: the end is where a
    // compiler or a linter says what is wrong, and a command that writes without end must not fill the memory. A
    // stream that can't be read any more is taken as closed.
    async fn read_end<Stream: AsyncRead + Unpin>(stream: Option<Stream>, end: &mut Vec<u8>) {
        let Some(mut stream) = stream else {
            return;
        };
        let mut chunk: Vec<u8> = vec![0; READ_CHUNK_BYTES];
        loop {
            match stream.read(&mut chunk).await {
                Ok(0) | Err(_) => return,
                Ok(read_count) => {
                    end.extend_from_slice(&chunk[..read_count]);
                    // Trimmed once it holds twice what is kept, so each byte is moved at most once.
                    if end.len() > 2 * MAX_OUTPUT_BYTES {
                        end.drain(..end.len() - MAX_OUTPUT_BYTES);
                    }
                }
            }
        }
    }

    // What the command wrote to stdout, then stderr, each cut to its end.
    fn to_output(stdout_end: &[u8], stderr_end: &[u8]) -> String {
        let mut output: String = CommandRunnerGateway::to_text(stdout_end);
        let stderr: String = CommandRunnerGateway::to_text(stderr_end);
        if !output.is_empty() && !stderr.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(&stderr);
        output
    }

    fn to_text(end: &[u8]) -> String {
        let kept: &[u8] = &end[end.len().saturating_sub(MAX_OUTPUT_BYTES)..];
        String::from_utf8_lossy(kept).into_owned()
    }
}
