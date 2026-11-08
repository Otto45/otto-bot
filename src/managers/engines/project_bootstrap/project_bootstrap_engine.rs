use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::project_bootstrap::processors::app_name::app_name_processor::AppNameProcessor;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::project_bootstrap_engine_exception::ProjectBootstrapEngineException;
use crate::managers::gateways::command_runner::command_runner_gateway::CommandRunnerGateway;
use crate::managers::gateways::workspace::workspace_gateway::WorkspaceGateway;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::app_name::AppName;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::started_project::StartedProject;

// Starts an app from a stack pack's template, with no model: the template's files are copied into a new folder, its
// names (in file contents and paths) replaced with the app's, then the pack's formatters run over it, since a renamed
// name sorts and wraps differently (an import's place in its block, a line's length). The examples stay: they show
// the implementor every component kind, and the first plan replaces them.
pub struct ProjectBootstrapEngine {
    workspace_gateway: Arc<WorkspaceGateway>,
    command_runner_gateway: Arc<CommandRunnerGateway>,
    app_name_processor: Arc<AppNameProcessor>,
    format_timeout: Duration,
}

impl ProjectBootstrapEngine {
    pub fn new(
        workspace_gateway: Arc<WorkspaceGateway>,
        command_runner_gateway: Arc<CommandRunnerGateway>,
        app_name_processor: Arc<AppNameProcessor>,
        format_timeout: Duration,
    ) -> Self {
        ProjectBootstrapEngine {
            workspace_gateway,
            command_runner_gateway,
            app_name_processor,
            format_timeout,
        }
    }

    /// Makes `<parent_directory>/<the app's name in kebab case>` and copies the stack's template into it, renamed. The
    /// folder must be new or empty, so nothing of someone's is overwritten.
    pub async fn start_project(
        &self,
        stack_name: &str,
        app_name: &str,
        parent_directory: &Path,
        data_facade: &DataFacade,
    ) -> ApplicationResult<StartedProject> {
        let app_name: AppName = self.app_name_processor.to_app_name(app_name)?;
        let stack_packs: Vec<StackPack> = data_facade.get_stack_packs().await?;
        let Some(stack_pack) = stack_packs
            .iter()
            .find(|stack_pack: &&StackPack| stack_pack.name == stack_name)
        else {
            let stack_names: Vec<&str> = stack_packs
                .iter()
                .map(|stack_pack: &StackPack| stack_pack.name.as_str())
                .collect();
            return Err(ApplicationBaseException::new(
                ProjectBootstrapEngineException::UnknownStack,
                format!(
                    "There is no {stack_name} stack; the stacks are: {}",
                    stack_names.join(", ")
                ),
                ContextualData::new().with("stack_name", stack_name),
            ));
        };
        let Some(template_files) = data_facade.get_template_files(stack_pack).await? else {
            return Err(ApplicationBaseException::new(
                ProjectBootstrapEngineException::StackWithoutTemplate,
                format!("The {stack_name} stack has no template to start an app from"),
                ContextualData::new().with("stack_name", stack_name),
            ));
        };

        let workspace_directory: PathBuf = self
            .workspace_gateway
            .create_workspace_directory(&parent_directory.join(&app_name.kebab))
            .await?;
        for template_file in &template_files {
            let path: String =
                self.app_name_processor
                    .replace_names(&template_file.path, &stack_pack.name_replacements, &app_name);
            let content: String =
                self.app_name_processor
                    .replace_names(&template_file.content, &stack_pack.name_replacements, &app_name);
            self.workspace_gateway
                .write_file(&workspace_directory, &path, &content, true)
                .await?;
        }

        let formatted: bool = self.format(&workspace_directory, stack_pack).await;
        Ok(StartedProject {
            stack_name: stack_pack.name.clone(),
            app_name,
            workspace_directory,
            file_count: template_files.len(),
            formatted,
        })
    }

    // Runs the pack's formatters, in order, and says whether every one succeeded. One that fails or can't start (a
    // toolchain that isn't installed) doesn't stop the app being started: the agent's first run of the checks runs
    // them again and says what is wrong.
    async fn format(&self, workspace_directory: &Path, stack_pack: &StackPack) -> bool {
        let Some(stack_pack_format) = stack_pack.format.as_ref() else {
            return true;
        };
        let mut formatted: bool = true;
        for command in &stack_pack_format.commands {
            let context: ContextualData = ContextualData::new()
                .with("stack_name", stack_pack.name.as_str())
                .with("command", command.join(" "));
            match self
                .command_runner_gateway
                .run(workspace_directory, command, self.format_timeout)
                .await
            {
                Ok(command_result) if command_result.exit_code == Some(0) => {}
                Ok(command_result) => {
                    ApplicationLogger::warning(
                        "A new app's formatter failed",
                        context.with("timed_out", command_result.timed_out),
                        None,
                    );
                    formatted = false;
                }
                Err(exception) => {
                    ApplicationLogger::log_exception(&exception, LogLevel::Warning);
                    formatted = false;
                }
            }
        }
        formatted
    }
}
