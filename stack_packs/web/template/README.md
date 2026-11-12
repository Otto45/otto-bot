# Example App

An example web app built on the Domain Container architecture, with [TanStack Start](https://tanstack.com/start). It
saves customers, greets them, records an audit trail, and looks up user profiles from
[JSONPlaceholder](https://jsonplaceholder.typicode.com). The point isn't the features: it is the reference for how a
web app here is put together. The rules and design are in [AGENTS.md](AGENTS.md).

## What you need

- Node 24 or later, and [pnpm](https://pnpm.io) (`pnpm install` installs the dependencies).
- [dbmate](https://github.com/amacneil/dbmate), to create and migrate the database.
- A Postgres to run against: the factory's development Postgres (the repository's `compose.yaml`, on port 5433).

The checks need no database. Only running the app does.

## Running it

```sh
export DATABASE_URL="postgresql://previews:previews@localhost:5433/example_app?sslmode=disable"
dbmate --migrations-dir migrations --no-dump-schema up   # create the app's database and apply migrations/ (again after adding one)

pnpm install
pnpm run dev     # the app on http://localhost:3000, reloading as files change
pnpm run build   # the production server, in .output/
pnpm run start   # run it: it listens where HOST and PORT say (by default every interface, port 3000)
```

The production server stops cleanly on Ctrl-C or SIGTERM, once the requests under way have finished. Logs go to
stderr, one JSON object per line.

## Checks

```sh
pnpm run generate-route-tree   # writes src/routeTree.gen.ts from src/routes/ (the dev server and the build do too)
pnpm exec prettier --check .
pnpm exec eslint .
pnpm exec tsc --noEmit
pnpm run build                 # also fails when a page imports the server's code
```

## Pages

| Path | What it shows |
| --- | --- |
| `/` | What the app does |
| `/customers/new` | A form that saves a customer |
| `/customers/{customer_id}` | A customer's audit trail, and a button that greets them |
| `/users/{user_id}` | A JSONPlaceholder user, and a button that adds them as a customer |
| `/health` | `{"status": "ok"}`, for a deployment's health check |

## Configuration

Every setting has a default, so the app runs unconfigured once `dbmate up` has made its database.

| Variable | Default |
| --- | --- |
| `EXAMPLE_APP_GREETING_PREFIX` | `Hello` |
| `EXAMPLE_APP_RELATIONAL_DATABASE_CONNECTION_STRING` | `postgresql://previews:previews@localhost:5433/example_app?sslmode=disable` |
| `EXAMPLE_APP_DOCUMENT_DATABASE_CONNECTION_STRING` | `in-memory://document` |
| `EXAMPLE_APP_JSON_PLACEHOLDER_BASE_URL` | `https://jsonplaceholder.typicode.com` |
| `EXAMPLE_APP_JSON_PLACEHOLDER_TIMEOUT_SECONDS` | `10` |
| `EXAMPLE_APP_JSON_PLACEHOLDER_MAX_RETRIES` | `3` (a request JSON Placeholder failed on its side, or that couldn't reach it, is sent again this many times; 0 to 10) |
| `HOST`, `PORT` (production server only) | every interface, `3000` |

## Where things are

- `src/routes/`: the pages, one file each, named after their path; `src/components/`: what pages share.
- `src/server_functions/`: what the pages call on the server, each one operation of the facade.
- `src/domain_container/`: the app's domain, which runs only on the server: `example_app_facade.ts` is its single
  entry point, and `managers/` holds the workflows, business logic, and the gateways to databases and other services.
- `src/styles/app.css`: the theme (colours, fonts, radii) every component is styled with.
- `migrations/`: the database schema, one SQL file per change.

Starting a new app from this one is described at the end of [AGENTS.md](AGENTS.md).
