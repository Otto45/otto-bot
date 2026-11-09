# otto-bot: Domain Container architecture

otto-bot, the coding agent harness, follows the Domain Container architecture itself. These rules apply to every
change to otto-bot, and the reasons given with them say what a rule protects, so that a case the rules don't name is
decided the same way. What otto-bot is and how it works is in `docs/DESIGN.md`.

The apps otto-bot builds follow their stack pack's rules (`stack_packs/<stack>/`, e.g. `stack_packs/rust/template/AGENTS.md`),
not this file. The Rust pack's template is where these rules come from; a rule changed here for otto-bot says so, and
an example a rule gives from the template (e.g. `GreetingProcessor`, `JsonPlaceholderGateway`) is found in
`stack_packs/rust/template/`.

## Tools
- Rust (edition 2024) with Cargo; Tokio for async; clap for the CLI's arguments; serde for wire and resource models (`serde_path_to_error` names the field that failed); reqwest for outgoing HTTP
- every change passes `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
- `unsafe` is forbidden (`unsafe_code = "forbid"`); `module_inception` is allowed, because a folder and its main file share a name (`service_locator/service_locator.rs`)
- `Cargo.lock` is kept: otto-bot is an app, not a template, and its builds lock their dependencies

## Layout
```
src/
    bin/otto.rs                 the CLI's entry point: only calls services::cli::main::main()
    lib.rs                      the crate: otto_bot_facade, managers (the domain container), services
    otto_bot_facade.rs          Domain Facade: public entry point for every client, creates the ServiceLocator
    services/<name>/            anything that runs and listens, outside the domain container (see Service)
    managers/
        otto_bot_manager.rs     Manager: serves sessions and coordinates their runs
        service_locator/        ServiceLocator, ServiceLocatorSeamsProtocol, ServiceLocatorSeams
        runtime/                what a manager owns while the app runs: live sessions, runs, event channels
        common/                 stateless helpers shared by more than one component
        configuration_providers/ config models -> validators -> settings
        processors/             domain logic the Manager delegates to, one subfolder per processor type
        engines/                multi-step tasks the Manager hands off whole, one subfolder per engine
        gateways/<system>/      one gateway per external system (e.g. openai_responses/)
        data_layer/             DataFacade -> DataManager -> data gateways (see Data layer)
        models/                 domain models; models/settings/ the settings
        exceptions/             ApplicationBaseException and its kinds
        logger/                 ApplicationLogger
stack_packs/<stack>/            what otto-bot knows about building apps in one language (see docs/DESIGN.md)
docs/DESIGN.md                  otto-bot's design
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
- what a manager owns while the app runs (otto-bot's live sessions, their runs and event channels) lives in `managers/runtime/`, one type per file; it holds tasks and channels, so it is not a domain model, only its manager uses it, and its manager's `close()` releases it
  - a run is a Tokio task; the error of each run is logged where the run catches it, and its end is reported as an event; a run that is cancelled or panics has its end reported by a watcher task (`OttoBotManager::watch`)
- stateless helpers used by more than one component live in `managers/common/` (e.g. `common/validators/validator_string.rs`, `common/formatters/validation_error_formatter.rs`)
  - associated functions only, no I/O; never registered in the `ServiceLocator`
  - never copy a common helper into a component as a private function

## Facade, locator and closing
- the domain facade creates the `ServiceLocator`, then the `DataFacade` (passing it the `ServiceLocator`)
  - the `ServiceLocator` never creates the `DataFacade`
  - the domain facade passes both the `ServiceLocator` and the `DataFacade` (`Arc<DataFacade>`) to every manager it creates
  - the domain facade owns the `DataFacade` and closes it; managers share it but never close it
  - the domain facade is created inside a Tokio runtime: the database pool starts its maintenance task when it is built
  - creating the facade validates all configuration, so invalid configuration makes `OttoBotFacade::new()` fail before anything runs
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

## The agent's workspace and tools
- the `WorkspaceGateway` is the only code that reads or changes the agent's workspace, so it is where the workspace's boundary is kept: every path is relative to the workspace and resolves inside it (no `..`, no symbolic link out of it), and `.env` files and `.git/` are never read or changed
- the `CommandRunnerGateway` runs only a stack pack's declared commands, with no input, a timeout, and without `OPENAI_API_KEY` in their environment
- what the agent reads, and what checks print, goes to the model only through the `ToolProcessor`, which takes recognizable secrets out (`SecretRedactor`)
- an exception a tool call runs into (a missing file, a patch that doesn't apply, a check that can't start) is told to the model as that call's output, so it can do something else; `AgentRunEngine::is_told_to_model` decides which, naming every family; any other exception ends the run
- every agent runs on its own lane (`AgentAssignment`: a lane name, a role, a workspace) with its own conversation; there are three roles: the orchestrator (the session's own agent), the implementor and the reviewer
  - every role gets the same tools in the same order: a role is restricted by which tools it may call (`ToolProcessor::get_allowed_tool_names`, sent as `allowed_tools`), never by removing tools, so the cached prompt prefix is shared; the engine refuses a call outside them too (`ToolNotAllowed`, told to the model)
  - a role's reasoning effort is a `configuration_update` item at the start of its lane, never the request's `reasoning.effort`, which would miss the cache
- the orchestrator holds the conversation with the person, who is not technical (otto-bot is part of a software factory that builds apps for non-technical people): it speaks in plain words about what the app does, never about files, code or checks, and asks only what the app should do, deciding the technical rest itself; its lane keeps the whole session, it reads the code (`read_files`, `list_files`, `search`) but changes no file and runs no check, and it asks the person what only they can decide by ending its turn with the question (no run waits on a person); when a request is clear it briefs the implementor with `brief_implementor` (the task, the decisions made, what done looks like, and the files to start from), which the `BriefProcessor` checks without a model, reporting every problem in one `InvalidBrief`, told to the model
  - an accepted brief ends the orchestrator's run (`AgentRunOutcome`); the manager keeps it (`briefs.jsonl`, whose count numbers it), reports `BriefAccepted`, and has the implementor build it
  - once the brief is built and reviewed, the result is told to the orchestrator as a harness note and it gets a turn to tell the person, in plain words, what their app can do now (`AgentRunEngine::reply_after_build`), with only the reading tools: a new brief waits for the person's answer
- the implementor (`AgentRunEngine::implement_brief`) is the only role that changes files, in the session's own workspace: each brief gets a fresh lane (`implementor_<n>`), so it starts from the brief alone; it plans the change, builds it, and ends with `report` (done, or blocked with what it needs); once it reports done, the harness runs every check itself and sends it what fails, for up to `IMPLEMENTOR_ROUNDS`, and reports `BriefImplemented`
  - its requests open with the orchestrator's kept project map (already cached), a note of what changed since it was built, and the brief, with the files it names read into it (up to `MAX_BRIEF_FILES`), so it starts without a turn of reading
- the reviewer (`AgentRunEngine::review_change`) reviews each brief's change, on a fresh lane (`reviewer_<n>`), once the implementor reports it done with every check passing: it is given the person's last message, the brief, and the change's diff (the `DiffProcessor`'s, from snapshots of the workspace's text files before and after), may only read, judges what the code does rather than what its comments or the brief say about it (a documented limitation that makes the change wrong is still a finding; money is kept exact), judges it against the app as it is (a way of doing things the app's code already uses is not a finding unless the change makes it worse or the person asked for more), and ends with `submit_review` (findings graded high, medium or low); high-severity findings go back to the same implementor for one round of fixes, held to the checks again, and every finding is reported with `BriefImplemented`
- every request opens with the `PromptProcessor`'s layers, each a content block ending in an explicit cache breakpoint, ordered from what the most requests share to what the fewest do: the stack's layer (instructions and the pack's rules; it names no workspace, so every project on the stack shares it; 12 hours), then the workspace's (its path and project map; 30 minutes), then its assignment's (what its role is; 30 minutes); implicit mode caches the conversation after them
  - nothing that varies goes in a layer (no time, no id, nothing unsorted), and a layer never changes during a run; something new for the model is appended to the conversation instead
- the project map is every file of the workspace and, under each source file, its public items with their bodies left out, picked by the stack pack's tree-sitter query (`[project_map]` in pack.toml: a grammar built into otto-bot, source globs, and a query whose `@item` captures are shown and whose `@body` and `@omit` captures inside them are left out)
  - a session keeps the map its requests open with from run to run (its `project_maps.jsonl`), so the prefix and the orchestrator's lane carry over; each run tells what changed since the last as a harness note (a developer message) in the conversation, and the map is rebuilt only when the notes would be more than a quarter of it
- no agent formats code or regenerates a generated file by hand: the harness runs the stack pack's formatters (`[format]` in pack.toml: the code's formatter, and e.g. the OpenAPI export) before every run of the checks, and tells the agent which files they changed (read again before patching them), so a format or contract check fails only on code that doesn't parse or compile
- the architecture lint runs as the `architecture` check, before the pack's commands, so the agent learns of a rule it broke as it learns of a compiler error; its rules are the stack pack's (`[lint]` in pack.toml), checked without a model: `imports` rules (a file whose path matches a pattern imports nothing matching another; a `name` group in both means "another of the same kind"), `query` rules (tree-sitter `@violation` captures, minus `@allowed` ones) and `named_after_file`
  - the Rust pack's rules pass on its template and on otto-bot itself, and the Python and TypeScript packs' on their templates; a rule that flags code the architecture allows is fixed in the pack, not worked around in code
  - the stack pack's grammar, queries, globs and patterns are checked when used: one that can't be used is an `InvalidStackPack` exception, which ends the run
- starting an app (`otto new`) is the `ProjectBootstrapEngine`'s, with no model: it copies the stack pack's template (what the template's `.gitignore` ignores left out, read by the `StackPackStoreGateway`) into a new or empty folder named after the app, replaces the pack's `[rename]` names in file contents and paths (`AppNameProcessor`: one pass, the longest name first), and runs the pack's formatters; the examples stay, for the first brief to replace
- the OpenAI gateway's transport is a setting (`OTTO_BOT_OPENAI_TRANSPORT`: `websocket`, the default, or `http`); the domain always hands it a lane's whole conversation, and the transport decides what to send
  - over WebSocket each lane (one agent of a session, `<session_id>/<lane_name>`) has its own connection, which holds the lane's latest response; a request that only adds to the conversation that response covers continues it (`previous_response_id`) and sends the new items only
  - a continuation the connection no longer holds, or a lost connection, is sent once more with the whole conversation on a fresh connection; a response that sends no event for the idle timeout (`OTTO_BOT_OPENAI_IDLE_TIMEOUT_SECONDS`; a live one streams events every few seconds, even while reasoning) has stalled, and counts as a lost connection; a connection near the API's 60-minute limit is replaced before a request; a lane whose last request was abandoned (its run cancelled) gets a new connection; if no connection can be opened, the request goes over HTTP
  - a request the API failed on its side, whatever the transport, is sent again, up to `OTTO_BOT_OPENAI_MAX_RETRIES` times (3 by default), after a wait that doubles from a second (up to 20 seconds, with jitter) or the `Retry-After` the API asks for (up to a minute): an error status of 429, 500, 502, 503 or 504, an error event or a failed response of type `server_error` or a rate limit, a request that couldn't connect or timed out, and a connection lost again after its resend are `OpenAiUnavailable` (`ApiErrorMapper` decides, from the status, type and code); a refusal sending again won't change (a malformed request, a bad key, `insufficient_quota`) is `OpenAiRequest`, and a response that ran past the whole response timeout isn't sent again either; each retry is logged as a warning

## Data layer
- changed for otto-bot: its data is files, not databases, so it has no migrations and no relational gateway; the Postgres rules below apply only if one is ever added
  ```
  managers/data_layer/managers/gateways/
      session_store/       a folder per session: session.json, and append-only conversation_items/<lane_name>.jsonl (one per lane), usage_records.jsonl, project_maps.jsonl and briefs.jsonl
      stack_pack_store/    the stack packs, read-only: each folder's pack.toml
  ```
  - a session's files are only ever appended to, never rewritten, so a crash mid-write loses at most the line being written; whatever needs a changed record appends a new one
  - each line is a document (a resource model, every field optional): read back, it is deserialized, validated and mapped like anything else from outside the process
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
- anything that runs and listens (the CLI, a stdio server for editors, an HTTP API) is a service, in `services/<name>/` beside the domain facade (e.g. `services/cli/`), with its binary's entry point in `src/bin/`; every client of otto-bot is a service over the same facade
  ```
  services/cli/
      main.rs                 entry point: logger, tracing bridge, panic hook, arguments, run
      cli_service.rs          CliService: creates the facade, converses until the person quits, closes it
      wire_models/            clap argument models (CliArguments, ServiceTierArgument)
      mappers/                wire models <-> domain models, events -> terminal output, exceptions -> client error or not (CliErrorMapper)
      exception_handlers/     ApplicationExceptionReport: the facade's exception logged, and what to tell the person
      logging_bridges/        TracingLoggingBridge
  ```
  - a service sits outside the domain container, calls only the domain facade, and holds no application logic
  - every service follows its framework's own idioms (clap for the CLI's arguments; axum for an HTTP service)
  - the domain container speaks only in its own terms: what a client sends is mapped (`mappers/`) to domain models (or obvious primitives, such as ids) before the facade is called, and wire or framework types never reach it
  - every domain model the facade returns, and every `SessionEvent`, is mapped to the service's own output before it is shown or serialized, so the domain can change without changing a client
  - it decides, for every exception kind, whether the client can put it right (e.g. `CliErrorMapper::is_client_error`, naming every family with no `_` arm); a client error is logged as a warning and told as its own message, anything else is logged as an error with its stack
    - changed for otto-bot: the CLI's person is its operator, so it also shows a server error's own message (which never holds a secret: see Exceptions) and says the details are in the log; a service that answers anyone else (e.g. an HTTP API) answers generically, so tables, queries and upstream details never leak
  - it creates the domain facade once when it starts and closes it when it stops (`CliService::run`), on every path; nothing builds one per request
  - a client subscribes to a session's events before it submits a request, and follows them until the run ends (`SessionEvent::ends_run`); runs never wait on a person, so no service asks one anything mid-run
  - the CLI logs to a file (`$OTTO_BOT_HOME/logs/otto.log`, by default `~/.otto-bot/logs/otto.log`), because its terminal is the conversation; its entry point opens the log before anything else runs, the way a service's entry point reads where it listens
  - the CLI reads nothing from the environment itself: its arguments override what configuration holds (e.g. `--service-tier` over `OTTO_BOT_SERVICE_TIER`), through the options it passes the facade
  - an HTTP service, when there is one, follows the Rust pack's HTTP rules (`stack_packs/rust/template/AGENTS.md`, Service): axum, utoipa and `openapi.json`, its `JsonBody` and `PathParameters` extractors, `HOST` and `PORT`, graceful shutdown on Ctrl-C and SIGTERM
