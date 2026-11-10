# Domain Container architecture

This app follows the Domain Container architecture. These rules apply to every change, and the reasons given with
them say what a rule protects, so that a case the rules don't name is decided the same way.

## Tools
- Rust (edition 2024) with Cargo; Tokio for async; axum for HTTP services, with utoipa / `utoipa-axum` for their contract; serde for wire and resource models (`serde_path_to_error` names the field that failed); reqwest for outgoing HTTP; sqlx for Postgres; dbmate for migrations
- every change passes `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo run --bin export-openapi -- --check`
- `unsafe` is forbidden (`unsafe_code = "forbid"`); `module_inception` is allowed, because a folder and its main file share a name (`service_locator/service_locator.rs`)
- `Cargo.lock` is not kept: it names the package, so an app made from the template locks its own dependencies

## Layout
```
src/
    main.rs                     entry point (example-app): calls the Domain Facade, logs what fails
    bin/                        the other binaries' entry points; each only calls a service's main()
    lib.rs                      the crate: example_app_facade, managers (the domain container), services
    example_app_facade.rs       Domain Facade: public entry point, creates the ServiceLocator and DataFacade
    services/<name>/            anything that runs and listens, outside the domain container (see Service)
    managers/
        example_app_manager.rs  Manager: orchestrates workflows
        service_locator/        ServiceLocator, ServiceLocatorSeamsProtocol, ServiceLocatorSeams
        common/                 stateless helpers shared by more than one component
        configuration_providers/ config models -> validators -> settings
        processors/             domain logic the Manager delegates to, one subfolder per processor type
        engines/                multi-step tasks the Manager hands off whole, one subfolder per engine
        gateways/<system>/      one gateway per external system
        data_layer/             DataFacade -> DataManager -> data gateways (see Data layer)
        models/                 domain models; models/settings/ the settings
        exceptions/             ApplicationBaseException and its kinds
        logger/                 ApplicationLogger
migrations/                     the relational database's schema: one plain SQL file per change
openapi.json                    the HTTP service's contract, generated
```

## Code
- every `let` binding and closure parameter has its type written out (`let pool: PgPool = ...`, `|error: sqlx::Error|`); no lint enforces this, so it is checked by reading
  - exception: the bindings of `for`, `match`, `if let` and `while let` patterns, which Rust does not allow to be annotated
  - exception: values whose type only inference can name (an `async` block, a closure stored in a variable, a `tracing` subscriber stack)
- a Protocol is a trait, implemented in an `impl <Name>Protocol for <Type>` block
  - a Protocol trait is suffixed `Protocol`, lives in its own file, and declares method signatures only: no default method bodies, no associated state
  - a Protocol trait has no `async fn`, so it can be used as `dyn` (an `async fn` in a trait can't); the async methods of the domain container belong to structs
- each file holds the one struct, enum or trait it is named after, in snake_case (`GreetingProcessor` in `greeting_processor.rs`); every folder is a module declared in its `mod.rs`
- a group of static methods (validators, mappers, query builders, formatters, common helpers) is a unit struct with associated functions (`pub struct ValidatorString;`), never built and never registered

## Domain container
- a function is `async` when it does I/O or awaits something that does (a gateway's request, a driver's query, a facade or manager method that calls a gateway); a function that only computes is plain `fn`, whichever component it is in
  - prefer async wherever there is I/O: use the async API of a driver, client or file system (Tokio, sqlx, reqwest), never a blocking call inside async code; work that can only block (a library with no async API, heavy CPU work) runs in `tokio::task::spawn_blocking`
  - a client that stands in for an async driver (e.g. `InMemoryDocumentDatabaseClient`) keeps the driver's `async` API, so nothing changes when the real driver replaces it
  - a method that does nothing yet (e.g. a manager's `close()` that owns nothing) is plain `fn`, and becomes `async` with the first I/O it does
- every fallible operation returns `ApplicationResult<T>` (`Result<T, ApplicationBaseException>`) and propagates with `?`; nothing in the domain container panics on an expected failure
- all application logic a manager delegates to lives in a `processors/` folder one level below the manager's folder (e.g. `managers/processors/`)
  - one subfolder per type of processor (e.g. `managers/processors/greeting/`)
  - processor structs are suffixed with `Processor` (e.g. `GreetingProcessor` in `greeting_processor.rs`)
  - processors are singletons: the service locator caches the single instance (`Arc<GreetingProcessor>`) and exposes it via a `get_*` method (e.g. `get_greeting_processor()`), never `create_*`
  - logic exclusive to one processor that is too in-depth to code directly in the processor goes in structs with associated functions in subfolders under that processor's folder (e.g. `managers/processors/greeting/formatters/`)
- a task that is one step to a manager but many steps of different kinds underneath (external calls, decisions, reads and saves) goes in an engine, so manager methods stay short coordinations
  - one subfolder per engine under `managers/engines/` (e.g. `managers/engines/customer_onboarding/`), holding `<Name>Engine` in `<name>_engine.rs` (e.g. `CustomerOnboardingEngine` in `customer_onboarding_engine.rs`)
  - engines are singletons: the service locator builds one via a `get_*` method (e.g. `get_customer_onboarding_engine()`), passing its constructor the gateways and processors it uses
  - an engine's processors live in a `processors/` folder inside the engine's folder (e.g. `managers/engines/customer_onboarding/processors/onboarding/`), follow every processor rule, and are used by that engine only
  - an engine uses gateways, its own processors and the `DataFacade`, which the manager passes into each call (e.g. `onboard_customer(user_id, &self.data_facade)`) because the `ServiceLocator` never sees the `DataFacade`
  - an engine never uses a manager's processors, another engine, or a manager
  - an engine keeps no state between calls and owns nothing to release, so it has no `close()` and is never registered for closing
- managers never call other managers; anything they share goes through the `DataFacade`, a processor or an engine
- stateless helpers used by more than one component live in `managers/common/` (e.g. `common/calculators/retry_delay_calculator.rs`, `common/validators/validator_string.rs`, `common/formatters/validation_error_formatter.rs`)
  - associated functions only, no I/O; never registered in the `ServiceLocator`
  - never copy a common helper into a component as a private function

## Facade, locator and closing
- the domain facade creates the `ServiceLocator`, then the `DataFacade` (passing it the `ServiceLocator`)
  - the `ServiceLocator` never creates the `DataFacade`
  - the domain facade passes both the `ServiceLocator` and the `DataFacade` (`Arc<DataFacade>`) to every manager it creates
  - the domain facade owns the `DataFacade` and closes it; managers share it but never close it
  - the domain facade is created inside a Tokio runtime: the database pool starts its maintenance task when it is built
  - creating the facade validates all configuration, so invalid configuration makes `ExampleAppFacade::new()` fail before anything runs
- closing is explicit: nothing can be awaited while a value is dropped, so whoever creates a domain facade calls `close().await` on every path, whether using it succeeded or failed (see `src/main.rs` and `HttpService::serve`)
  - the domain facade closes its managers, then the `DataFacade`, then the `ServiceLocator` last
  - the `ServiceLocator` closes every singleton it caches in its `close()`, in reverse order of creation, through the close callback it registers when it creates one, so nothing is closed while something created after it still uses it
  - managers and the `DataManager` never close gateways; a manager's `close()` releases only what the manager itself owns
- the `ServiceLocator` caches each singleton behind a `Mutex<Option<Arc<T>>>`, hands out `Arc` clones, and its methods take `&self`; a `get_*` that can fail returns `ApplicationResult<Arc<T>>`
  - a singleton only asks for the singletons it depends on, which never ask for it, so no cache's lock is taken twice
- the objects that reach outside the process (the configuration source, the HTTP client, the database pool, the document client) are built by the locator's `create_*` seams, declared by `ServiceLocatorSeamsProtocol` and implemented for production by `ServiceLocatorSeams`
  - a seam takes the settings it needs as a parameter rather than asking the locator
  - only the seams know the default sources (the environment, the network, the drivers); `ServiceLocator::new()` uses `ServiceLocatorSeams`
  - whatever needs other seams (a test swapping the network or the environment) builds the same production locator with `ServiceLocator::with_seams(...)`, and its seams call `ServiceLocatorSeams` for every seam they keep, so the caching and wiring that run are always the production ones
  - add a dependency that reaches outside the process by adding a seam for the deepest object under it, and a `get_*` that builds its gateway from that seam

## Models
- domain models and settings are structs with public fields that derive `Debug, Clone, PartialEq` (and `Eq` where they can); nothing in the domain container holds a `mut` one, so they never change once made, and borrowing rules keep a value another component holds from changing under it
- config models and resource models derive `Default`, and every field of theirs is an `Option` (`#[serde(default)]`), so anything from outside deserializes and the validator can say what is wrong with it
- domain models never derive `Serialize`: what goes over the wire is a service's own wire model

## Exceptions
- the exceptions are one type, `ApplicationBaseException` (`managers/exceptions/application_base_exception.rs`): a message, a context, the error it was raised from, and the stack where it was raised (Rust errors carry none of their own, so one is captured when it is created); `kind()` says which exception it is
  - each component's exceptions are a family: an enum suffixed `Exception` in its own file (e.g. `RelationalDatabaseGatewayException::CustomerNotFound`), nested in `ExceptionKind`; a new exception is a new variant, and `name()` gives it the name it is logged under (e.g. `"CustomerNotFoundException"`)
  - code that treats a family alike matches the family (`ExceptionKind::JsonPlaceholderGateway(_)`), and a match that decides something for every exception (e.g. `HttpErrorMapper::to_status_code`) names every family, with no `_ =>` arm, so a new family doesn't compile until it is decided there
  - an exception raised from another error keeps it with `.with_source(error)`, which is logged with it
- an exception's context (`ContextualData`) holds primitives (`ContextValue`) under snake_case keys and never anything sensitive (no bound parameter values, no credentials); a context with nothing in it is `ContextualData::new()`
  - a driver with one error type is described by what kind of failure it was (`error_type`: `DatabaseError`, `ConnectError`, `StatusError`), and a Postgres error by its SQLSTATE (`postgres_error_code`)
- an exception raised for something the caller can put right (a value that is not valid, something that was not found) has a message written for the person using the app (e.g. `ExceptionKind::InvalidCustomer`: "The customer's name is required"), because a service answers with it as it is; what only engineers need goes in the context
- unexpected exceptions are panics, and every entry point installs a panic hook (see Logger)

## Configuration
- configuration is read from the environment only by `ConfigurationSourceProviderEnvironment`, inside its methods, never in a `static`, except where a service listens, which its entry point reads (see Service)
- the `ConfigurationProvider` requires its `ConfigurationSourceProviderProtocol`; only the `ServiceLocatorSeams` (`create_configuration_source_provider`) knows the default source
- each setting group flows config model -> validator -> settings: the `ConfigurationProvider` loads and validates every group when it is created, then maps each with `expect("validated by <Validator>")`

## Gateways
- anything that comes from outside the process (an API response, a database row, a document) is deserialized into a resource model (a serde or `sqlx::FromRow` struct with every field `Option`), checked by a validator, then mapped to a domain model; resource models never leave their gateway, which returns domain models only
  ```
  managers/gateways/json_placeholder/
      json_placeholder_gateway.rs     sends the request, translates reqwest errors into exceptions
      resource_models/                the JSON body deserialized, every field optional
      validators/                     check the resource has every value the domain model needs
      mappers/                        map the validated resource to a domain model
  ```
  - a gateway's `resource_models`, `validators`, `mappers`, `query_builders` and `request_builders` are private modules of the gateway's module (`mod mappers;`, not `pub mod`), so the compiler keeps them inside it
  - a mapper takes the validated resource by value, unwraps each field with `expect("validated by <Validator>")`, and has a doc comment saying it expects a validated resource; each `expect` states a fact the validator checked, and fails only if the validator is wrong
  - a deserialization error is formatted by `ValidationErrorFormatter`: `format` for serde through `serde_path_to_error` (JSON bodies, documents), `format_row` for sqlx, `format_whole` for an error about the input as a whole; both libraries stop at the first error
  - the gateway is where driver and HTTP errors are translated into `ApplicationBaseException`; nothing above it sees a library's error
- when building a gateway's request is too involved to inline, it goes in a `request_builders/` subfolder of the gateway (the counterpart of a data gateway's `query_builders/`)
- an HTTP gateway is given its `reqwest::Client` (built by a seam, with the configured timeout) and its base URL, which it joins to each path (reqwest has none of its own); the client holds nothing that outlives it, so the gateway's `close()` has nothing to do
- a gateway retries a request that failed for a reason that may pass on its own, so a brief outage of the other system doesn't fail the app's operation
  - transient: a connection that couldn't be made or timed out, and a status of 429, 500, 502, 503 or 504 (or the error a service's API documents as temporary, e.g. `server_error`, a rate limit); never what sending again won't change: any other 4xx (a malformed request, a bad key, an exhausted quota even when it comes as a 429), and a response the gateway can't read
  - only a request that is safe to send twice is retried: a read, or a write the other system makes idempotent (e.g. with an idempotency key the gateway sends); a write that could happen twice is not retried, and its failure is told as it is
  - up to a few times (a setting of the gateway's settings group, e.g. `max_retries`, 3 by default), after a wait that doubles from about a second, up to a cap, with a little random jitter so callers that failed together don't retry together; when the service says how long to wait (`Retry-After`), that wait, up to a cap
  - every gateway waits the same: the wait comes from the shared `RetryDelayCalculator::get_retry_delay` (`common/calculators/retry_delay_calculator.rs`), and a `Retry-After` header is read with `RetryDelayCalculator::parse_retry_after`; a gateway keeps its own retry loop, which decides on its own exceptions, but never its own copy of the backoff
  - the wait is `tokio::time::sleep`, so nothing blocks while it waits
  - each retry is logged as a warning (with the attempt and the wait), so an outage shows in the log even when a retry succeeds; once the retries run out, the last failure is the gateway's exception, as any other
  - a transient failure is its own exception (e.g. `JsonPlaceholderGatewayException::JsonPlaceholderUnavailable`), so the retry decides on the exception's kind, and a service that answers its callers can tell "try again later" (503) from a request that is wrong
  - a database gateway leaves connecting to its pool, whose acquire timeout already waits for a connection; it retries a statement only for what the database documents as safe to run again (e.g. Postgres's serialization failure, `40001`, or deadlock, `40P01`, for a transaction that is run again whole)

## Data layer
- the data layer repeats the architecture one level down: the `DataFacade` (the only entry point a manager sees) over one `DataManager`, which orchestrates every data gateway
  ```
  managers/data_layer/
      data_facade.rs
      managers/
          data_manager.rs
          gateways/
              relational_database/   relational_database_gateway.rs, query_builders/, resource_models/, validators/, mappers/
              document_database/     the same shape, with find/insert queries, documents and clients/
  ```
  - one manager means a data operation can grow to read from several databases and combine the results into one domain model without the layers above changing; callers never learn which database served a request
  - add a database by adding a gateway subfolder, a seam for its client, a `get_*` in the `ServiceLocator`, and its use in `DataManager`
- relational data is kept in Postgres through sqlx: SQL is sent through a query builder with bound parameters (`$1`, `$2`); values are never formatted into a statement
  - `SqlQuery.statement` is a `&'static str`, so sqlx refuses a statement built at run time; `AssertSqlSafe` is never used in the app
  - `SqlParameter` has a variant only for a type some query binds; add one (e.g. `Integer(i64)`) with the first query that binds it
  - no `sqlx::query!` macros: building needs no database
  - a gateway holds the pool, which the `create_relational_database_pool` seam builds without connecting (`connect_lazy_with`); it opens its first connection when the gateway first sends a statement
  - the pool keeps no connection open while the app is idle, so a database that sleeps while nothing is connected to it can: `min_connections(0)`, and a connection unused for a minute is closed (`idle_timeout`); a refused connection is retried until `acquire_timeout` (10 seconds), then fails the request
- the schema changes only through a new file in `migrations/`, named `<timestamp>_<what_it_does>.sql` with a `-- migrate:up` and a `-- migrate:down` section; never edit a file that has run
  - the app never applies migrations: dbmate applies them to its database before it starts, in development as in a deployment
- `InMemoryDocumentDatabaseClient` (`document_database/clients/`) is a stand-in so the template needs no second server: its store is shared by every client with the same connection string and outlives `close()`; replace it with a real driver client rather than copying it into production code

## Logger
- `ApplicationLogger` (`managers/logger/application_logger.rs`) is an exception to the architecture
  - it is a global singleton (a `static`) that owns its own handlers; any code anywhere imports it and logs via associated functions (e.g. `ApplicationLogger::info(message, ContextualData::new())`)
  - it configures itself with defaults on first use; `ApplicationLogger::configure(...)` is only called at app boot (e.g. in `main()`), and `ApplicationLogger::close()` when the app ends
  - it is agnostic of any logging library: it routes `LogEntry` values to `LogHandlerProtocol` implementations (`managers/logger/handlers/`); the service's `TracingLoggingBridge` routes what libraries log through `tracing` (hyper, sqlx, reqwest) to it, so every line has one format
  - it is never created by, or registered in, the `ServiceLocator`, and never injected through constructors
  - its public functions are `#[track_caller]`, so each entry's source is the code that logged it (file, line and column)
  - `ConsoleLogHandler` writes each entry as one line of JSON (time, level, logger, message, context, source, and an exception's type, message and stack), so a log store keeps an entry and its stack as one event; an exception's stack is the backtrace it captured, then its `Caused by:` chain; `LogRedactor` first takes out of the line what must never be logged (credentials in a URL, tokens, keys, email addresses), which the context never holds in the first place
- ERROR is for what an engineer must look at: an unexpected exception is logged with `ApplicationLogger::error("<what failed>", context, Some(&exception))`, so its stack is kept; a client error (4xx) is a warning, so client errors can be found on their own, logged without a stack with `ApplicationLogger::log_exception(&exception, LogLevel::Warning)`
- every exception is logged once, where it is caught, and nothing that can fail runs without something to catch it
  - every entry point installs a panic hook that logs a panic nothing caught, as a `PanicException` with its stack
  - the hook is the only place a panic is seen with its stack, and it runs before the panic reaches whatever catches it; so the service's `UnexpectedErrorMiddleware`, which catches a panic in a request, has the hook hand that panic over (`take_over_panic`) rather than log it, and logs it itself with the request
  - a background loop logs the error of each iteration and keeps going

## Service
- anything that runs and listens (an HTTP API, a gRPC server, a queue consumer, a stdio server) is a service, in `services/<name>/` beside the domain facade (e.g. `services/http/`), with its binary's entry point in `src/bin/`
  ```
  services/http/
      main.rs                 entry point: logger, tracing bridge, panic hook, HOST and PORT, serve
      http_service.rs         HttpService: creates the facade, serves until shutdown, closes it; router and contract
      routers/                each route maps the request in, calls one facade operation, maps the result out
      wire_models/            serde + utoipa request and response models
      extractors/             JsonBody and PathParameters: axum's Json and Path, answering with an ErrorResponse
      mappers/                wire models <-> domain models, and exceptions -> HTTP errors (HttpErrorMapper)
      exception_handlers/     ApplicationExceptionResponse: the facade's exception as a response, logged
      middleware/             UnexpectedErrorMiddleware: a panic in a request, logged and answered generically
      logging_bridges/        TracingLoggingBridge
      export_openapi.rs       writes the service's contract to openapi.json
  ```
  - a service sits outside the domain container, calls only the domain facade, and holds no application logic
  - an HTTP service uses axum; every service follows its framework's own idioms for routing, deserializing and checking requests, and serializing responses
  - the domain container speaks only in its own terms: requests are mapped (`mappers/`) to domain models (or obvious primitives, such as ids) before the facade is called, and wire or framework types never reach it
  - every domain model the facade returns is mapped to the service's own response model (`wire_models/`) before it is serialized, so the domain can change without changing the API
  - it maps `ApplicationBaseException` kinds to protocol errors (e.g. `HttpErrorMapper`); a client error (4xx) is answered with the exception's own message, everything else is logged and answered generically so tables, SQL and upstream details never leak
  - a route returns `Result<_, ApplicationExceptionResponse>`, whose `IntoResponse` maps, logs and answers the exception; a route never matches on an exception itself
  - axum's own rejections (a path parameter or body that doesn't fit: 400, 422) are answered in the same `ErrorResponse` shape, through the service's `JsonBody` and `PathParameters` extractors, so callers handle one shape of error
  - it creates the domain facade once when it starts and closes it when it stops (`HttpService::serve`); requests share that instance through the router's state (`State<Arc<ExampleAppFacade>>`), and nothing builds one per request
  - a request model says what shape a request has (its fields and their types) and no more: whether a value is acceptable is decided inside the domain container, so the caller is answered with the domain's own message
  - each route names the responses it can answer with in its `#[utoipa::path]` (e.g. `(status = NOT_FOUND, body = ErrorResponse)`), so they are part of the service's contract
  - an HTTP service's contract is `openapi.json` at the app's root: every route with its wire models, written by `cargo run --bin export-openapi` (`services/http/export_openapi.rs`); never edit it by hand, and run the command after changing a router or a wire model, so a change to the API shows up as a change to that file
  - it has its own entry point (e.g. `services/http/main.rs`), whose `main()` reads where to listen: `HOST` and `PORT` when they are set (a deployment sets them), otherwise `127.0.0.1` and the app's development port (constants in that file)
  - it shuts down gracefully on Ctrl-C and SIGTERM, finishing the requests under way, then closes the domain facade
  - a panic in a response's body, after the response has started, is past the middleware: the server ends that connection, and the panic hook logs it

## Starting an app from the template
1. Copy `rust-template/`, then rename `example-app` / `example_app` (`Cargo.toml`, the `example_app::` paths in `src/main.rs` and `src/bin/`), `ExampleAppFacade`, `ExampleAppManager` and the `EXAMPLE_APP_` prefix of the environment variables.
2. Replace `processors/greeting/`, `processors/customer/`, `engines/customer_onboarding/` and the example settings with the app's own domain logic and configuration.
3. Replace the example databases, `Customer` and `AuditEvent` (and `migrations/`) with the app's own.
4. Replace the JSON Placeholder gateway and `UserProfile` with the HTTP services the app depends on.
5. Replace the example routers and wire models in `services/http/` with the app's own and run `cargo run --bin export-openapi`, or remove the service (and its binaries) if the app isn't served over HTTP.
6. Update the README's configuration and routes tables.
