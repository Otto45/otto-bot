# Domain Container architecture

This app follows the Domain Container architecture. These rules apply to every change, and the reasons given with
them say what a rule protects, so that a case the rules don't name is decided the same way.

## Tools
- TypeScript on Node 24, which runs the `.ts` files as they are, their types stripped (no build step); pnpm; Hono for HTTP services, served by `@hono/node-server`, with its contract from `@hono/zod-openapi`; zod for wire and resource models; `fetch` for outgoing HTTP; `pg` for Postgres; dbmate for migrations
- every change passes `pnpm exec prettier --check .`, `pnpm exec eslint .` (typescript-eslint's strict, type-checked rules), `pnpm exec tsc --noEmit` (strict) and `pnpm run export-openapi --check`
- TypeScript is 6.0: typescript-eslint doesn't support TypeScript 7 yet
- Node strips the types without compiling, so only syntax that erases is used (`erasableSyntaxOnly`): no `enum` (a `const` object and its values' type, e.g. `LogLevel`), no parameter properties, no `namespace`
- `pnpm-lock.yaml` is not kept by the template, so an app made from it locks its own dependencies
- a dependency is added to `package.json` (`dependencies`, or `devDependencies` for a tool), never installed by hand; `pnpm install` installs what it names

## Layout
```
src/
    main.ts                     entry point (pnpm run example-app): calls the Domain Facade, logs what fails
    example_app_facade.ts       Domain Facade: public entry point, creates the ServiceLocator and DataFacade
    services/<name>/            anything that runs and listens, outside the domain container (see Service)
    managers/
        example_app_manager.ts  Manager: orchestrates workflows
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
package.json                    dependencies, the entry points (scripts), and the #src/ import alias
```

## Code
- every variable has its type written out (`const pool: pg.Pool = ...`, `let timeoutError: string | undefined = undefined`), and so does every class field, every function's parameters and its return; the compiler checks the types, and the architecture lint and eslint check that they are written
  - exception: a callback's parameters and return, when the function it is passed to types them (a Hono route's handler, `defaultHook`), and the bindings of `for ... of`, `catch` and destructuring
  - exception: a field initialized with an `as const` literal (`public readonly family = "JsonPlaceholderGateway" as const`)
  - exception: a zod schema (`UserResourceSchema`), whose type only inference can name; the type of what it reads is `z.infer`'s, written beside it (`export type UserResource = z.infer<typeof UserResourceSchema>`)
  - nothing is typed `any`; a value of unknown shape is `unknown`, and is narrowed (`instanceof`, a zod schema) before it is used
- every function is a method of a class, except in an entry point's module (`main.ts`, `export_openapi.ts`)
- every class member says who may use it (`public`, `private`, `protected`), and a field that never changes once set is `readonly`
- a Protocol is an `interface`, and a class that implements one says so (`class ServiceLocatorSeams implements ServiceLocatorSeamsProtocol`)
  - a Protocol is suffixed `Protocol`, lives in its own file, and declares method signatures only: no fields, no implementation
- each file holds the one class, interface or type it is named after, in snake_case (`GreetingProcessor` in `greeting_processor.ts`); a zod schema and the type it gives share a file (`UserResourceSchema` and `UserResource` in `user_resource.ts`)
- imports go through the `#src/` alias, with the file's `.ts` extension (`import type { Customer } from "#src/managers/models/customer.ts"`), never a relative path; what is imported only as a type is imported with `import type`, which Node erases, so a type-only import never makes a cycle
- a group of static methods (validators, mappers, query builders, formatters, common helpers) is a class of `public static` methods only, never built and never registered
- a module-level constant is UPPER_SNAKE_CASE and not exported (`const ACQUIRE_TIMEOUT_MILLISECONDS: number = 10_000`)

## Domain container
- a function is `async` (returns a `Promise`) when it does I/O or awaits something that does (a gateway's request, a driver's query, a facade or manager method that calls a gateway); a function that only computes is not, whichever component it is in
  - prefer async wherever there is I/O: use the async API of a driver, client or file system (`pg`, `fetch`, `node:fs/promises`), never a synchronous one (`readFileSync`) inside the domain container; heavy CPU work runs in a `worker_threads` worker
  - a client that stands in for an async driver (e.g. `InMemoryDocumentDatabaseClient`) keeps the driver's async API, so nothing changes when the real driver replaces it
  - a method that does nothing yet (e.g. a manager's `close()` that owns nothing) is not async, and becomes async with the first I/O it does
  - every promise is awaited or returned (eslint's `no-floating-promises`), so no failure goes unseen
- every failure the domain container expects is thrown as an `ApplicationBaseException` (see Exceptions) and propagates; nothing catches an exception it can't handle, and nothing returns `undefined` to mean it failed
- all application logic a manager delegates to lives in a `processors/` folder one level below the manager's folder (e.g. `managers/processors/`)
  - one subfolder per type of processor (e.g. `managers/processors/greeting/`)
  - processor classes are suffixed with `Processor` (e.g. `GreetingProcessor` in `greeting_processor.ts`)
  - processors are singletons: the service locator caches the single instance and exposes it via a `get*` method (e.g. `getGreetingProcessor()`), never `create*`
  - logic exclusive to one processor that is too in-depth to code directly in the processor goes in classes of static methods in subfolders under that processor's folder (e.g. `managers/processors/greeting/formatters/`)
- a task that is one step to a manager but many steps of different kinds underneath (external calls, decisions, reads and saves) goes in an engine, so manager methods stay short coordinations
  - one subfolder per engine under `managers/engines/` (e.g. `managers/engines/customer_onboarding/`), holding `<Name>Engine` in `<name>_engine.ts` (e.g. `CustomerOnboardingEngine` in `customer_onboarding_engine.ts`)
  - engines are singletons: the service locator builds one via a `get*` method (e.g. `getCustomerOnboardingEngine()`), passing its constructor the gateways and processors it uses
  - an engine's processors live in a `processors/` folder inside the engine's folder (e.g. `managers/engines/customer_onboarding/processors/onboarding/`), follow every processor rule, and are used by that engine only
  - an engine uses gateways, its own processors and the `DataFacade`, which the manager passes into each call (e.g. `onboardCustomer(userId, this.dataFacade)`) because the `ServiceLocator` never sees the `DataFacade`
  - an engine never uses a manager's processors, another engine, or a manager
  - an engine keeps no state between calls and owns nothing to release, so it has no `close()` and is never registered for closing
- managers never call other managers; anything they share goes through the `DataFacade`, a processor or an engine
- stateless helpers used by more than one component live in `managers/common/` (e.g. `common/calculators/retry_delay_calculator.ts`, `common/validators/validator_string.ts`, `common/formatters/validation_error_formatter.ts`)
  - static methods only, no I/O; never registered in the `ServiceLocator`
  - never copy a common helper into a component as a private method

## Facade, locator and closing
- the domain facade creates the `ServiceLocator`, then the `DataFacade` (passing it the `ServiceLocator`)
  - the `ServiceLocator` never creates the `DataFacade`
  - the domain facade passes both the `ServiceLocator` and the `DataFacade` to every manager it creates
  - the domain facade owns the `DataFacade` and closes it; managers share it but never close it
  - creating the facade validates all configuration, so invalid configuration makes `new ExampleAppFacade()` throw before anything runs; the configuration is the first thing anything asks the locator for, so a configuration that fails leaves nothing open
- closing is explicit: whoever creates a domain facade holds it with `await using` (it is `AsyncDisposable`), which closes it on every path, whether using it succeeded or failed (see `main.ts` and `HttpService.serve`)
  - the domain facade closes its managers, then the `DataFacade`, then the `ServiceLocator` last
  - the `ServiceLocator` closes every singleton it caches in its `close()`, in reverse order of creation, through the close callback it registers when it creates one, so nothing is closed while something created after it still uses it
  - managers and the `DataManager` never close gateways; a manager's `close()` releases only what the manager itself owns
- the `ServiceLocator` caches each singleton in a field that starts `undefined`, and builds it (`??=`) the first time its `get*` is called; the `get*` methods are synchronous, so with one thread running JavaScript no two callers are ever inside one at once, and each singleton is built once
  - a singleton only asks for the singletons it depends on, which never ask for it
- the objects that reach outside the process (the configuration source, the `fetch` the HTTP gateway sends with, the database pool, the document client) are built by the locator's `create*` seams, declared by `ServiceLocatorSeamsProtocol` and implemented for production by `ServiceLocatorSeams`
  - a seam takes the settings it needs as a parameter rather than asking the locator
  - only the seams know the default sources (the environment, the network, the drivers); `new ServiceLocator()` uses `ServiceLocatorSeams`
  - whatever needs other seams (a test swapping the network or the environment) builds the same production locator with `new ServiceLocator(seams)`, and its seams call `ServiceLocatorSeams` for every seam they keep, so the caching and wiring that run are always the production ones
  - add a dependency that reaches outside the process by adding a seam for the deepest object under it, and a `get*` that builds its gateway from that seam

## Models
- domain models and settings are interfaces whose every field is `readonly`, made as object literals: they never change once made, so no component can change one another holds
- config models are interfaces whose every field is optional (`greetingPrefix?: string | undefined`), so the validator can say what is wrong with what a source read
- resource models are zod schemas whose every field is optional (`z.string().nullish()`, missing or null), and the types they give (`z.infer`), so anything from outside is read and the validator can say what is wrong with it
- domain models are never serialized: what goes over the wire is a service's own wire model

## Money
- money is kept exact: an amount is an integer number of its currency's minor units (cents), kept with its currency, and never a floating-point number, which can't hold 0.1 and drifts by a cent once multiplied and rounded
- what an amount is multiplied or divided by (an exchange rate, a percentage, a tax rate) is a decimal, a decimal.js `Decimal`, read from its text, never through a float: a rate a float has held has already lost digits, and no rounding gets them back
- arithmetic is done in decimals and rounded once, at the end, to minor units, half to even (banker's rounding: half a cent goes to the even cent) unless the person asked for another rounding; comparisons against a threshold (e.g. "orders over 100 dollars") are made in minor units
- `MoneyCalculator` (`common/calculators/money_calculator.ts`) does it: `multiply` (an amount times a factor, e.g. a rate), `takePercentage` (e.g. a 10% discount) and `parseDecimal` (a decimal from text); never round money by hand
  - an amount in minor units is a `number` that is a safe integer (`Number.isSafeInteger`); a result beyond one is a bug, a `RangeError`; it computes at 100 significant digits, since decimal.js rounds every operation to 20 by default
- from outside the process:
  - a JSON body with a decimal is read with `MoneyCalculator.parseJson` (`JSON.parse` with a reviver that reads each number that isn't a safe integer from its source text, as a `Decimal`), and its resource model's field is `z.instanceof(Decimal)`; `JSON.parse` alone (or `response.json()`) reads it as a `number`, and loses digits
  - Postgres: an amount in minor units is a `bigint` column, which pg reads as a string: its resource model's field is `z.string()`, and its validator checks it is a safe integer before the mapper makes it a `number`; a decimal is a `numeric` column, which pg also reads as a string, made a `Decimal` with `MoneyCalculator.parseDecimal`, and bound as `decimal.toString()`
  - a wire model carries an amount as an integer in minor units, with its currency, and a decimal as a string (`decimal.toString()`; a request's is parsed with `MoneyCalculator.parseDecimal`), so no client reads it as a float either

## Exceptions
- every exception the application throws is an `ApplicationBaseException` (`managers/exceptions/application_base_exception.ts`), an abstract `Error`: a message and a context; JavaScript keeps the stack where it was thrown, and its `name` is its class's, which it is logged under (e.g. `"CustomerNotFoundException"`)
  - each component's exceptions are a family: a class suffixed `Exception` that extends `ApplicationBaseException` (e.g. `RelationalDatabaseGatewayException`), in its own file, with each of its exceptions a subclass in a file of its own (e.g. `CustomerNotFoundException`); a new exception is a new subclass
  - each family names itself in its `family` field (`public readonly family = "RelationalDatabaseGateway" as const`), which makes the families a discriminated union: `ExceptionFamily` (`exceptions/exception_family.ts`), whose members each family's `getFamily()` returns; a new family is added to the union (it can't implement `getFamily()` until it is)
  - code that treats a family alike catches it with `instanceof` (`error instanceof JsonPlaceholderGatewayException`), and a switch that decides something for every exception (e.g. `HttpErrorMapper.toStatusCode`) switches on `exception.getFamily().family` and names every family, with no `default`, so a new family doesn't compile (or lint: `switch-exhaustiveness-check`) until it is decided there
  - an exception thrown from another error keeps it as its `cause` (`new PostgresQueryException(message, context, { cause: error })`), which is logged with it
- an exception's context (`ContextualData`) holds primitives (`ContextValue`: `string | number | boolean`) under snake_case keys and never anything sensitive (no bound parameter values, no credentials); a context with nothing in it is `new ContextualData()`, and `withValue` returns a new one with one more value
  - a library's error is described by its class (`error_type`: e.g. `TypeError`, `AggregateError`), and a Postgres error by its SQLSTATE (`postgres_error_code`)
- an exception thrown for something the caller can put right (a value that is not valid, something that was not found) has a message written for the person using the app (e.g. `InvalidCustomerException`: "The customer's name is required"), because a service answers with it as it is; what only engineers need goes in the context
- unexpected exceptions are every other error (a bug, a library's error nothing translated), and every entry point installs `UncaughtExceptionLogger` (see Logger)

## Configuration
- configuration is read from the environment only by `ConfigurationSourceProviderEnvironment`, inside its methods, never when a module loads, except where a service listens, which its entry point reads (see Service)
- the `ConfigurationProvider` requires its `ConfigurationSourceProviderProtocol`; only the `ServiceLocatorSeams` (`createConfigurationSourceProvider`) knows the default source
- each setting group flows config model -> validator -> settings: the `ConfigurationProvider` loads and validates every group when it is created, then maps each with `ValidatedValue.expect(value, "<Validator>")`

## Gateways
- anything that comes from outside the process (an API response, a database row, a document) is read through a resource model's zod schema (`safeParse`), checked by a validator, then mapped to a domain model; resource models never leave their gateway, which returns domain models only
  ```
  managers/gateways/json_placeholder/
      json_placeholder_gateway.ts     sends the request, translates fetch errors into exceptions
      resource_models/                the JSON body's zod schema, every field optional
      validators/                     check the resource has every value the domain model needs
      mappers/                        map the validated resource to a domain model
  ```
  - a gateway's `resource_models`, `validators`, `mappers`, `query_builders` and `request_builders` are its own: nothing outside the gateway's folder imports them
  - a mapper takes the validated resource, unwraps each field with `ValidatedValue.expect(field, "<Validator>")`, and has a doc comment saying it expects a validated resource; each `expect` states a fact the validator checked, and throws only if the validator is wrong (an unexpected `Error`)
  - an input that can't be read is formatted by `ValidationErrorFormatter`: `format(error, rootName)` for a zod error (every problem, by where it is in the input, never the value), `formatWhole` for one about the input as a whole (a body that isn't JSON)
  - the gateway is where driver and HTTP errors are translated into `ApplicationBaseException`s; nothing above it sees a library's error
- when building a gateway's request is too involved to inline, it goes in a `request_builders/` subfolder of the gateway (the counterpart of a data gateway's `query_builders/`)
- an HTTP gateway is given its `fetch` (a seam's), the service's base URL, which it joins to each path, and the configured timeout, which it sends with each request (`AbortSignal.timeout`); `fetch` keeps no connection the gateway owns, so its `close()` has nothing to do
- a gateway retries a request that failed for a reason that may pass on its own, so a brief outage of the other system doesn't fail the app's operation
  - transient: a connection that couldn't be made or timed out, and a status of 429, 500, 502, 503 or 504 (or the error a service's API documents as temporary, e.g. `server_error`, a rate limit); never what sending again won't change: any other 4xx (a malformed request, a bad key, an exhausted quota even when it comes as a 429), and a response the gateway can't read
  - only a request that is safe to send twice is retried: a read, or a write the other system makes idempotent (e.g. with an idempotency key the gateway sends); a write that could happen twice is not retried, and its failure is told as it is
  - up to a few times (a setting of the gateway's settings group, e.g. `max_retries`, 3 by default), after a wait that doubles from about a second, up to a cap, with a little random jitter so callers that failed together don't retry together; when the service says how long to wait (`Retry-After`), that wait, up to a cap
  - every gateway waits the same: the wait comes from the shared `RetryDelayCalculator.getRetryDelayMilliseconds` (`common/calculators/retry_delay_calculator.ts`), and a `Retry-After` header is read with `RetryDelayCalculator.parseRetryAfter`; a gateway keeps its own retry loop, which decides on its own exceptions, but never its own copy of the backoff
  - the wait is `await setTimeout(milliseconds)` from `node:timers/promises`, so nothing blocks while it waits
  - each retry is logged as a warning (with the attempt and the wait), so an outage shows in the log even when a retry succeeds; once the retries run out, the last failure is the gateway's exception, as any other
  - a transient failure is its own exception (e.g. `JsonPlaceholderUnavailableException`), so the retry decides on the exception's kind, and a service that answers its callers can tell "try again later" (503) from a request that is wrong
  - a database gateway leaves connecting to its pool, whose acquire timeout already waits for a connection; it retries a statement only for what the database documents as safe to run again (e.g. Postgres's serialization failure, `40001`, or deadlock, `40P01`, for a transaction that is run again whole)

## Data layer
- the data layer repeats the architecture one level down: the `DataFacade` (the only entry point a manager sees) over one `DataManager`, which orchestrates every data gateway
  ```
  managers/data_layer/
      data_facade.ts
      managers/
          data_manager.ts
          gateways/
              relational_database/   relational_database_gateway.ts, query_builders/, resource_models/, validators/, mappers/
              document_database/     the same shape, with find/insert queries, documents and clients/
  ```
  - one manager means a data operation can grow to read from several databases and combine the results into one domain model without the layers above changing; callers never learn which database served a request
  - add a database by adding a gateway subfolder, a seam for its client, a `get*` in the `ServiceLocator`, and its use in `DataManager`
- relational data is kept in Postgres through `pg`: SQL is sent through a query builder with bound parameters (`$1`, `$2`); values are never formatted into a statement
  - a `SqlQuery` is made only by `SqlQuery.of`, which takes a statement whose type is its literal text (a string or a template with nothing interpolated), so one built from a value at run time (`${...}`, a concatenation) doesn't compile; nothing casts one to it
  - `SqlParameter` names only the types some query binds; add one (e.g. `string | number`) with the first query that binds it
  - a gateway holds the pool, which the `createRelationalDatabasePool` seam builds without connecting (`new pg.Pool(...)` connects when a statement is sent); the gateway handles the pool's `error` event (an idle connection the server dropped), which would otherwise end the process
  - the pool keeps no connection open while the app is idle, so a database that sleeps while nothing is connected to it can: `min: 0`, and a connection unused for a minute is closed (`idleTimeoutMillis`); a statement waits up to 10 seconds for a connection (`connectionTimeoutMillis`), then fails the request
- the schema changes only through a new file in `migrations/`, named `<timestamp>_<what_it_does>.sql` with a `-- migrate:up` and a `-- migrate:down` section; never edit a file that has run
  - the app never applies migrations: dbmate applies them to its database before it starts (`dbmate --migrations-dir migrations up`), in development as in a deployment
- `InMemoryDocumentDatabaseClient` (`document_database/clients/`) is a stand-in so the template needs no second server: its store is shared by every client with the same connection string and outlives `close()`; replace it with a real driver client rather than copying it into production code

## Logger
- `ApplicationLogger` (`managers/logger/application_logger.ts`) is an exception to the architecture
  - it is a global singleton (a class with static state) that owns its own handlers; any code anywhere imports it and logs via static methods (e.g. `ApplicationLogger.info(message, context)`)
  - it configures itself with defaults on first use; `ApplicationLogger.configure(...)` is only called at app boot (e.g. in `main()`), and `ApplicationLogger.close()` when the app ends
  - it is agnostic of any logging library: it routes `LogEntry` values to `LogHandlerProtocol` implementations (`managers/logger/handlers/`); Hono, `pg` and `fetch` log nothing of their own, so there is nothing to bridge to it (a library that does log gets a bridge in the service, as the Rust and Python templates have)
  - it is never created by, or registered in, the `ServiceLocator`, and never injected through constructors
  - each entry's source is the code that called it (file, line, column and function), read from the call stack
  - `ConsoleLogHandler` writes each entry as one line of JSON (time, level, logger, message, context, source, and an exception's type, message and stack), so a log store keeps an entry and its stack as one event; an exception's stack is its own, then its `Caused by:` chain; `LogRedactor` first takes out of the line what must never be logged (credentials in a URL, tokens, keys, email addresses), which the context never holds in the first place
- ERROR is for what an engineer must look at: an unexpected exception is logged with `ApplicationLogger.error("<what failed>", context, error)`, so its stack is kept; a client error (4xx) is a warning, so client errors can be found on their own, logged without a stack with `ApplicationLogger.logException(exception, LogLevel.Warning)`
- every exception is logged once, where it is caught, and nothing that can fail runs without something to catch it
  - every entry point installs `UncaughtExceptionLogger`: an uncaught exception is logged and ends the process (as Node would), and a promise rejected with nothing to handle it is logged and the process goes on
  - the service's `UnexpectedErrorHandler` (Hono's `onError`) catches an unexpected error in a request, logs it with the request, and answers it
  - a background loop logs the exception of each iteration and keeps going

## Service
- anything that runs and listens (an HTTP API, a gRPC server, a queue consumer, a stdio server) is a service, in `services/<name>/` beside the domain facade (e.g. `services/http/`), with its entry point in `package.json`'s scripts
  ```
  services/http/
      main.ts                 entry point (pnpm run example-app-http): logger, uncaught errors, HOST and PORT, serve
      http_service.ts         HttpService: creates the facade, serves until shutdown, closes it; the application and its contract
      http_environment.ts     what a request's context holds: the one facade
      routers/                each route maps the request in, calls one facade operation, maps the result out
      wire_models/            zod-openapi request and response schemas
      mappers/                wire models <-> domain models, and exceptions -> HTTP errors (HttpErrorMapper)
      exception_handlers/     ApplicationExceptionHandler, RequestRejectionHandler, UnexpectedErrorHandler: Hono's onError
      shutdown_signal.ts      Ctrl-C and SIGTERM
      export_openapi.ts       writes the service's contract to openapi.json
  ```
  - a service sits outside the domain container, calls only the domain facade, and holds no application logic
  - an HTTP service uses Hono; every service follows its framework's own idioms for routing, validating requests, and serializing responses
  - a router is a class whose static `register(application)` adds its routes, each `application.openapi(createRoute({...}), handler)`, so a route and its contract are one declaration
  - the domain container speaks only in its own terms: requests are mapped (`mappers/`) to domain models (or obvious primitives, such as ids) before the facade is called, and wire or framework types never reach it
  - every domain model the facade returns is mapped to the service's own response model (`wire_models/`) before it is serialized, so the domain can change without changing the API
  - it maps `ApplicationBaseException` families to protocol errors (e.g. `HttpErrorMapper`); a client error (4xx) is answered with the exception's own message, everything else is logged and answered generically so tables, SQL and upstream details never leak
  - a route lets the facade's exception propagate to Hono's `onError`, which hands it to `ApplicationExceptionHandler`, which maps, logs and answers it; a route never catches an exception itself
  - Hono's own answer to a request that doesn't fit is given in the same `ErrorResponse` shape, so callers handle one shape of error: a body or a path parameter that doesn't fit its schema is a 422 (`defaultHook`), and a body that isn't JSON a 400 (`RequestRejectionHandler`)
  - it creates the domain facade once when it starts and closes it when it stops (`HttpService.serve`); requests share that instance through the context (`context.get("exampleAppFacade")`, set by a middleware), and nothing builds one per request
  - a request model says what shape a request has (its fields and their types) and no more: whether a value is acceptable is decided inside the domain container, so the caller is answered with the domain's own message
  - each route names the responses it can answer with in its `responses` (e.g. `404: { description, content: { "application/json": { schema: ErrorResponseSchema } } }`), so they are part of the service's contract
  - an HTTP service's contract is `openapi.json` at the app's root: every route with its wire models, written by `pnpm run export-openapi` (`services/http/export_openapi.ts`); never edit it by hand, and run the command after changing a router or a wire model, so a change to the API shows up as a change to that file
  - it has its own entry point (e.g. `services/http/main.ts`), which reads where to listen: `HOST` and `PORT` when they are set (a deployment sets them), otherwise `127.0.0.1` and the app's development port (constants in that file)
  - it shuts down gracefully on Ctrl-C and SIGTERM: the server stops taking connections and finishes the requests under way, then the facade is closed

## Starting an app from the template
1. Copy the template, then rename `example-app` / `example_app` (`package.json`, `src/example_app_facade.ts` and its imports), `ExampleAppFacade`, `ExampleAppManager` and the `EXAMPLE_APP_` prefix of the environment variables.
2. Replace `processors/greeting/`, `processors/customer/`, `engines/customer_onboarding/` and the example settings with the app's own domain logic and configuration.
3. Replace the example databases, `Customer` and `AuditEvent` (and `migrations/`) with the app's own.
4. Replace the JSON Placeholder gateway and `UserProfile` with the HTTP services the app depends on.
5. Replace the example routers and wire models in `services/http/` with the app's own and run `pnpm run export-openapi`, or remove the service (and its scripts) if the app isn't served over HTTP.
6. Update the README's configuration and routes tables.
