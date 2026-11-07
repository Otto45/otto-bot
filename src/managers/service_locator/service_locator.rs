use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::managers::configuration_providers::configuration_provider::ConfigurationProvider;
use crate::managers::data_layer::managers::gateways::session_store::session_store_gateway::SessionStoreGateway;
use crate::managers::data_layer::managers::gateways::stack_pack_store::stack_pack_store_gateway::StackPackStoreGateway;
use crate::managers::engines::agent_run::agent_run_engine::AgentRunEngine;
use crate::managers::engines::agent_run::processors::architecture_lint::architecture_lint_processor::ArchitectureLintProcessor;
use crate::managers::engines::agent_run::processors::patch::patch_processor::PatchProcessor;
use crate::managers::engines::agent_run::processors::project_map::project_map_processor::ProjectMapProcessor;
use crate::managers::engines::agent_run::processors::prompt::prompt_processor::PromptProcessor;
use crate::managers::engines::agent_run::processors::scaffold::scaffold_processor::ScaffoldProcessor;
use crate::managers::engines::agent_run::processors::tool::tool_processor::ToolProcessor;
use crate::managers::engines::agent_run::processors::work_plan::work_plan_processor::WorkPlanProcessor;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::gateways::command_runner::command_runner_gateway::CommandRunnerGateway;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connector::ResponsesWebSocketConnector;
use crate::managers::gateways::openai_responses::openai_responses_gateway::OpenAiResponsesGateway;
use crate::managers::gateways::workspace::workspace_gateway::WorkspaceGateway;
use crate::managers::models::settings::agent_settings::AgentSettings;
use crate::managers::models::settings::openai_responses_settings::OpenAiResponsesSettings;
use crate::managers::processors::session::session_processor::SessionProcessor;
use crate::managers::service_locator::service_locator_seams::ServiceLocatorSeams;
use crate::managers::service_locator::service_locator_seams_protocol::ServiceLocatorSeamsProtocol;

/// A singleton's close(), registered when the singleton is created.
type CloseCallback = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Builds, owns and closes every dependency except the DataFacade. Every singleton is created on first use and
/// cached; the Domain Facade, its managers and the DataFacade share them (`Arc`), so the locator's methods take
/// `&self` and each cache sits behind a lock.
pub struct ServiceLocator {
    seams: Arc<dyn ServiceLocatorSeamsProtocol>,
    configuration_provider: Mutex<Option<Arc<ConfigurationProvider>>>,
    openai_responses_gateway: Mutex<Option<Arc<OpenAiResponsesGateway>>>,
    workspace_gateway: Mutex<Option<Arc<WorkspaceGateway>>>,
    command_runner_gateway: Mutex<Option<Arc<CommandRunnerGateway>>>,
    session_store_gateway: Mutex<Option<Arc<SessionStoreGateway>>>,
    stack_pack_store_gateway: Mutex<Option<Arc<StackPackStoreGateway>>>,
    session_processor: Mutex<Option<Arc<SessionProcessor>>>,
    prompt_processor: Mutex<Option<Arc<PromptProcessor>>>,
    tool_processor: Mutex<Option<Arc<ToolProcessor>>>,
    patch_processor: Mutex<Option<Arc<PatchProcessor>>>,
    project_map_processor: Mutex<Option<Arc<ProjectMapProcessor>>>,
    architecture_lint_processor: Mutex<Option<Arc<ArchitectureLintProcessor>>>,
    scaffold_processor: Mutex<Option<Arc<ScaffoldProcessor>>>,
    work_plan_processor: Mutex<Option<Arc<WorkPlanProcessor>>>,
    agent_run_engine: Mutex<Option<Arc<AgentRunEngine>>>,
    // The close() of every singleton created so far, in creation order.
    close_callbacks: Mutex<Vec<CloseCallback>>,
}

impl Default for ServiceLocator {
    fn default() -> Self {
        ServiceLocator::new()
    }
}

impl ServiceLocator {
    pub fn new() -> Self {
        ServiceLocator::with_seams(Arc::new(ServiceLocatorSeams))
    }

    /// The production locator with other seams (e.g. a test's, which swap what reaches outside the process).
    pub fn with_seams(seams: Arc<dyn ServiceLocatorSeamsProtocol>) -> Self {
        ServiceLocator {
            seams,
            configuration_provider: Mutex::new(None),
            openai_responses_gateway: Mutex::new(None),
            workspace_gateway: Mutex::new(None),
            command_runner_gateway: Mutex::new(None),
            session_store_gateway: Mutex::new(None),
            stack_pack_store_gateway: Mutex::new(None),
            session_processor: Mutex::new(None),
            prompt_processor: Mutex::new(None),
            tool_processor: Mutex::new(None),
            patch_processor: Mutex::new(None),
            project_map_processor: Mutex::new(None),
            architecture_lint_processor: Mutex::new(None),
            scaffold_processor: Mutex::new(None),
            work_plan_processor: Mutex::new(None),
            agent_run_engine: Mutex::new(None),
            close_callbacks: Mutex::new(Vec::new()),
        }
    }

    pub fn get_configuration_provider(&self) -> ApplicationResult<Arc<ConfigurationProvider>> {
        ServiceLocator::get_or_create(&self.configuration_provider, || {
            Ok(Arc::new(ConfigurationProvider::new(
                self.seams.create_configuration_source_provider(),
            )?))
        })
    }

    pub fn get_openai_responses_gateway(&self) -> ApplicationResult<Arc<OpenAiResponsesGateway>> {
        ServiceLocator::get_or_create(&self.openai_responses_gateway, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let openai_responses_settings: &OpenAiResponsesSettings =
                configuration_provider.get_openai_responses_settings();
            let http_client: reqwest::Client = self
                .seams
                .create_openai_responses_http_client(openai_responses_settings)?;

            let web_socket_connector: ResponsesWebSocketConnector = self
                .seams
                .create_openai_responses_web_socket_connector(openai_responses_settings)?;

            let openai_responses_gateway: Arc<OpenAiResponsesGateway> = Arc::new(OpenAiResponsesGateway::new(
                http_client,
                web_socket_connector,
                openai_responses_settings.transport,
                openai_responses_settings.base_url.clone(),
                openai_responses_settings.api_key.clone(),
                openai_responses_settings.model.clone(),
            ));
            let gateway_to_close: Arc<OpenAiResponsesGateway> = openai_responses_gateway.clone();
            self.register_close(Box::new(move || {
                Box::pin(async move { gateway_to_close.close().await })
            }));
            Ok(openai_responses_gateway)
        })
    }

    pub fn get_workspace_gateway(&self) -> ApplicationResult<Arc<WorkspaceGateway>> {
        ServiceLocator::get_or_create(&self.workspace_gateway, || {
            let workspace_gateway: Arc<WorkspaceGateway> = Arc::new(WorkspaceGateway::new());
            let gateway_to_close: Arc<WorkspaceGateway> = workspace_gateway.clone();
            self.register_close(Box::new(move || Box::pin(async move { gateway_to_close.close() })));
            Ok(workspace_gateway)
        })
    }

    pub fn get_command_runner_gateway(&self) -> ApplicationResult<Arc<CommandRunnerGateway>> {
        ServiceLocator::get_or_create(&self.command_runner_gateway, || {
            let command_runner_gateway: Arc<CommandRunnerGateway> = Arc::new(CommandRunnerGateway::new());
            let gateway_to_close: Arc<CommandRunnerGateway> = command_runner_gateway.clone();
            self.register_close(Box::new(move || Box::pin(async move { gateway_to_close.close() })));
            Ok(command_runner_gateway)
        })
    }

    pub fn get_session_store_gateway(&self) -> ApplicationResult<Arc<SessionStoreGateway>> {
        ServiceLocator::get_or_create(&self.session_store_gateway, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let session_store_gateway: Arc<SessionStoreGateway> = Arc::new(SessionStoreGateway::new(
                configuration_provider.get_storage_settings().sessions_directory.clone(),
            ));
            let gateway_to_close: Arc<SessionStoreGateway> = session_store_gateway.clone();
            self.register_close(Box::new(move || Box::pin(async move { gateway_to_close.close() })));
            Ok(session_store_gateway)
        })
    }

    pub fn get_stack_pack_store_gateway(&self) -> ApplicationResult<Arc<StackPackStoreGateway>> {
        ServiceLocator::get_or_create(&self.stack_pack_store_gateway, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let stack_pack_store_gateway: Arc<StackPackStoreGateway> = Arc::new(StackPackStoreGateway::new(
                configuration_provider
                    .get_storage_settings()
                    .stack_packs_directory
                    .clone(),
            ));
            let gateway_to_close: Arc<StackPackStoreGateway> = stack_pack_store_gateway.clone();
            self.register_close(Box::new(move || Box::pin(async move { gateway_to_close.close() })));
            Ok(stack_pack_store_gateway)
        })
    }

    pub fn get_session_processor(&self) -> ApplicationResult<Arc<SessionProcessor>> {
        ServiceLocator::get_or_create(&self.session_processor, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            Ok(Arc::new(SessionProcessor::new(
                configuration_provider.get_agent_settings().default_service_tier,
            )))
        })
    }

    pub fn get_prompt_processor(&self) -> ApplicationResult<Arc<PromptProcessor>> {
        ServiceLocator::get_or_create(&self.prompt_processor, || Ok(Arc::new(PromptProcessor)))
    }

    pub fn get_tool_processor(&self) -> ApplicationResult<Arc<ToolProcessor>> {
        ServiceLocator::get_or_create(&self.tool_processor, || Ok(Arc::new(ToolProcessor)))
    }

    pub fn get_patch_processor(&self) -> ApplicationResult<Arc<PatchProcessor>> {
        ServiceLocator::get_or_create(&self.patch_processor, || Ok(Arc::new(PatchProcessor)))
    }

    pub fn get_project_map_processor(&self) -> ApplicationResult<Arc<ProjectMapProcessor>> {
        ServiceLocator::get_or_create(&self.project_map_processor, || Ok(Arc::new(ProjectMapProcessor)))
    }

    pub fn get_architecture_lint_processor(&self) -> ApplicationResult<Arc<ArchitectureLintProcessor>> {
        ServiceLocator::get_or_create(&self.architecture_lint_processor, || {
            Ok(Arc::new(ArchitectureLintProcessor))
        })
    }

    pub fn get_scaffold_processor(&self) -> ApplicationResult<Arc<ScaffoldProcessor>> {
        ServiceLocator::get_or_create(&self.scaffold_processor, || Ok(Arc::new(ScaffoldProcessor)))
    }

    pub fn get_work_plan_processor(&self) -> ApplicationResult<Arc<WorkPlanProcessor>> {
        ServiceLocator::get_or_create(&self.work_plan_processor, || Ok(Arc::new(WorkPlanProcessor)))
    }

    pub fn get_agent_run_engine(&self) -> ApplicationResult<Arc<AgentRunEngine>> {
        ServiceLocator::get_or_create(&self.agent_run_engine, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let agent_settings: &AgentSettings = configuration_provider.get_agent_settings();
            // The validator keeps the timeout positive and finite; one too long for a Duration is refused here.
            let check_timeout: Duration = Duration::try_from_secs_f64(agent_settings.check_timeout_seconds).map_err(
                |_: std::time::TryFromFloatSecsError| {
                    ApplicationBaseException::new(
                        ConfigurationProviderException::InvalidConfiguration,
                        "Agent configuration is invalid",
                        ContextualData::new()
                            .with("setting_name", "agent")
                            .with("errors", "check_timeout_seconds is too long"),
                    )
                },
            )?;

            // Holds nothing of its own to release, so it registers no close(); the gateways it uses already have.
            Ok(Arc::new(AgentRunEngine::new(
                self.get_openai_responses_gateway()?,
                self.get_workspace_gateway()?,
                self.get_command_runner_gateway()?,
                self.get_prompt_processor()?,
                self.get_tool_processor()?,
                self.get_patch_processor()?,
                self.get_project_map_processor()?,
                self.get_architecture_lint_processor()?,
                self.get_scaffold_processor()?,
                self.get_work_plan_processor()?,
                agent_settings.max_turns,
                check_timeout,
            )))
        })
    }

    /// Closes every singleton this locator created, in reverse order of creation. The Domain Facade calls this last,
    /// after its managers and the DataFacade, so nothing is closed while still in use.
    pub async fn close(&self) {
        let close_callbacks: Vec<CloseCallback> = std::mem::take(&mut *ServiceLocator::lock(&self.close_callbacks));

        for close_callback in close_callbacks.into_iter().rev() {
            close_callback().await;
        }
    }

    fn register_close(&self, close_callback: CloseCallback) {
        ServiceLocator::lock(&self.close_callbacks).push(close_callback);
    }

    // The cache is locked while its singleton is created, so two callers at once create it once. Each singleton only
    // asks for the ones it depends on, which never ask for it, so no lock is ever taken twice.
    fn get_or_create<T>(
        slot: &Mutex<Option<Arc<T>>>,
        create: impl FnOnce() -> ApplicationResult<Arc<T>>,
    ) -> ApplicationResult<Arc<T>> {
        let mut cached: MutexGuard<'_, Option<Arc<T>>> = ServiceLocator::lock(slot);
        if let Some(singleton) = cached.as_ref() {
            return Ok(singleton.clone());
        }

        let singleton: Arc<T> = create()?;
        *cached = Some(singleton.clone());
        Ok(singleton)
    }

    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
