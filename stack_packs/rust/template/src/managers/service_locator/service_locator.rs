use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::managers::configuration_providers::configuration_provider::ConfigurationProvider;
use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::InMemoryDocumentDatabaseClient;
use crate::managers::data_layer::managers::gateways::document_database::document_database_gateway::DocumentDatabaseGateway;
use crate::managers::data_layer::managers::gateways::relational_database::relational_database_gateway::RelationalDatabaseGateway;
use crate::managers::engines::customer_onboarding::customer_onboarding_engine::CustomerOnboardingEngine;
use crate::managers::engines::customer_onboarding::processors::onboarding::onboarding_processor::OnboardingProcessor;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::gateways::json_placeholder::json_placeholder_gateway::JsonPlaceholderGateway;
use crate::managers::models::settings::json_placeholder_settings::JsonPlaceholderSettings;
use crate::managers::processors::customer::customer_processor::CustomerProcessor;
use crate::managers::processors::greeting::greeting_processor::GreetingProcessor;
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
    relational_database_gateway: Mutex<Option<Arc<RelationalDatabaseGateway>>>,
    document_database_gateway: Mutex<Option<Arc<DocumentDatabaseGateway>>>,
    json_placeholder_gateway: Mutex<Option<Arc<JsonPlaceholderGateway>>>,
    greeting_processor: Mutex<Option<Arc<GreetingProcessor>>>,
    customer_processor: Mutex<Option<Arc<CustomerProcessor>>>,
    onboarding_processor: Mutex<Option<Arc<OnboardingProcessor>>>,
    customer_onboarding_engine: Mutex<Option<Arc<CustomerOnboardingEngine>>>,
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
            relational_database_gateway: Mutex::new(None),
            document_database_gateway: Mutex::new(None),
            json_placeholder_gateway: Mutex::new(None),
            greeting_processor: Mutex::new(None),
            customer_processor: Mutex::new(None),
            onboarding_processor: Mutex::new(None),
            customer_onboarding_engine: Mutex::new(None),
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

    pub fn get_relational_database_gateway(&self) -> ApplicationResult<Arc<RelationalDatabaseGateway>> {
        ServiceLocator::get_or_create(&self.relational_database_gateway, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let pool: sqlx::PgPool = self
                .seams
                .create_relational_database_pool(configuration_provider.get_relational_database_settings())?;

            let relational_database_gateway: Arc<RelationalDatabaseGateway> =
                Arc::new(RelationalDatabaseGateway::new(pool));
            let gateway_to_close: Arc<RelationalDatabaseGateway> = relational_database_gateway.clone();
            self.register_close(Box::new(move || {
                Box::pin(async move { gateway_to_close.close().await })
            }));
            Ok(relational_database_gateway)
        })
    }

    pub fn get_document_database_gateway(&self) -> ApplicationResult<Arc<DocumentDatabaseGateway>> {
        ServiceLocator::get_or_create(&self.document_database_gateway, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let document_database_client: InMemoryDocumentDatabaseClient = self
                .seams
                .create_document_database_client(configuration_provider.get_document_database_settings());

            let document_database_gateway: Arc<DocumentDatabaseGateway> =
                Arc::new(DocumentDatabaseGateway::new(document_database_client));
            let gateway_to_close: Arc<DocumentDatabaseGateway> = document_database_gateway.clone();
            self.register_close(Box::new(move || {
                Box::pin(async move { gateway_to_close.close().await })
            }));
            Ok(document_database_gateway)
        })
    }

    pub fn get_json_placeholder_gateway(&self) -> ApplicationResult<Arc<JsonPlaceholderGateway>> {
        ServiceLocator::get_or_create(&self.json_placeholder_gateway, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let json_placeholder_settings: &JsonPlaceholderSettings =
                configuration_provider.get_json_placeholder_settings();
            let http_client: reqwest::Client = self
                .seams
                .create_json_placeholder_http_client(json_placeholder_settings)?;

            let json_placeholder_gateway: Arc<JsonPlaceholderGateway> = Arc::new(JsonPlaceholderGateway::new(
                http_client,
                json_placeholder_settings.base_url.clone(),
            ));
            let gateway_to_close: Arc<JsonPlaceholderGateway> = json_placeholder_gateway.clone();
            self.register_close(Box::new(move || Box::pin(async move { gateway_to_close.close() })));
            Ok(json_placeholder_gateway)
        })
    }

    pub fn get_greeting_processor(&self) -> ApplicationResult<Arc<GreetingProcessor>> {
        ServiceLocator::get_or_create(&self.greeting_processor, || {
            let configuration_provider: Arc<ConfigurationProvider> = self.get_configuration_provider()?;
            let greeting_prefix: String = configuration_provider.get_example_settings().greeting_prefix.clone();
            Ok(Arc::new(GreetingProcessor::new(greeting_prefix)))
        })
    }

    pub fn get_customer_processor(&self) -> ApplicationResult<Arc<CustomerProcessor>> {
        ServiceLocator::get_or_create(&self.customer_processor, || Ok(Arc::new(CustomerProcessor)))
    }

    pub fn get_onboarding_processor(&self) -> ApplicationResult<Arc<OnboardingProcessor>> {
        ServiceLocator::get_or_create(&self.onboarding_processor, || Ok(Arc::new(OnboardingProcessor)))
    }

    pub fn get_customer_onboarding_engine(&self) -> ApplicationResult<Arc<CustomerOnboardingEngine>> {
        ServiceLocator::get_or_create(&self.customer_onboarding_engine, || {
            let json_placeholder_gateway: Arc<JsonPlaceholderGateway> = self.get_json_placeholder_gateway()?;
            let onboarding_processor: Arc<OnboardingProcessor> = self.get_onboarding_processor()?;

            // Holds nothing of its own to release, so it registers no close(); the gateway it uses already has.
            Ok(Arc::new(CustomerOnboardingEngine::new(
                json_placeholder_gateway,
                onboarding_processor,
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
