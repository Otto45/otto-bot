from collections.abc import Awaitable, Callable
from typing import final

from example_app.managers.configuration_providers.configuration_provider import ConfigurationProvider
from example_app.managers.data_layer.managers.gateways.document_database.document_database_gateway import (
    DocumentDatabaseGateway,
)
from example_app.managers.data_layer.managers.gateways.relational_database.relational_database_gateway import (
    RelationalDatabaseGateway,
)
from example_app.managers.engines.customer_onboarding.customer_onboarding_engine import CustomerOnboardingEngine
from example_app.managers.engines.customer_onboarding.processors.onboarding.onboarding_processor import (
    OnboardingProcessor,
)
from example_app.managers.gateways.json_placeholder.json_placeholder_gateway import JsonPlaceholderGateway
from example_app.managers.processors.customer.customer_processor import CustomerProcessor
from example_app.managers.processors.greeting.greeting_processor import GreetingProcessor
from example_app.managers.service_locator.service_locator_seams import ServiceLocatorSeams
from example_app.managers.service_locator.service_locator_seams_protocol import ServiceLocatorSeamsProtocol


# Builds, owns and closes every dependency except the DataFacade. Every singleton is created on first use and cached;
# the Domain Facade, its managers and the DataFacade share them. Its get_* methods are synchronous, so on the event loop
# no two callers can be inside one at once, and each singleton is created once.
@final
class ServiceLocator:
    def __init__(self, seams: ServiceLocatorSeamsProtocol | None = None) -> None:
        """With no seams, the production ones; other seams (e.g. a test's) swap what reaches outside the process."""
        self._seams: ServiceLocatorSeamsProtocol = seams if seams is not None else ServiceLocatorSeams()
        self._configuration_provider: ConfigurationProvider | None = None
        self._relational_database_gateway: RelationalDatabaseGateway | None = None
        self._document_database_gateway: DocumentDatabaseGateway | None = None
        self._json_placeholder_gateway: JsonPlaceholderGateway | None = None
        self._greeting_processor: GreetingProcessor | None = None
        self._customer_processor: CustomerProcessor | None = None
        self._onboarding_processor: OnboardingProcessor | None = None
        self._customer_onboarding_engine: CustomerOnboardingEngine | None = None
        # The close() of every singleton created so far, registered when it is created, in creation order.
        self._close_callbacks: list[Callable[[], Awaitable[None]]] = []

    def get_configuration_provider(self) -> ConfigurationProvider:
        if self._configuration_provider is None:
            self._configuration_provider = ConfigurationProvider(self._seams.create_configuration_source_provider())
        return self._configuration_provider

    def get_relational_database_gateway(self) -> RelationalDatabaseGateway:
        if self._relational_database_gateway is None:
            configuration_provider: ConfigurationProvider = self.get_configuration_provider()
            relational_database_gateway: RelationalDatabaseGateway = RelationalDatabaseGateway(
                self._seams.create_relational_database_pool(configuration_provider.get_relational_database_settings())
            )
            self._close_callbacks.append(relational_database_gateway.close)
            self._relational_database_gateway = relational_database_gateway
        return self._relational_database_gateway

    def get_document_database_gateway(self) -> DocumentDatabaseGateway:
        if self._document_database_gateway is None:
            configuration_provider: ConfigurationProvider = self.get_configuration_provider()
            document_database_gateway: DocumentDatabaseGateway = DocumentDatabaseGateway(
                self._seams.create_document_database_client(configuration_provider.get_document_database_settings())
            )
            self._close_callbacks.append(document_database_gateway.close)
            self._document_database_gateway = document_database_gateway
        return self._document_database_gateway

    def get_json_placeholder_gateway(self) -> JsonPlaceholderGateway:
        if self._json_placeholder_gateway is None:
            configuration_provider: ConfigurationProvider = self.get_configuration_provider()
            json_placeholder_gateway: JsonPlaceholderGateway = JsonPlaceholderGateway(
                self._seams.create_json_placeholder_http_client(configuration_provider.get_json_placeholder_settings())
            )
            self._close_callbacks.append(json_placeholder_gateway.close)
            self._json_placeholder_gateway = json_placeholder_gateway
        return self._json_placeholder_gateway

    def get_greeting_processor(self) -> GreetingProcessor:
        if self._greeting_processor is None:
            configuration_provider: ConfigurationProvider = self.get_configuration_provider()
            self._greeting_processor = GreetingProcessor(configuration_provider.get_example_settings().greeting_prefix)
        return self._greeting_processor

    def get_customer_processor(self) -> CustomerProcessor:
        if self._customer_processor is None:
            self._customer_processor = CustomerProcessor()
        return self._customer_processor

    def get_onboarding_processor(self) -> OnboardingProcessor:
        if self._onboarding_processor is None:
            self._onboarding_processor = OnboardingProcessor()
        return self._onboarding_processor

    def get_customer_onboarding_engine(self) -> CustomerOnboardingEngine:
        if self._customer_onboarding_engine is None:
            # Holds nothing of its own to release, so it registers no close(); the gateway it uses already has.
            self._customer_onboarding_engine = CustomerOnboardingEngine(
                self.get_json_placeholder_gateway(), self.get_onboarding_processor()
            )
        return self._customer_onboarding_engine

    async def close(self) -> None:
        """Closes every singleton this locator created, in reverse order of creation. The Domain Facade calls this
        last, after its managers and the DataFacade, so nothing is closed while still in use."""
        close_callbacks: list[Callable[[], Awaitable[None]]] = self._close_callbacks
        self._close_callbacks = []
        for close_callback in reversed(close_callbacks):
            await close_callback()
