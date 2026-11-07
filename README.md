# otto-bot

A coding agent harness that builds apps on the Domain Container architecture, in Rust, Python and TypeScript. It is
itself a Domain Container app, in Rust. The design is [docs/DESIGN.md](docs/DESIGN.md); the rules every change to it
follows are [AGENTS.md](AGENTS.md).

Status: build step 1 of [docs/DESIGN.md](docs/DESIGN.md). The CLI runs a session in which each request is one
OpenAI Responses API call; the agent's tools, its memory of the conversation and the session store come next.

## What you need

- Rust: `rust-version` in `Cargo.toml` is the oldest toolchain it builds with.
- An OpenAI API key, in `OPENAI_API_KEY`.

## Running it

```sh
export OPENAI_API_KEY=...
cargo run                              # the interactive CLI
cargo run -- --service-tier fast       # a session at the fast tier
cargo run -- --help
```

Type a request and press Enter. Ctrl-C cancels a run under way; Ctrl-D, `/exit` or Ctrl-C at the prompt quits.
Warnings and errors are logged to stderr, one JSON object per line.

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

## Where things are

- `src/otto_bot_facade.rs`: the domain's single entry point; every client calls through it.
- `src/managers/`: the domain: sessions, the agent's runs, and the gateway to the OpenAI API.
- `src/services/cli/`: the interactive CLI, a thin layer over the facade.
- `stack_packs/rust/`: what otto-bot knows about building Rust apps: the template app and its rules.
