use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::managers::configuration_providers::configuration_provider::ConfigurationProvider;
use crate::managers::engines::agent_run::agent_run_engine::AgentRunEngine;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::gateways::openai_responses::openai_responses_gateway::OpenAiResponsesGateway;
use crate::managers::models::settings::openai_responses_settings::OpenAiResponsesSettings;
use crate::managers::processors::session::session_processor::SessionProcessor;
use crate::managers::service_locator::service_locator_seams::ServiceLocatorSeams;
use crate::managers::service_locator::service_locator_seams_protocol::ServiceLocatorSeamsProtocol;

/// A singleton's close(), registered when the singleton is created.
type CloseCallback = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Builds, owns and closes every dependency except the DataFacade. Every singleton is created on first use and
/// cached; the Domain Facade and its managers share them (`Arc`), so the locator's methods take `&self` and each
/// cache sits behind a lock.
pub struct ServiceLocator {
    seams: Arc<dyn ServiceLocatorSeamsProtocol>,
    configuration_provider: Mutex<Option<Arc<ConfigurationProvider>>>,
    openai_responses_gateway: Mutex<Option<Arc<OpenAiResponsesGateway>>>,
    session_processor: Mutex<Option<Arc<SessionProcessor>>>,
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
            session_processor: Mutex::new(None),
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

            let openai_responses_gateway: Arc<OpenAiResponsesGateway> = Arc::new(OpenAiResponsesGateway::new(
                http_client,
                openai_responses_settings.base_url.clone(),
                openai_responses_settings.api_key.clone(),
                openai_responses_settings.model.clone(),
            ));
            let gateway_to_close: Arc<OpenAiResponsesGateway> = openai_responses_gateway.clone();
            self.register_close(Box::new(move || Box::pin(async move { gateway_to_close.close() })));
            Ok(openai_responses_gateway)
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

    pub fn get_agent_run_engine(&self) -> ApplicationResult<Arc<AgentRunEngine>> {
        ServiceLocator::get_or_create(&self.agent_run_engine, || {
            let openai_responses_gateway: Arc<OpenAiResponsesGateway> = self.get_openai_responses_gateway()?;

            // Holds nothing of its own to release, so it registers no close(); the gateway it uses already has.
            Ok(Arc::new(AgentRunEngine::new(openai_responses_gateway)))
        })
    }

    /// Closes every singleton this locator created, in reverse order of creation. The Domain Facade calls this last,
    /// after its managers, so nothing is closed while still in use.
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
