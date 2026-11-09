# Example App

An example app built on the Domain Container architecture, in TypeScript. It saves customers, greets them, records an
audit trail, and looks up user profiles from [JSONPlaceholder](https://jsonplaceholder.typicode.com). The point
isn't the features: it is the reference for how an app here is put together. The rules and design are in
[AGENTS.md](AGENTS.md).

## What you need

- Node 24 or later, which runs the TypeScript as it is, and [pnpm](https://pnpm.io) (`pnpm install` installs the
  dependencies).
- [dbmate](https://github.com/amacneil/dbmate), to create and migrate the database.
- A Postgres to run against: the factory's development Postgres (the repository's `compose.yaml`, on port 5433).

The checks need no database. Only running the app does.

## Running it

```sh
export DATABASE_URL="postgresql://previews:previews@localhost:5433/example_app?sslmode=disable"
dbmate --migrations-dir migrations --no-dump-schema up   # create the app's database and apply migrations/ (again after adding one)

pnpm install
pnpm run example-app        # run the app once: it saves, greets and onboards a customer, and prints what happened
pnpm run example-app-http   # serve the HTTP API on http://127.0.0.1:8000
```

The HTTP service listens where `HOST` and `PORT` say when they are set, and stops cleanly on Ctrl-C or SIGTERM.
Logs go to stderr, one JSON object per line.

## Checks

```sh
pnpm exec prettier --check .
pnpm exec eslint .
pnpm exec tsc --noEmit
pnpm run export-openapi --check   # openapi.json matches the code (without --check: rewrite it)
```

## HTTP API

The full contract is [openapi.json](openapi.json).

| Method | Path | What it does |
| --- | --- | --- |
| `PUT` | `/customers/{customer_id}` (body: `name`, `email`) | Saves a customer (204) |
| `POST` | `/customers/{customer_id}/greetings` | Greets a saved customer and records it (201) |
| `GET` | `/customers/{customer_id}/audit-events` | Lists a customer's audit events, oldest first |
| `GET` | `/user-profiles/{user_id}` | Looks up a JSONPlaceholder user |

Every error is answered as `{"error": "..."}`. A 4xx says what to fix; a 5xx says only that the request failed,
and the details are in the logs.

## Configuration

Every setting has a default, so the app runs unconfigured once `dbmate up` has made its database.

| Variable | Default |
| --- | --- |
| `EXAMPLE_APP_GREETING_PREFIX` | `Hello` |
| `EXAMPLE_APP_RELATIONAL_DATABASE_CONNECTION_STRING` | `postgresql://previews:previews@localhost:5433/example_app?sslmode=disable` |
| `EXAMPLE_APP_DOCUMENT_DATABASE_CONNECTION_STRING` | `in-memory://document` |
| `EXAMPLE_APP_JSON_PLACEHOLDER_BASE_URL` | `https://jsonplaceholder.typicode.com` |
| `EXAMPLE_APP_JSON_PLACEHOLDER_TIMEOUT_SECONDS` | `10` |
| `HOST`, `PORT` (HTTP service only) | `127.0.0.1`, `8000` |

## Where things are

- `src/example_app_facade.ts`: the app's single entry point; everything else calls through it.
- `src/managers/`: the domain: workflows, business logic, and the gateways to databases and other services.
- `src/services/http/`: the HTTP API, a thin layer over the facade.
- `migrations/`: the database schema, one SQL file per change.

Starting a new app from this one is described at the end of [AGENTS.md](AGENTS.md).
