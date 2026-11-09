import { ConfigurationProvider } from "#src/managers/configuration_providers/configuration_provider.ts";
import { DocumentDatabaseGateway } from "#src/managers/data_layer/managers/gateways/document_database/document_database_gateway.ts";
import { RelationalDatabaseGateway } from "#src/managers/data_layer/managers/gateways/relational_database/relational_database_gateway.ts";
import { CustomerOnboardingEngine } from "#src/managers/engines/customer_onboarding/customer_onboarding_engine.ts";
import { OnboardingProcessor } from "#src/managers/engines/customer_onboarding/processors/onboarding/onboarding_processor.ts";
import { JsonPlaceholderGateway } from "#src/managers/gateways/json_placeholder/json_placeholder_gateway.ts";
import type { JsonPlaceholderSettings } from "#src/managers/models/settings/json_placeholder_settings.ts";
import { CustomerProcessor } from "#src/managers/processors/customer/customer_processor.ts";
import { GreetingProcessor } from "#src/managers/processors/greeting/greeting_processor.ts";
import { ServiceLocatorSeams } from "#src/managers/service_locator/service_locator_seams.ts";
import type { ServiceLocatorSeamsProtocol } from "#src/managers/service_locator/service_locator_seams_protocol.ts";

// Builds, owns and closes every dependency except the DataFacade. Every singleton is created on first use and cached;
// the Domain Facade, its managers and the DataFacade share them. Its get* methods are synchronous, so with one thread
// running JavaScript no two callers are ever inside one at once, and each singleton is created once.
export class ServiceLocator {
  private readonly seams: ServiceLocatorSeamsProtocol;
  private configurationProvider: ConfigurationProvider | undefined = undefined;
  private relationalDatabaseGateway: RelationalDatabaseGateway | undefined = undefined;
  private documentDatabaseGateway: DocumentDatabaseGateway | undefined = undefined;
  private jsonPlaceholderGateway: JsonPlaceholderGateway | undefined = undefined;
  private greetingProcessor: GreetingProcessor | undefined = undefined;
  private customerProcessor: CustomerProcessor | undefined = undefined;
  private onboardingProcessor: OnboardingProcessor | undefined = undefined;
  private customerOnboardingEngine: CustomerOnboardingEngine | undefined = undefined;
  // The close() of every singleton created so far, registered when it is created, in creation order.
  private closeCallbacks: (() => Promise<void> | void)[] = [];

  /** With no seams, the production ones; other seams (e.g. a test's) swap what reaches outside the process. */
  public constructor(seams: ServiceLocatorSeamsProtocol = new ServiceLocatorSeams()) {
    this.seams = seams;
  }

  public getConfigurationProvider(): ConfigurationProvider {
    this.configurationProvider ??= new ConfigurationProvider(this.seams.createConfigurationSourceProvider());
    return this.configurationProvider;
  }

  public getRelationalDatabaseGateway(): RelationalDatabaseGateway {
    if (this.relationalDatabaseGateway === undefined) {
      const configurationProvider: ConfigurationProvider = this.getConfigurationProvider();
      const relationalDatabaseGateway: RelationalDatabaseGateway = new RelationalDatabaseGateway(
        this.seams.createRelationalDatabasePool(configurationProvider.getRelationalDatabaseSettings()),
      );
      this.closeCallbacks.push(async (): Promise<void> => {
        await relationalDatabaseGateway.close();
      });
      this.relationalDatabaseGateway = relationalDatabaseGateway;
    }
    return this.relationalDatabaseGateway;
  }

  public getDocumentDatabaseGateway(): DocumentDatabaseGateway {
    if (this.documentDatabaseGateway === undefined) {
      const configurationProvider: ConfigurationProvider = this.getConfigurationProvider();
      const documentDatabaseGateway: DocumentDatabaseGateway = new DocumentDatabaseGateway(
        this.seams.createDocumentDatabaseClient(configurationProvider.getDocumentDatabaseSettings()),
      );
      this.closeCallbacks.push(async (): Promise<void> => {
        await documentDatabaseGateway.close();
      });
      this.documentDatabaseGateway = documentDatabaseGateway;
    }
    return this.documentDatabaseGateway;
  }

  public getJsonPlaceholderGateway(): JsonPlaceholderGateway {
    if (this.jsonPlaceholderGateway === undefined) {
      const jsonPlaceholderSettings: JsonPlaceholderSettings =
        this.getConfigurationProvider().getJsonPlaceholderSettings();
      const jsonPlaceholderGateway: JsonPlaceholderGateway = new JsonPlaceholderGateway(
        this.seams.createJsonPlaceholderFetch(jsonPlaceholderSettings),
        jsonPlaceholderSettings.baseUrl,
        jsonPlaceholderSettings.timeoutSeconds,
        jsonPlaceholderSettings.maxRetries,
      );
      this.closeCallbacks.push((): void => {
        jsonPlaceholderGateway.close();
      });
      this.jsonPlaceholderGateway = jsonPlaceholderGateway;
    }
    return this.jsonPlaceholderGateway;
  }

  public getGreetingProcessor(): GreetingProcessor {
    this.greetingProcessor ??= new GreetingProcessor(
      this.getConfigurationProvider().getExampleSettings().greetingPrefix,
    );
    return this.greetingProcessor;
  }

  public getCustomerProcessor(): CustomerProcessor {
    this.customerProcessor ??= new CustomerProcessor();
    return this.customerProcessor;
  }

  public getOnboardingProcessor(): OnboardingProcessor {
    this.onboardingProcessor ??= new OnboardingProcessor();
    return this.onboardingProcessor;
  }

  public getCustomerOnboardingEngine(): CustomerOnboardingEngine {
    // Holds nothing of its own to release, so it registers no close(); the gateway it uses already has.
    this.customerOnboardingEngine ??= new CustomerOnboardingEngine(
      this.getJsonPlaceholderGateway(),
      this.getOnboardingProcessor(),
    );
    return this.customerOnboardingEngine;
  }

  /**
   * Closes every singleton this locator created, in reverse order of creation. The Domain Facade calls this last, after
   * its managers and the DataFacade, so nothing is closed while still in use.
   */
  public async close(): Promise<void> {
    const closeCallbacks: (() => Promise<void> | void)[] = this.closeCallbacks;
    this.closeCallbacks = [];
    for (const closeCallback of closeCallbacks.reverse()) {
      await closeCallback();
    }
  }
}
