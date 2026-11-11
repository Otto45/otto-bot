# Domain Container architecture

This app follows the Domain Container architecture. These rules apply to every change, and the reasons given with
them say what a rule protects, so that a case the rules don't name is decided the same way.

## Tools
- Python 3.14 with uv; asyncio for async; FastAPI for HTTP services, served by uvicorn, with the contract FastAPI builds; pydantic for wire and resource models; httpx for outgoing HTTP; asyncpg for Postgres (typed by `asyncpg-stubs`); dbmate for migrations
- every change passes `uv run ruff format --check`, `uv run ruff check`, `uv run pyright` (strict) and `uv run export-openapi --check`
- `uv.lock` is not kept: it names the package, so an app made from the template locks its own dependencies
- a dependency is added to `pyproject.toml` (`dependencies`, or the `dev` group for a tool), never installed by hand; `uv run` installs what it names

## Layout
```
src/example_app/
    main.py                     entry point (example-app): calls the Domain Facade, logs what fails
    example_app_facade.py       Domain Facade: public entry point, creates the ServiceLocator and DataFacade
    services/<name>/            anything that runs and listens, outside the domain container (see Service)
    managers/
        example_app_manager.py  Manager: orchestrates workflows
        service_locator/        ServiceLocator, ServiceLocatorSeamsProtocol, ServiceLocatorSeams
        common/                 stateless helpers shared by more than one component
        configuration_providers/ config models -> validators -> settings
        processors/             domain logic the Manager delegates to, one subfolder per processor type
        engines/                multi-step tasks the Manager hands off whole, one subfolder per engine
        gateways/<system>/      one gateway per external system
        data_layer/             DataFacade -> DataManager -> data gateways (see Data layer)
        models/                 domain models; models/settings/ the settings
        exceptions/             ApplicationBaseException, its families, and ExceptionFamily
        logger/                 ApplicationLogger
migrations/                     the relational database's schema: one plain SQL file per change
openapi.json                    the HTTP service's contract, generated
pyproject.toml                  dependencies, the entry points ([project.scripts]), ruff's and pyright's settings
```

## Code
- every variable has its type annotation (`pool: asyncpg.Pool = ...`), a rebinding too (`redacted: str = pattern.sub(...)`), and so does every instance attribute where `__init__` sets it (`self._pool: asyncpg.Pool = pool`) and every function's parameters and return; pyright (strict) checks the types, and the architecture lint checks that they are written
  - exception: the targets of `for`, `with ... as`, `except ... as`, comprehensions and tuple unpacking, `match` captures, a `nonlocal` name, and an enum's members, which Python does not allow to be annotated (rather than rebind a `nonlocal`, keep the state on an object, e.g. `ResponseStartTracker`)
  - nothing is typed `Any` unless a library hands it over as `Any` (a FastAPI error, asyncio's exception handler context, the OpenAPI document), and the code says so where it does
- every function is a method of a class, except in an entry point's module (`main.py`, `export_openapi.py`)
- a Protocol is a `typing.Protocol`, and a class that implements one inherits from it explicitly (`class ServiceLocatorSeams(ServiceLocatorSeamsProtocol):`)
  - a Protocol is suffixed `Protocol`, lives in its own file, and declares method signatures only (each body is `...`): no state, no implementation
- a class that nothing inherits from is marked `@final`, except a model (a dataclass or a pydantic model) and a Protocol; a class that becomes a base loses its `@final`
- each file holds the one class (or type alias) it is named after, in snake_case (`GreetingProcessor` in `greeting_processor.py`); every folder is a package with an `__init__.py`, empty but for the package docstring
- imports are absolute (`from example_app.managers.models.customer import Customer`), one class per import line where it fits; a module imported only for its annotations, where importing it would be a cycle, is imported under `if TYPE_CHECKING:` (annotations are evaluated lazily, so they need no quotes)
- a group of static methods (validators, mappers, query builders, formatters, common helpers) is a `@final` class with `@staticmethod`s only, never built and never registered
- a module-level constant is `Final` and private (`_ACQUIRE_TIMEOUT_SECONDS: Final[float] = 10`)

## Domain container
- a function is `async` when it does I/O or awaits something that does (a gateway's request, a driver's query, a facade or manager method that calls a gateway); a function that only computes is plain `def`, whichever component it is in
  - prefer async wherever there is I/O: use the async API of a driver, client or file system (asyncpg, httpx), never a blocking call inside async code; work that can only block (a library with no async API, heavy CPU work) runs in `asyncio.to_thread`
  - a client that stands in for an async driver (e.g. `InMemoryDocumentDatabaseClient`) keeps the driver's async API, so nothing changes when the real driver replaces it
  - a method that does nothing yet (e.g. a manager's `close()` that owns nothing) is plain `def`, and becomes `async` with the first I/O it does
- every failure the domain container expects is raised as an `ApplicationBaseException` (see Exceptions) and propagates; nothing catches an exception it can't handle, and nothing returns `None` to mean it failed
- all application logic a manager delegates to lives in a `processors/` folder one level below the manager's folder (e.g. `managers/processors/`)
  - one subfolder per type of processor (e.g. `managers/processors/greeting/`)
  - processor classes are suffixed with `Processor` (e.g. `GreetingProcessor` in `greeting_processor.py`)
  - processors are singletons: the service locator caches the single instance and exposes it via a `get_*` method (e.g. `get_greeting_processor()`), never `create_*`
  - logic exclusive to one processor that is too in-depth to code directly in the processor goes in classes of static methods in subfolders under that processor's folder (e.g. `managers/processors/greeting/formatters/`)
- a task that is one step to a manager but many steps of different kinds underneath (external calls, decisions, reads and saves) goes in an engine, so manager methods stay short coordinations
  - one subfolder per engine under `managers/engines/` (e.g. `managers/engines/customer_onboarding/`), holding `<Name>Engine` in `<name>_engine.py` (e.g. `CustomerOnboardingEngine` in `customer_onboarding_engine.py`)
  - engines are singletons: the service locator builds one via a `get_*` method (e.g. `get_customer_onboarding_engine()`), passing its constructor the gateways and processors it uses
  - an engine's processors live in a `processors/` folder inside the engine's folder (e.g. `managers/engines/customer_onboarding/processors/onboarding/`), follow every processor rule, and are used by that engine only
  - an engine uses gateways, its own processors and the `DataFacade`, which the manager passes into each call (e.g. `onboard_customer(user_id, self._data_facade)`) because the `ServiceLocator` never sees the `DataFacade`; the engine imports `DataFacade` under `if TYPE_CHECKING:`, since the `DataFacade`'s module imports the `ServiceLocator`'s, which imports the engine's
  - an engine never uses a manager's processors, another engine, or a manager
  - an engine keeps no state between calls and owns nothing to release, so it has no `close()` and is never registered for closing
- managers never call other managers; anything they share goes through the `DataFacade`, a processor or an engine
- stateless helpers used by more than one component live in `managers/common/` (e.g. `common/calculators/retry_delay_calculator.py`, `common/validators/validator_string.py`, `common/formatters/validation_error_formatter.py`)
  - static methods only, no I/O; never registered in the `ServiceLocator`
  - never copy a common helper into a component as a private method

## Facade, locator and closing
- the domain facade creates the `ServiceLocator`, then the `DataFacade` (passing it the `ServiceLocator`)
  - the `ServiceLocator` never creates the `DataFacade`
  - the domain facade passes both the `ServiceLocator` and the `DataFacade` to every manager it creates
  - the domain facade owns the `DataFacade` and closes it; managers share it but never close it
  - the domain facade is created inside the running event loop: the database pool binds to the loop it is built in
  - creating the facade validates all configuration, so invalid configuration makes `ExampleAppFacade()` fail before anything runs; the configuration is the first thing anything asks the locator for, so a configuration that fails leaves nothing open
- closing is explicit: whoever creates a domain facade uses it in `async with`, which closes it on every path, whether using it succeeded or failed (see `main.py` and `HttpService.serve`)
  - the domain facade closes its managers, then the `DataFacade`, then the `ServiceLocator` last
  - the `ServiceLocator` closes every singleton it caches in its `close()`, in reverse order of creation, through the close callback it registers when it creates one, so nothing is closed while something created after it still uses it
  - managers and the `DataManager` never close gateways; a manager's `close()` releases only what the manager itself owns
- the `ServiceLocator` caches each singleton in an attribute that starts as `None`, and builds it the first time its `get_*` is called; the `get_*` methods are synchronous, so on the event loop no two callers are ever inside one at once, and each singleton is built once
  - a singleton only asks for the singletons it depends on, which never ask for it
- the objects that reach outside the process (the configuration source, the HTTP client, the database pool, the document client) are built by the locator's `create_*` seams, declared by `ServiceLocatorSeamsProtocol` and implemented for production by `ServiceLocatorSeams`
  - a seam takes the settings it needs as a parameter rather than asking the locator
  - only the seams know the default sources (the environment, the network, the drivers); `ServiceLocator()` uses `ServiceLocatorSeams`
  - whatever needs other seams (a test swapping the network or the environment) builds the same production locator with `ServiceLocator(seams=...)`, and its seams call `ServiceLocatorSeams` for every seam they keep, so the caching and wiring that run are always the production ones
  - add a dependency that reaches outside the process by adding a seam for the deepest object under it, and a `get_*` that builds its gateway from that seam

## Models
- domain models and settings are `@dataclass(frozen=True, slots=True)`: they never change once made, so no component can change one another holds
- config models are `@dataclass(slots=True)` and resource models are pydantic `BaseModel`s; every field of theirs is optional (`str | None = None`), so anything from outside deserializes and the validator can say what is wrong with it
- domain models are never serialized: what goes over the wire is a service's own wire model

## Money
- money is kept exact: an amount is an integer number of its currency's minor units (cents), kept with its currency, and never a floating-point number, which can't hold 0.1 and drifts by a cent once multiplied and rounded
- what an amount is multiplied or divided by (an exchange rate, a percentage, a tax rate) is a decimal, `decimal.Decimal`, read from its text, never through a float: a rate a float has held has already lost digits, and no rounding gets them back
- arithmetic is done in decimals and rounded once, at the end, to minor units, half to even (banker's rounding: half a cent goes to the even cent) unless the person asked for another rounding; comparisons against a threshold (e.g. "orders over 100 dollars") are made in minor units
- `MoneyCalculator` (`common/calculators/money_calculator.py`) does it: `multiply` (an amount times a factor, e.g. a rate), `take_percentage` (e.g. a 10% discount) and `parse_decimal` (a decimal from text); never round money by hand
  - it computes in a 100-digit context: the default context's 28 digits would round a large amount times a long rate halfway through
- from outside the process:
  - a JSON body with a decimal is read with `MoneyCalculator.parse_json` (`json.loads(body, parse_float=Decimal)`), then `model_validate`, its resource model's field a `Decimal`; `model_validate_json` reads every JSON number as a float first, and loses digits (a `json.JSONDecodeError` is a `ValueError`, raised as the gateway's invalid-response exception)
  - Postgres: an amount in minor units is a `bigint` column, an `int`; a decimal is a `numeric` column, which asyncpg reads and binds as a `Decimal`
  - a wire model carries an amount as an `int` in minor units, with its currency, and a decimal as a `Decimal`, which pydantic writes as a JSON string, so no client reads it as a float either

## Exceptions
- every exception the application raises is an `ApplicationBaseException` (`managers/exceptions/application_base_exception.py`): a message and a context; Python keeps the stack where it was raised (its traceback), and `get_name()` gives the name it is logged under (its class's, e.g. `"CustomerNotFoundException"`)
  - each component's exceptions are a family: a base class suffixed `Exception` that subclasses `ApplicationBaseException` (e.g. `RelationalDatabaseGatewayException`), in its own file, with each of its exceptions a `@final` subclass in a file of its own (e.g. `CustomerNotFoundException`); a new exception is a new subclass
  - `ExceptionFamily` (`exceptions/exception_family.py`) is the union of every family, and each family's `get_family()` returns the exception as a member of it; a new family is added to the union (it can't implement `get_family()` until it is)
  - code that treats a family alike catches or matches the family (`except JsonPlaceholderGatewayException`), and a match that decides something for every exception (e.g. `HttpErrorMapper.to_status_code`) matches `exception.get_family()` and names every family, with no `case _:`, so pyright (`reportMatchNotExhaustive`) fails it until a new family is decided there
  - an exception raised from another error is raised `from` it (`raise PostgresQueryException(...) from error`), which keeps it in the stack that is logged
- an exception's context (`ContextualData`) holds primitives (`ContextValue`: `str | int | float | bool`) under snake_case keys and never anything sensitive (no bound parameter values, no credentials); a context with nothing in it is `ContextualData()`, and `with_value` returns a new one with one more value
  - a library's error is described by its class (`error_type`: `type(error).__name__`, e.g. `ConnectError`), and a Postgres error by its SQLSTATE (`postgres_error_code`)
- an exception raised for something the caller can put right (a value that is not valid, something that was not found) has a message written for the person using the app (e.g. `InvalidCustomerException`: "The customer's name is required"), because a service answers with it as it is; what only engineers need goes in the context
- unexpected exceptions are every other exception (a bug, a library's error nothing translated), and every entry point installs `UncaughtExceptionLogger` (see Logger)

## Configuration
- configuration is read from the environment only by `ConfigurationSourceProviderEnvironment`, inside its methods, never at import time, except where a service listens, which its entry point reads (see Service)
- the `ConfigurationProvider` requires its `ConfigurationSourceProviderProtocol`; only the `ServiceLocatorSeams` (`create_configuration_source_provider`) knows the default source
- each setting group flows config model -> validator -> settings: the `ConfigurationProvider` loads and validates every group when it is created, then maps each with `ValidatedValue.expect(value, "<Validator>")`

## Gateways
- anything that comes from outside the process (an API response, a database row, a document) is deserialized into a resource model (a pydantic model with every field optional), checked by a validator, then mapped to a domain model; resource models never leave their gateway, which returns domain models only
  ```
  managers/gateways/json_placeholder/
      json_placeholder_gateway.py     sends the request, translates httpx errors into exceptions
      resource_models/                the JSON body deserialized, every field optional
      validators/                     check the resource has every value the domain model needs
      mappers/                        map the validated resource to a domain model
  ```
  - a gateway's `resource_models`, `validators`, `mappers`, `query_builders` and `request_builders` are its own: nothing outside the gateway's folder imports them
  - a mapper takes the validated resource, unwraps each field with `ValidatedValue.expect(field, "<Validator>")`, and has a docstring saying it expects a validated resource; each `expect` states a fact the validator checked, and raises only if the validator is wrong (an unexpected exception, never an `assert`, which `python -O` removes)
  - a deserialization error (pydantic's `ValidationError`, for a JSON body, a row or a document) is formatted by `ValidationErrorFormatter.format(error, root_name)`: every problem, by where it is in the input, never the value
  - the gateway is where driver and HTTP errors are translated into `ApplicationBaseException`s; nothing above it sees a library's error
- when building a gateway's request is too involved to inline, it goes in a `request_builders/` subfolder of the gateway (the counterpart of a data gateway's `query_builders/`)
- an HTTP gateway is given its `httpx.AsyncClient`, built by a seam with the service's base URL and the configured timeout; the client pools connections, so the gateway's `close()` closes it (`aclose()`)
- a gateway retries a request that failed for a reason that may pass on its own, so a brief outage of the other system doesn't fail the app's operation
  - transient: a connection that couldn't be made or timed out, and a status of 429, 500, 502, 503 or 504 (or the error a service's API documents as temporary, e.g. `server_error`, a rate limit); never what sending again won't change: any other 4xx (a malformed request, a bad key, an exhausted quota even when it comes as a 429), and a response the gateway can't read
  - only a request that is safe to send twice is retried: a read, or a write the other system makes idempotent (e.g. with an idempotency key the gateway sends); a write that could happen twice is not retried, and its failure is told as it is
  - up to a few times (a setting of the gateway's settings group, e.g. `max_retries`, 3 by default), after a wait that doubles from about a second, up to a cap, with a little random jitter so callers that failed together don't retry together; when the service says how long to wait (`Retry-After`), that wait, up to a cap
  - every gateway waits the same: the wait comes from the shared `RetryDelayCalculator.get_retry_delay_seconds` (`common/calculators/retry_delay_calculator.py`), and a `Retry-After` header is read with `RetryDelayCalculator.parse_retry_after`; a gateway keeps its own retry loop, which decides on its own exceptions, but never its own copy of the backoff
  - the wait is `await asyncio.sleep(...)`, so nothing blocks while it waits
  - each retry is logged as a warning (with the attempt and the wait), so an outage shows in the log even when a retry succeeds; once the retries run out, the last failure is the gateway's exception, as any other
  - a transient failure is its own exception (e.g. `JsonPlaceholderUnavailableException`), so the retry decides on the exception's kind, and a service that answers its callers can tell "try again later" (503) from a request that is wrong
  - a database gateway leaves connecting to its pool, whose acquire timeout already waits for a connection; it retries a statement only for what the database documents as safe to run again (e.g. Postgres's serialization failure, `40001`, or deadlock, `40P01`, for a transaction that is run again whole)

## Data layer
- the data layer repeats the architecture one level down: the `DataFacade` (the only entry point a manager sees) over one `DataManager`, which orchestrates every data gateway
  ```
  managers/data_layer/
      data_facade.py
      managers/
          data_manager.py
          gateways/
              relational_database/   relational_database_gateway.py, query_builders/, resource_models/, validators/, mappers/
              document_database/     the same shape, with find/insert queries, documents and clients/
  ```
  - one manager means a data operation can grow to read from several databases and combine the results into one domain model without the layers above changing; callers never learn which database served a request
  - add a database by adding a gateway subfolder, a seam for its client, a `get_*` in the `ServiceLocator`, and its use in `DataManager`
- relational data is kept in Postgres through asyncpg: SQL is sent through a query builder with bound parameters (`$1`, `$2`); values are never formatted into a statement
  - `SqlQuery.statement` is a `LiteralString`, so pyright refuses a statement built from a value at run time (an f-string, a concatenation with a variable); nothing casts one to it
  - `SqlParameter` names only the types some query binds; add one (e.g. `str | int | datetime`) with the first query that binds it
  - a parameter is bound as the Python type of its column: an `int` for an integer, a `datetime` for a `timestamptz`, a `Decimal` for a `numeric`; asyncpg encodes each parameter by its column's type and doesn't parse text into it, so a value passed as text for any other type (a datetime's `isoformat()`, an `int` as a string) fails every time the statement runs, which no check can see without a database
  - a row (`asyncpg.Record`) is read into its resource model with `model_validate(dict(record.items()))`
  - a gateway holds the pool, which the `create_relational_database_pool` seam builds without starting it (`asyncpg.create_pool(...)`, not awaited); the gateway starts it, once, when it first sends a statement, and closes it only if it started it
  - the pool keeps no connection open while the app is idle, so a database that sleeps while nothing is connected to it can: `min_size=0`, and a connection unused for a minute is closed (`max_inactive_connection_lifetime`); a statement waits up to 10 seconds for a connection (`acquire(timeout=...)`, opening one included), then fails the request
- the schema changes only through a new file in `migrations/`, named `<timestamp>_<what_it_does>.sql` with a `-- migrate:up` and a `-- migrate:down` section; never edit a file that has run
  - the app never applies migrations: dbmate applies them to its database before it starts (`dbmate --migrations-dir migrations up`), in development as in a deployment
- `InMemoryDocumentDatabaseClient` (`document_database/clients/`) is a stand-in so the template needs no second server: its store is shared by every client with the same connection string and outlives `close()`; replace it with a real driver client rather than copying it into production code

## Logger
- `ApplicationLogger` (`managers/logger/application_logger.py`) is an exception to the architecture
  - it is a global singleton (a class with class-level state) that owns its own handlers; any code anywhere imports it and logs via class methods (e.g. `ApplicationLogger.info(message, context)`)
  - it configures itself with defaults on first use; `ApplicationLogger.configure(...)` is only called at app boot (e.g. in `main()`), and `ApplicationLogger.close()` when the app ends
  - it is agnostic of any logging library: it routes `LogEntry` values to `LogHandlerProtocol` implementations (`managers/logger/handlers/`); the service's `LoggingBridge` routes what libraries log through Python's `logging` (uvicorn, httpx, asyncpg) to it, so every line has one format
  - it is never created by, or registered in, the `ServiceLocator`, and never injected through constructors
  - each entry's source is the code that called it (file, line and function), found from the call stack
  - `ConsoleLogHandler` writes each entry as one line of JSON (time, level, logger, message, context, source, and an exception's type, message and stack), so a log store keeps an entry and its stack as one event; an exception's stack is its traceback with the exceptions it was raised from, as Python formats it; `LogRedactor` first takes out of the line what must never be logged (credentials in a URL, tokens, keys, email addresses), which the context never holds in the first place
- ERROR is for what an engineer must look at: an unexpected exception is logged with `ApplicationLogger.error("<what failed>", context, exception)`, so its stack is kept; a client error (4xx) is a warning, so client errors can be found on their own, logged without a stack with `ApplicationLogger.log_exception(exception, LogLevel.WARNING)`
- every exception is logged once, where it is caught, and nothing that can fail runs without something to catch it
  - every entry point installs `UncaughtExceptionLogger`: `install()` for an exception that ends the app or a thread, and `install_for_event_loop()`, inside the running loop, for one raised in an asyncio task nothing awaited
  - the service's `UnexpectedErrorMiddleware` catches an unexpected exception in a request, logs it with the request, and answers it; it goes no further, so the server doesn't log it again
  - a background loop logs the exception of each iteration and keeps going

## Service
- anything that runs and listens (an HTTP API, a gRPC server, a queue consumer, a stdio server) is a service, in `services/<name>/` beside the domain facade (e.g. `services/http/`), with its entry point in `[project.scripts]`
  ```
  services/http/
      main.py                 entry point (example-app-http): logger, logging bridge, uncaught exceptions, HOST and PORT, serve
      http_service.py         HttpService: creates the facade, serves until shutdown, closes it; the application and its contract
      routers/                each route maps the request in, calls one facade operation, maps the result out
      wire_models/            pydantic request and response models
      dependencies/           ExampleAppFacadeDependency: hands a route the one facade (FastAPI's Depends)
      mappers/                wire models <-> domain models, and exceptions -> HTTP errors (HttpErrorMapper)
      exception_handlers/     ApplicationExceptionHandler, RequestValidationExceptionHandler, HttpExceptionHandler: errors answered as an ErrorResponse, logged
      middleware/             UnexpectedErrorMiddleware: an unexpected exception in a request, logged and answered generically
      logging_bridges/        LoggingBridge
      export_openapi.py       writes the service's contract to openapi.json
  ```
  - a service sits outside the domain container, calls only the domain facade, and holds no application logic
  - an HTTP service uses FastAPI; every service follows its framework's own idioms for routing, deserializing and checking requests, and serializing responses
  - a router is a `@final` class whose `create_router()` builds an `APIRouter` and registers its routes, each a static method (`router.add_api_route(path, CustomerRouter.save_customer, ...)`)
  - the domain container speaks only in its own terms: requests are mapped (`mappers/`) to domain models (or obvious primitives, such as ids) before the facade is called, and wire or framework types never reach it
  - every domain model the facade returns is mapped to the service's own response model (`wire_models/`) before it is serialized, so the domain can change without changing the API
  - it maps `ApplicationBaseException` families to protocol errors (e.g. `HttpErrorMapper`); a client error (4xx) is answered with the exception's own message, everything else is logged and answered generically so tables, SQL and upstream details never leak
  - a route lets the facade's exception propagate to `ApplicationExceptionHandler`, which maps, logs and answers it; a route never catches an exception itself
  - FastAPI's own answer to a request that doesn't fit (a body or a path parameter that doesn't fit its model: 422) is given in the same `ErrorResponse` shape, by `RequestValidationExceptionHandler`, so callers handle one shape of error; so are its HTTP errors (a path that is no route's: 404; a method its route doesn't accept: 405, with `Allow`), by `HttpExceptionHandler`, never FastAPI's `{"detail": ...}`
  - it creates the domain facade once when it starts and closes it when it stops (`HttpService.serve`); requests share that instance through `ExampleAppFacadeDependency`, and nothing builds one per request
  - a request model says what shape a request has (its fields and their types) and no more: whether a value is acceptable is decided inside the domain container, so the caller is answered with the domain's own message
  - each route names the responses it can answer with in its `responses` (e.g. `HTTPStatus.NOT_FOUND: {"description": ..., "model": ErrorResponse}`), so they are part of the service's contract
  - an HTTP service's contract is `openapi.json` at the app's root: every route with its wire models, written by `uv run export-openapi` (`services/http/export_openapi.py`); never edit it by hand, and run the command after changing a router or a wire model, so a change to the API shows up as a change to that file
  - it has its own entry point (e.g. `services/http/main.py`), whose `main()` reads where to listen: `HOST` and `PORT` when they are set (a deployment sets them), otherwise `127.0.0.1` and the app's development port (constants in that file)
  - it shuts down gracefully on Ctrl-C and SIGTERM: uvicorn stops taking requests and finishes the ones under way, then hands the signal back to the handler `main()` installed, which does nothing, so the facade is closed before the process ends
  - an exception raised after a response has started is past the middleware: the server ends that connection, and logs it through the `LoggingBridge`

## Starting an app from the template
1. Copy the template, then rename `example-app` / `example_app` (`pyproject.toml`, the `src/example_app/` folder and every `example_app.` import), `ExampleAppFacade`, `ExampleAppManager` and the `EXAMPLE_APP_` prefix of the environment variables.
2. Replace `processors/greeting/`, `processors/customer/`, `engines/customer_onboarding/` and the example settings with the app's own domain logic and configuration.
3. Replace the example databases, `Customer` and `AuditEvent` (and `migrations/`) with the app's own.
4. Replace the JSON Placeholder gateway and `UserProfile` with the HTTP services the app depends on.
5. Replace the example routers and wire models in `services/http/` with the app's own and run `uv run export-openapi`, or remove the service (and its entry points) if the app isn't served over HTTP.
6. Update the README's configuration and routes tables.
