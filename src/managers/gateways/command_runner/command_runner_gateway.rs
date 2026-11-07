use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::command_runner_gateway_exception::CommandRunnerGatewayException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::command_result::CommandResult;

// What a command the harness runs never inherits: the model's API key, which nothing it runs needs.
const WITHHELD_ENVIRONMENT_VARIABLES: [&str; 1] = ["OPENAI_API_KEY"];

/// Runs commands (a stack pack's checks) in the workspace, with no input, collecting what they write.
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
        for environment_variable_name in WITHHELD_ENVIRONMENT_VARIABLES {
            process.env_remove(environment_variable_name);
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
