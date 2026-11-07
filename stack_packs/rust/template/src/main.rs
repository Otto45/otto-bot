use std::panic::PanicHookInfo;
use std::process::ExitCode;

use example_app::example_app_facade::ExampleAppFacade;
use example_app::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use example_app::managers::exceptions::contextual_data::ContextualData;
use example_app::managers::logger::application_logger::ApplicationLogger;
use example_app::managers::logger::log_level::LogLevel;
use example_app::managers::logger::panic_exception::PanicException;
use example_app::managers::models::audit_event::AuditEvent;
use example_app::managers::models::customer::Customer;
use example_app::managers::models::greeting::Greeting;
use example_app::managers::models::user_profile::UserProfile;

// Entry point: calls the Domain Facade, and logs what fails.
fn main() -> ExitCode {
    ApplicationLogger::configure(LogLevel::Info, None);
    std::panic::set_hook(Box::new(log_panic));

    let exit_code: ExitCode = match tokio::runtime::Runtime::new() {
        Err(error) => {
            ApplicationLogger::error(
                "The app's runtime could not be started",
                ContextualData::new(),
                Some(&error),
            );
            ExitCode::FAILURE
        }
        Ok(runtime) => match runtime.block_on(run()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(exception) => handle_application_exception(&exception),
        },
    };

    ApplicationLogger::close();
    exit_code
}

async fn run() -> ApplicationResult<()> {
    let example_app_facade: ExampleAppFacade = ExampleAppFacade::new()?;

    // There is no `async with`: the facade is closed after it is used, whether using it failed or not.
    let used: ApplicationResult<()> = use_facade(&example_app_facade).await;
    example_app_facade.close().await;
    used
}

async fn use_facade(example_app_facade: &ExampleAppFacade) -> ApplicationResult<()> {
    example_app_facade
        .save_customer(&Customer {
            id: "1".to_string(),
            name: "World".to_string(),
            email: "world@example.com".to_string(),
        })
        .await?;

    let greeting: Greeting = example_app_facade.get_customer_greeting("1").await?;
    let audit_events: Vec<AuditEvent> = example_app_facade.get_customer_audit_events("1").await?;
    let user_profile: UserProfile = example_app_facade.get_user_profile(1).await?;
    let onboarded_customer: Customer = example_app_facade.onboard_customer(2).await?;

    println!("{}", greeting.message);
    println!("Audit events recorded: {}", audit_events.len());
    println!(
        "User profile: {} ({}) works at {}",
        user_profile.name, user_profile.email, user_profile.company_name
    );
    println!(
        "Onboarded customer {}: {}",
        onboarded_customer.id, onboarded_customer.name
    );
    Ok(())
}

fn handle_application_exception(exception: &ApplicationBaseException) -> ExitCode {
    ApplicationLogger::log_exception(exception, LogLevel::Error);
    ExitCode::FAILURE
}

// Logs a panic nothing caught, as every other line is logged, before the thread (or the app) stops.
fn log_panic(panic_hook_info: &PanicHookInfo<'_>) {
    let panic_exception: PanicException = PanicException::from_panic_hook_info(panic_hook_info);
    ApplicationLogger::error(
        "The app stopped on an unexpected error",
        ContextualData::new(),
        Some(&panic_exception),
    );
}
