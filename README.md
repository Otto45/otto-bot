# otto-bot

A coding agent harness that builds apps on the Domain Container architecture, in Rust, Python and TypeScript. It is
itself a Domain Container app, in Rust. The design is [docs/DESIGN.md](docs/DESIGN.md); the rules every change to it
follows are [AGENTS.md](AGENTS.md).

Status: build step 3 of [docs/DESIGN.md](docs/DESIGN.md). One agent works in a workspace: it reads, lists and
searches files, edits them with `apply_patch`, and runs the stack pack's checks, remembering the whole session. Every
request opens with cached layers (the stack's rules, then the workspace's project map), and goes over a WebSocket
connection that sends only what is new. The checks include an architecture lint of the stack pack's rules.
Scaffolding, sub-agents and the other stack packs come next.

## What you need

- Rust: `rust-version` in `Cargo.toml` is the oldest toolchain it builds with.
- An OpenAI API key, in `OPENAI_API_KEY`.

## Running it

```sh
export OPENAI_API_KEY=...
cargo run -- --workspace ../my-app      # the interactive CLI, working in ../my-app (default: the current directory)
cargo run -- --service-tier fast        # a session at the fast tier
cargo run -- --help
```

Type a request and press Enter: otto works on it until it is done, showing each tool it uses, then the run's turns
and tokens. Ctrl-C cancels a run under way; Ctrl-D, `/exit` or Ctrl-C at the prompt quits.

otto reads and writes nothing outside the workspace, and never reads `.env` files or `.git/`. Its checks run without
`OPENAI_API_KEY` in their environment.

Everything otto keeps is under `~/.otto-bot/` (`OTTO_BOT_HOME`): `logs/otto.log` (one JSON object per line) and
`sessions/<id>/`, with each session's conversation (`conversation_items.jsonl`) and token usage
(`usage_records.jsonl`).

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
