# otto-bot

A coding agent harness that builds apps on the Domain Container architecture, in Rust, Python and TypeScript. It is
itself a Domain Container app, in Rust. The design is [docs/DESIGN.md](docs/DESIGN.md); the rules every change to it
follows are [AGENTS.md](AGENTS.md).

Status: build steps 1-4 of [docs/DESIGN.md](docs/DESIGN.md) are done. An agent
works in a workspace: it reads, lists and searches files, edits them with `apply_patch`, and runs the stack pack's
checks, remembering the whole session. Every request opens with cached layers (the stack's rules, then the workspace's
project map), and goes over a WebSocket connection that sends only what is new. The checks include an architecture lint
of the stack pack's rules, and the stack's formatters (the code's formatting, the OpenAPI contract) run before the checks,
so no agent spends a turn on them. Agents run in roles on lanes of their own: the orchestrator (the session's own agent) makes a small change
itself and plans a bigger one (`submit_plan`); the implementor builds
it, until every check passes; then the reviewer reviews the change's diff, and the implementor fixes what it says must
be fixed. Stack packs: Rust, Python and TypeScript; `otto new` starts an app from a pack's template.

## What you need

- Rust: `rust-version` in `Cargo.toml` is the oldest toolchain it builds with.
- An OpenAI API key, in `OPENAI_API_KEY`.

## Running it

```sh
export OPENAI_API_KEY=...
cargo run -- --workspace ../my-app      # the interactive CLI, working in ../my-app (default: the current directory)
cargo run -- --service-tier fast        # a session at the fast tier
cargo run -- new orders-app --stack python --in ..   # start ../orders-app from the Python template, then converse in it
cargo run -- --help
```

`new` copies the stack pack's template into a new (or empty) folder named after the app, renames its example names
(`example_app`, `ExampleAppFacade`, `EXAMPLE_APP_`, ...) after the app, and runs the pack's formatters, with no model;
the example components stay as the pattern for each kind, for the first request to replace.

Type a request and press Enter: otto works on it until it is done, showing each tool it uses, then the run's turns
and tokens. Ctrl-C cancels a run under way; Ctrl-D, `/exit` or Ctrl-C at the prompt quits.

otto reads and writes nothing outside the workspace, and never reads `.env` files or `.git/`. Its checks run without
`OPENAI_API_KEY` in their environment.

Everything otto keeps is under `~/.otto-bot/` (`OTTO_BOT_HOME`): `logs/otto.log` (one JSON object per line) and
`sessions/<id>/`, with each agent's conversation (`conversation_items/<lane>.jsonl`, the session's own agent being
`orchestrator`), token usage (`usage_records.jsonl`) and accepted plans (`plans.jsonl`).

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
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
| `OTTO_BOT_OPENAI_TRANSPORT` | `websocket` (a connection per session that sends only what's new each turn; or `http`) |
| `OTTO_BOT_HOME` | `~/.otto-bot` |
| `OTTO_BOT_STACK_PACKS_DIRECTORY` | this checkout's `stack_packs/` |
| `OTTO_BOT_MAX_TURNS` | `60` (model turns one run may take) |
| `OTTO_BOT_CHECK_TIMEOUT_SECONDS` | `600` (per check) |

## Where things are

- `src/otto_bot_facade.rs`: the domain's single entry point; every client calls through it.
- `src/managers/`: the domain: sessions, the agent's runs (`engines/agent_run/`), the gateways to the OpenAI API, the
  workspace and its commands, and the data layer (sessions and stack packs, as files).
- `src/services/cli/`: the interactive CLI, a thin layer over the facade.
- `stack_packs/rust/`: what otto-bot knows about building Rust apps: the template app and its rules.
