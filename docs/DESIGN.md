# otto-bot design

otto-bot is a coding agent harness, written in Rust, that builds and changes apps following the Domain Container
architecture, in Rust, Python and TypeScript. It is itself a Domain Container app.

Status: planned, not yet implemented.

## Goals

1. Model calls go through the OpenAI Responses API.
2. Prompt caching is maximized, to keep generated code cheap.
3. Work is divided among sub-agents, including when starting a new project.
4. Code is written as fast as possible without giving up quality or security.

The architecture is what makes these reachable. Every file's role follows from its path (a file in
`managers/gateways/<system>/validators/` can only be a gateway's validator), so the harness can:

- **generate instead of infer**: module wiring, service locator registration, facade pass-throughs, renames and
  contract exports are fixed patterns the harness writes itself (49 of the Rust template's 138 source files are
  `mod.rs`);
- **compute context instead of exploring**: for any task it knows which files an agent needs, and sends them up front;
- **compute the division of work**: component boundaries and dependency rules split a feature into packages with
  disjoint files that can be built in parallel.

## Decisions

| Topic | Decision |
| --- | --- |
| Target stacks | Rust, Python, TypeScript |
| Model | `gpt-6.1-sol` for every role (one model, one shared prompt cache) |
| Service tier | `--service-tier standard\|fast\|ultrafast`, default `standard` (sent as `"default"`) |
| Clients | several, all over one domain facade; an interactive CLI first |
| Data retention | `store=false`; the harness keeps every conversation itself |
| Approvals | none: runs are unattended, and will run in their own sandbox VM |
| Tests in generated apps | deferred; packs keep an empty `test` command slot |

## Stack packs

The harness core knows the architecture's component kinds and dependency rules, and nothing about any language.
Everything language-specific is in a stack pack:

```
stack_packs/<stack>/
    pack.toml       detection (e.g. Cargo.toml), check commands, layout map (component kind -> path pattern),
                    naming rules, rename map, example components to strip
    rules.md        the stack's architecture rules (the template's AGENTS.md)
    template/       the starting app: one example of every component kind
    scaffolds/      skeletons per component kind (processor, engine, gateway, data gateway, router, ...)
    queries/        tree-sitter queries that extract public signatures, imports and component declarations
```

| | Rust | Python | TypeScript |
| --- | --- | --- | --- |
| Template | the existing Rust template | ported by the harness, reviewed | ported by the harness, reviewed |
| Module wiring | `mod.rs` | `__init__.py` | `index.ts` barrels, or none |
| Checks | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `export-openapi --check` | formatter, linter, strict type checker, contract check | formatter, linter, `tsc --noEmit` (strict), contract check |

- Without tests, a strict type checker is part of the Python and TypeScript packs' rules.
- The Python and TypeScript templates are ported from the Rust template by the harness itself, as its first real
  multi-agent job, and reviewed before they become packs.

## Prompt and cache design

Every request, for every agent, is built in the same layers, from most shared to least shared, each ending in an
explicit cache breakpoint (`prompt_cache_options.mode: "explicit"`, `prompt_cache_breakpoint` on an `input_text`
block of a developer message):

| Breakpoint | Content | TTL | Shared by |
| --- | --- | --- | --- |
| BP1: stack prefix | the fixed tool set, harness instructions, `rules.md`, the pack's template code | 12h | every agent and project of the stack |
| BP2: project map | tree; signatures of the facade, service locator and DataFacade; models; exception kinds; settings | 12h / 30m | every agent in the session |
| BP3: work package | full text of the files the agent owns; signatures of what it calls | 30m | one agent |
| conversation | append-only items, a breakpoint after each tool result | 30m | one agent |

Rules that keep the cache hitting:

- **One tool list, one model.** Every role gets the same tools in the same order and is restricted with
  `allowed_tools`, never by removing tools. Caches are per model.
- **Reasoning effort per role** is set by appending a `configuration_update` item, not by changing
  `reasoning.effort`, which would change the prefix.
- **Nothing variable above BP3**: no timestamps, ids or unsorted lists. The project map is rendered deterministically
  (sorted paths, canonical formatting).
- **The project map is never rewritten mid-wave.** Changes are appended as "changes since the map was built"; the
  map is rebuilt only between waves, as a deliberate cache break.
- **Warm, then fan out.** After the skeleton step, one `prewarm` request writes BP2; then all workers start together
  and read it.
- **Measure.** Every call records `cached_tokens`, `cache_write_tokens` and its details in the usage ledger.

Rough cost on `gpt-6.1-sol` ($2 / $10 per million input / output tokens; cached reads 0.05x; 12h writes 2.5x): the
~45k-token stack prefix costs about $0.23 to write and about $0.0045 per request to read, against $0.09 uncached.
Output dominates the cost, which is why the speed levers below also cut cost.

## Model I/O

- **Responses API over WebSocket mode**: each turn sends only new input items (up to ~40% faster on long tool-call
  runs). Each sub-agent is a `stream_id` lane; a connection allows 16 responses in flight and 32 lanes, and lasts 60
  minutes. HTTP is the fallback.
- **`store=false`.** The harness's session log holds every lane's full ordered item list, including encrypted
  reasoning (`include: ["reasoning.encrypted_content"]`) and compaction items. When a lane is lost (disconnect, the
  60-minute limit, an error that evicts it), the gateway replays the item list from the log; the prefix is unchanged,
  so the replay is a cache hit. A forked lane's `response.in_progress` is awaited before its parent lane advances.
- **Edits use the native `apply_patch` tool** (V4A diffs, one file operation per call, several calls per response).
  The harness applies them and answers each `call_id` with `apply_patch_call_output`; a mismatch is rejected with a
  precise reason.
- **Tools**, the same set for every role:

  | Tool | What it does |
  | --- | --- |
  | `read_files` | many paths, globs or component names in one call |
  | `search` | text search across the project |
  | `apply_patch` | native file edits |
  | `scaffold` | creates a component and its wiring, deterministically |
  | `run_checks` | runs the pack's check commands |
  | `request_contract_change` | asks the orchestrator to change a frozen interface |
  | `report` | the agent's structured, terse completion report |

- **Compaction** (`/responses/compact`, or `context_management`) only at wave boundaries, since it resets cache reuse.
- **Service tier** comes from the session's options; the ledger records the tier each response actually ran at,
  since `fast` can be downgraded under load.

## Workflow

### Implementing a feature

1. **Plan.** One agent at high reasoning effort returns a structured work plan: new or changed models, exception
   variants, settings and facade operations; new components (kind, name, signatures); migrations and routes; and work
   packages with the files each owns. The harness checks it without a model: every path is legal for its kind, no
   file is owned twice, and no dependency rule is broken (manager -> manager, engine -> engine, ...). A rejected plan
   goes back to the planner. Ambiguities in the request are decided by the planner and recorded as assumptions.
2. **Skeleton.** The harness scaffolds every new component (files, module wiring, the locator's `get_*`, cache slot
   and close registration, facade pass-throughs). The planner writes only the contracts: models, exception variants,
   and signatures with placeholder bodies. The skeleton must pass the checks; it is checkpointed, and the interfaces
   are frozen.
3. **Fan out.** One worker per package (a gateway with its resource models, validators and mappers; the data layer
   with its migration; a processor; an engine; the service's routes and wire models). A worker may only patch files
   it owns; an interface change goes through `request_contract_change`.
4. **Integrate.** Manager methods (short coordinations) are written, then the full checks and the architecture lint
   run. Each error goes to the worker that owns the file, on its still-warm lane.
5. **Review.** The architecture lint, then a lean reviewer that sees BP1 and the diff only, focused on security.

Each worker has its own git worktree, so it compiles against the skeleton plus its own changes only. Build output is
copied into each worktree with copy-on-write clones (`cp -c` on APFS) so builds start warm. Disjoint ownership makes
merging conflict-free.

### Starting a project

The pack's template is copied, renamed and stripped of its examples without a model, leaving a compiling empty app.
The planner then designs the domain from the description, and the feature workflow builds it.

## Speed levers

In order of expected impact:

1. Generate instead of infer (scaffolds, wiring, renames, contract exports).
2. Pre-load context, so no turn is spent exploring.
3. Parallel workers sharing the cached BP1 and BP2.
4. Small output: diffs, `text.verbosity: low`, structured reports, reasoning effort matched to the role.
5. Checks run after each patch batch and their results come back in the same tool result, filtered to the worker's
   files and deduplicated.
6. WebSocket transport, and `fast` / `ultrafast` when speed is worth the price.
7. Warm incremental builds in every worktree.

## Security

The sandbox VM the harness runs in is the security boundary: its isolation and outbound network policy contain what
an agent does. Inside it, the harness keeps what protects the work and what keeps secrets from leaving:

- **Workspace confinement and file ownership**: paths resolve to their real location, symlinks can't escape, and
  every patch is checked against its worker's files.
- **Declared commands only**: workers run the pack's check commands, not a general shell.
- **Secret redaction** before anything is sent to the model; `.env` files are never read.
- **File contents are data, never instructions.**
- **The architecture's own rules**: bound SQL parameters, generic answers to 5xx errors, redaction in logs, all
  enforced by the lint and the review.
- **The VM image holds no long-lived credentials** other than the OpenAI key.

Until the VM exists, the harness runs with the user's own permissions, and should only be pointed at trusted
repositories.

## The harness's layout

```
src/
    otto_bot_facade.rs              start_session, submit, subscribe, cancel, close
    services/cli/                   interactive CLI (first client); later e.g. services/stdio/, services/http/
    managers/
        otto_bot_manager.rs         coordinates plan -> skeleton -> fan out -> integrate -> review
        processors/
            work_plan/              checks a plan against the pack's layout, ownership and dependency rules; orders waves
            prompt_assembly/        builds the layered prompt, renders it deterministically, places breakpoints
            architecture_map/       the project map, from tree-sitter parses
            architecture_lint/      deterministic rule checks
            scaffold/               renders component skeletons and wiring edits from pack scaffolds
        engines/
            agent_run/              runs one agent to completion: model turn -> tool calls -> results; applies
                                    patches, enforces ownership
            project_bootstrap/      copies, renames and strips a pack's template (no model)
        gateways/
            openai_responses/       WebSocket and HTTP transport, lanes, replay from the session log
            workspace/              reads many files, applies V4A patches atomically, lists the tree
            command_runner/         runs a pack's declared commands
            git/                    worktrees, checkpoints, diffs
        data_layer/
            data_facade.rs, managers/data_manager.rs
            managers/gateways/
                session_store/      SQLite: sessions, plans, lane item logs, usage ledger
                stack_pack_store/   the stack packs
        common/                     diagnostics parsing (used by the engine and the manager)
        models/, exceptions/, logger/, configuration_providers/, service_locator/
stack_packs/rust/, stack_packs/python/, stack_packs/typescript/
```

- **One engine talks to the model.** An engine never uses another engine, so `agent_run` is the only one, and the
  manager calls it for every role.
- **The facade works for any client.** `submit` runs to completion on its own in a background task. Progress is a
  stream of `SessionEvent`, a domain enum: plan proposed, package started, patch applied, check result, usage, done.
  Each service maps the events to its own wire format.
- **Any client can resume any session**, because the session log is the whole record of it.
- **Session options** (such as the service tier) are passed to `start_session` and override configured defaults,
  which only the configuration provider reads from the environment.

## Build order

1. The harness skeleton from the Rust template; the Rust pack; the facade's session and event API; the CLI service.
2. A single-agent loop over HTTP, replaying the item list from the session log; the usage ledger; `--service-tier`.
3. WebSocket lanes with replay; layered breakpoints; the project map; `scaffold`; the architecture lint.
4. The multi-agent workflow: plan, skeleton, workers in worktrees, integration, review.
5. The Python pack, then the TypeScript pack, ported by the harness; starting new projects.
6. Resume, compaction, a second client.

A fixed set of feature tasks on template apps is run against every change to the harness, recording wall time,
cost, cache hit rate, lint violations and whether the checks pass, so speed and quality trade-offs are measured.

## To verify early

- A `configuration_update` item changes reasoning effort without breaking the cached prefix.
- Changing the service tier mid-session keeps cache hits.
- The `12h` cache TTL is available with zero data retention; if not, `30m` with steady reuse.
- Forking WebSocket lanes, and replay after a lost lane, behave as documented with `store=false`.

## Open

- Library choices for the Python and TypeScript templates. Suggested: FastAPI, pydantic, httpx, asyncpg, uv, ruff,
  pyright (strict); Node, Fastify or Hono, zod, `pg`, `tsc` (strict), eslint.
- Whether the `software-engineer-python` skill holds the Python conventions for the architecture.
- Tests in generated apps.
