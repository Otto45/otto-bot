# otto-bot

A coding agent harness that builds apps on the Domain Container architecture, in Rust, Python and TypeScript. It is
itself a Domain Container app, in Rust. The design is [docs/DESIGN.md](docs/DESIGN.md); the rules every change to it
follows are [AGENTS.md](AGENTS.md).

Status: build steps 1-5 of [docs/DESIGN.md](docs/DESIGN.md) are done. An agent
works in a workspace: it reads, lists and searches files, edits them with `apply_patch`, and runs the stack pack's
checks, remembering the whole session. Every request opens with cached layers (the stack's rules, then the workspace's
project map), and goes over a WebSocket connection that sends only what is new. The checks include an architecture lint
of the stack pack's rules, and the stack's formatters (the code's formatting, the OpenAPI contract) run before the checks,
so no agent spends a turn on them. Agents run in roles on lanes of their own: the orchestrator (the session's own
agent) holds the conversation, asks what only the person can decide, and briefs the implementor; the implementor plans
and builds each brief from a fresh start, until every check passes; then the reviewer reviews the change's diff, the
implementor fixes what it says must be fixed, and the orchestrator tells the person how it went. Stack packs: Rust, Python and TypeScript; `otto new` starts an app from a pack's template.

## What you need

- Rust: `rust-version` in `Cargo.toml` is the oldest toolchain it builds with.
- An OpenAI API key, in `OPENAI_API_KEY`.

## Running it

```sh
export OPENAI_API_KEY=...
cargo run -- --workspace ../my-app      # the interactive CLI, working in ../my-app (default: the current directory)
cargo run -- --service-tier fast        # a session at the fast tier
cargo run -- new orders-app --stack python --in ..   # start ../orders-app from the Python template, then converse in it
cargo run -- sessions --workspace ../my-app         # the sessions started in ../my-app, most recently active first
cargo run -- --session <id>             # resume one, in its workspace and at its tier, where it was left
cargo run -- --help
```

`new` copies the stack pack's template into a new (or empty) folder named after the app, renames its example names
(`example_app`, `ExampleAppFacade`, `EXAMPLE_APP_`, ...) after the app, and runs the pack's formatters, with no model;
the example components stay as the pattern for each kind, for the first brief to replace.

Type a request and press Enter: otto works on it until it is done, showing each tool it uses, then the run's turns
and tokens. Ctrl-C cancels a run under way; Ctrl-D, `/exit` or Ctrl-C at the prompt quits. The session's id is shown
when it starts: `--session <id>` carries on with it later, its conversation, project map and briefs as they were.

otto reads and writes nothing outside the workspace, and never reads `.env` files or `.git/`. Its checks run without
`OPENAI_API_KEY` in their environment.

Everything otto keeps is under `~/.otto-bot/` (`OTTO_BOT_HOME`): `logs/otto.log` (one JSON object per line) and
`sessions/<id>/`, with the session (`session.json`), what the person and otto said (`transcript.jsonl`), each agent's conversation (`conversation_items/<lane>.jsonl`, the session's own agent being
`orchestrator`), token usage (`usage_records.jsonl`), accepted briefs (`briefs.jsonl`), the steps its runs got to (`run_records.jsonl`) and what its next review is made against (`review_baselines.jsonl`); each brief's implementor and reviewer are
`implementor_<n>` and `reviewer_<n>`. `locks/` holds the locks that keep two runs, in this process or another, out of one
app at once.

## The HTTP API

`otto-http` is otto's second client, for the software factory platform: it starts apps by name, holds sessions in
them, and streams what happens as server-sent events. Apps are kept under `OTTO_BOT_WORKSPACES_DIRECTORY` (default
`~/.otto-bot/apps/`), each in a folder named after it; a client names an app, never a path.

```sh
export OPENAI_API_KEY=... OTTO_BOT_HTTP_API_TOKEN=...   # every request but /health bears it: Authorization: Bearer ...
cargo run --bin otto-http                               # serves on http://127.0.0.1:8650 (HOST and PORT override it)
cargo run --bin export-openapi                          # rewrites openapi.json (--check: fails if it is out of date)
```

The contract is [openapi.json](openapi.json).

| Method | Path | What it does |
| --- | --- | --- |
| `POST` | `/apps` (body: `name`, `stack`) | Starts an app from its stack's template (201); its kebab-case name addresses it from then on |
| `GET` | `/apps/{app_name}/sessions` | Lists the app's sessions, most recently active first |
| `POST` | `/apps/{app_name}/sessions` (body: `service_tier`, optional) | Starts a session (201, with its id) |
| `GET` | `/apps/{app_name}/sessions/{session_id}` | The session: when it started and was last active, its first request |
| `POST` | `/apps/{app_name}/sessions/{session_id}/messages` (body: `text`) | Sends the person's message (202); 409 while the session, or another session of the app, is working |
| `GET` | `/apps/{app_name}/sessions/{session_id}/messages` | What the person and otto said, oldest first, in plain words |
| `POST` | `/apps/{app_name}/sessions/{session_id}/cancel` | Stops the run under way (204) |
| `GET` | `/apps/{app_name}/sessions/{session_id}/events` | The session's events, as server-sent events, for as long as the client stays |
| `GET` | `/health` | Whether the service is up (no token) |

- **Events** are JSON with a `type` (`run_started`, `assistant_replied`, `tool_used`, `brief_accepted`,
  `brief_implemented`, `lane_compacted`, `run_completed`, `run_failed`, `run_cancelled`), each with a numbered `id`.
  What is meant for the person is an `assistant_replied` with `to_person: true`: plain words, for someone who isn't
  technical. A stream opened without an id first replays the current run from its start (so one opened just after a
  message was sent misses nothing); one opened with `?after=<id>` or `Last-Event-ID` (which an `EventSource` sends when
  it reconnects) replays what came after that event.
- **Sessions resume on their own**: after a restart, or once one was let go after `OTTO_BOT_IDLE_SESSION_MINUTES`
  with no run and no client following it, the next request to it serves it again, where it was left.
- **One run per app**: a message for an app another of its sessions is working for is refused (409), since both runs
  would change the same files.
- Errors are answered as `{"error": "..."}`: a 4xx says what to fix; a 5xx (and a failed run's event) says only that
  the request failed, and the details are in the log, which goes to stderr, one JSON object per line.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --bin export-openapi -- --check
```

## Configuration

| Variable | Default |
| --- | --- |
| `OPENAI_API_KEY` | none: required |
| `OTTO_BOT_MODEL` | `gpt-6.1-sol` |
| `OTTO_BOT_SERVICE_TIER` | `standard` (`standard`, `fast` or `ultrafast`; `--service-tier` overrides it) |
| `OTTO_BOT_OPENAI_BASE_URL` | `https://api.openai.com/v1` |
| `OTTO_BOT_OPENAI_TIMEOUT_SECONDS` | `600` |
| `OTTO_BOT_OPENAI_IDLE_TIMEOUT_SECONDS` | `120` (WebSocket: a response silent this long has stalled, and is sent again on a fresh connection) |
| `OTTO_BOT_OPENAI_MAX_RETRIES` | `3` (a request the API failed on its side, rate-limited, or couldn't be reached is sent again this many times, after 1, 2, 4... seconds or the `Retry-After` it asks for; 0 to 10) |
| `OTTO_BOT_OPENAI_TRANSPORT` | `websocket` (a connection per session that sends only what's new each turn; or `http`) |
| `OTTO_BOT_HOME` | `~/.otto-bot` |
| `OTTO_BOT_STACK_PACKS_DIRECTORY` | this checkout's `stack_packs/` |
| `OTTO_BOT_MAX_TURNS` | `60` (model turns one run may take) |
| `OTTO_BOT_ORCHESTRATOR_COMPACT_TOKENS` | `400000` (once a run ends, the session's conversation is compacted if its last request was this long) |
| `OTTO_BOT_LANE_COMPACT_TOKENS` | `600000` (any agent's conversation is compacted between turns once a request is this long) |
| `OTTO_BOT_CHECK_TIMEOUT_SECONDS` | `600` (per check) |
| `OTTO_BOT_WORKSPACES_DIRECTORY` | `~/.otto-bot/apps` (where the HTTP API keeps the apps it starts) |
| `OTTO_BOT_IDLE_SESSION_MINUTES` | `30` (a session with no run and no client following it is let go after this long, and resumed when next used) |
| `OTTO_BOT_HTTP_API_TOKEN` | none: `otto-http` won't start without it |
| `HOST`, `PORT` (`otto-http` only) | `127.0.0.1`, `8650` |

## Where things are

- `src/otto_bot_facade.rs`: the domain's single entry point; every client calls through it.
- `src/managers/`: the domain: sessions, the agent's runs (`engines/agent_run/`), the gateways to the OpenAI API, the
  workspace and its commands, and the data layer (sessions and stack packs, as files).
- `src/services/cli/`: the interactive CLI, a thin layer over the facade.
- `src/services/http/`: the HTTP API (`otto-http`), another thin layer over the same facade.
- `stack_packs/rust/`: what otto-bot knows about building Rust apps: the template app and its rules.
