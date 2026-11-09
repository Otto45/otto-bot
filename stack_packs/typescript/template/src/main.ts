import { ExampleAppFacade } from "#src/example_app_facade.ts";
import { ApplicationBaseException } from "#src/managers/exceptions/application_base_exception.ts";
import { ApplicationLogger } from "#src/managers/logger/application_logger.ts";
import { LogLevel } from "#src/managers/logger/log_level.ts";
import { UncaughtExceptionLogger } from "#src/managers/logger/uncaught_exception_logger.ts";
import type { AuditEvent } from "#src/managers/models/audit_event.ts";
import type { Customer } from "#src/managers/models/customer.ts";
import type { Greeting } from "#src/managers/models/greeting.ts";
import type { UserProfile } from "#src/managers/models/user_profile.ts";

// Entry point (example-app): calls the Domain Facade, and logs what fails.
async function main(): Promise<number> {
  ApplicationLogger.configure(LogLevel.Info);
  UncaughtExceptionLogger.install();
  try {
    await run();
    return 0;
  } catch (error: unknown) {
    if (error instanceof ApplicationBaseException) {
      ApplicationLogger.logException(error, LogLevel.Error);
    } else {
      ApplicationLogger.error(
        "The app stopped on an unexpected error",
        undefined,
        error instanceof Error ? error : new Error(String(error)),
      );
    }
    return 1;
  } finally {
    ApplicationLogger.close();
  }
}

async function run(): Promise<void> {
  await using exampleAppFacade: ExampleAppFacade = new ExampleAppFacade();

  await exampleAppFacade.saveCustomer({ id: "1", name: "World", email: "world@example.com" });

  const greeting: Greeting = await exampleAppFacade.getCustomerGreeting("1");
  const auditEvents: AuditEvent[] = await exampleAppFacade.getCustomerAuditEvents("1");
  const userProfile: UserProfile = await exampleAppFacade.getUserProfile(1);
  const onboardedCustomer: Customer = await exampleAppFacade.onboardCustomer(2);

  console.log(greeting.message);
  console.log(`Audit events recorded: ${auditEvents.length}`);
  console.log(`User profile: ${userProfile.name} (${userProfile.email}) works at ${userProfile.companyName}`);
  console.log(`Onboarded customer ${onboardedCustomer.id}: ${onboardedCustomer.name}`);
}

process.exitCode = await main();
