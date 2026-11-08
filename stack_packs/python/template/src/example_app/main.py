import asyncio

from example_app.example_app_facade import ExampleAppFacade
from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.logger.application_logger import ApplicationLogger
from example_app.managers.logger.log_level import LogLevel
from example_app.managers.logger.uncaught_exception_logger import UncaughtExceptionLogger
from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.models.greeting import Greeting
from example_app.managers.models.user_profile import UserProfile


# Entry point (example-app): calls the Domain Facade, and logs what fails.
def main() -> int:
    ApplicationLogger.configure(LogLevel.INFO)
    UncaughtExceptionLogger.install()
    try:
        asyncio.run(_run())
        return 0
    except ApplicationBaseException as exception:
        ApplicationLogger.log_exception(exception, LogLevel.ERROR)
        return 1
    except Exception as exception:
        ApplicationLogger.error("The app stopped on an unexpected error", None, exception)
        return 1
    finally:
        ApplicationLogger.close()


async def _run() -> None:
    UncaughtExceptionLogger.install_for_event_loop()

    async with ExampleAppFacade() as example_app_facade:
        await example_app_facade.save_customer(Customer(id="1", name="World", email="world@example.com"))

        greeting: Greeting = await example_app_facade.get_customer_greeting("1")
        audit_events: list[AuditEvent] = await example_app_facade.get_customer_audit_events("1")
        user_profile: UserProfile = await example_app_facade.get_user_profile(1)
        onboarded_customer: Customer = await example_app_facade.onboard_customer(2)

        print(greeting.message)
        print(f"Audit events recorded: {len(audit_events)}")
        print(f"User profile: {user_profile.name} ({user_profile.email}) works at {user_profile.company_name}")
        print(f"Onboarded customer {onboarded_customer.id}: {onboarded_customer.name}")
