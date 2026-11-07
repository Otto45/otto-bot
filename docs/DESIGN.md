# otto-bot design

otto-bot is a coding agent harness, written in Rust, that builds and changes apps following the Domain Container
architecture, in Rust, Python and TypeScript. It is itself a Domain Container app.

Status: build steps 1-3 are done (a single agent with the layered cache, WebSocket lanes, the project map, the
architecture lint and scaffold); step 4, the multi-agent workflow, is under way.

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
| Session store | append-only JSON Lines files, one folder per session under `~/.otto-bot/sessions/` (no database, no migrations) |
| Approvals | none: runs are unattended, and will run in their own sandbox VM |
| Tests in generated apps | deferred; packs keep an empty `test` command slot |

## Stack packs

The harness core knows the architecture's component kinds and dependency rules, and nothing about any language.
Everything language-specific is in a stack pack:

```
stack_packs/<stack>/
    pack.toml       detection (e.g. Cargo.toml), check commands, the project map and lint queries, scaffold kinds,
                    the layout (the files only the orchestrator writes, and where a worker's files may be), rename map
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
  `reasoning.effort`, which misses the cache (see Verified).
- **Nothing variable above BP3**: no timestamps, ids or unsorted lists. The project map is rendered deterministically
  (sorted paths, canonical formatting).
- **The project map is never rewritten mid-wave.** Changes are appended as "changes since the map was built"; the
  map is rebuilt only between waves, as a deliberate cache break. (Built in step 3: a session keeps its map from run
  to run and notes the changes as developer messages; it is rebuilt when the notes would exceed a quarter of it.)
- **Fan out together.** After the skeleton step all workers start at once: concurrent requests on a cold prefix
  share one cache write (see Verified), so no prewarm request is needed.
- **Measure.** Every call records `cached_tokens`, `cache_write_tokens` and its details in the usage ledger.

Rough cost on `gpt-6.1-sol` ($2 / $10 per million input / output tokens; cached reads 0.05x; 12h writes 2.5x): the
~45k-token stack prefix costs about $0.23 to write and about $0.0045 per request to read, against $0.09 uncached.
Output dominates the cost, which is why the speed levers below also cut cost.

## Model I/O

- **Responses API over WebSocket mode**: each turn sends only new input items (up to ~40% faster on long tool-call
  runs). Each agent is a lane (`<session>/<lane>`) with a connection of its own, which lasts 60 minutes; HTTP is the
  fallback. (A connection could carry several lanes as `stream_id`s, 16 responses in flight and 32 lanes, if the
  number of connections ever matters.)
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
  | `submit_plan` | the orchestrator's work plan, checked by the harness (a rejected plan is the call's error) |
  | `report` | a worker's or reviewer's structured, terse completion report |
  | `request_contract_change` | (later) asks the orchestrator to change a frozen interface mid fan-out |

- **Compaction** (`/responses/compact`, or `context_management`) only at wave boundaries, since it resets cache reuse.
- **Service tier** comes from the session's options; the ledger records the tier each response actually ran at,
  since `fast` can be downgraded under load.

## Workflow

### Implementing a feature

The manager keeps the order: each phase is one call to `AgentRunEngine`, which runs one agent (a lane) in a role.

1. **Orchestrator.** The session's agent, as in steps 1-3, with one more tool: `submit_plan`. A small job it does
   itself, with no plan, so a small request pays nothing for the workflow. A bigger one it plans: new or changed models,
   exception variants and settings; the components to scaffold (kind, name, within); and work packages, each with the
   files it owns and a brief. The `WorkPlanProcessor` checks the plan without a model: every path is legal for its kind
   (the pack's `[layout]`), no file is owned twice, and no worker owns a wiring file (module files, the service locator,
   the facade). A rejected plan is the tool call's error, so the orchestrator fixes it in the same turn loop.
   Ambiguities in the request are decided by the orchestrator and recorded as assumptions in the plan.
2. **Skeleton.** The harness scaffolds every component of the plan. The orchestrator, on its still-warm lane, writes
   only the contracts: models, exception variants, and signatures with placeholder bodies. The skeleton must pass every
   check (the architecture lint catches a contract that breaks a dependency rule); then it is snapshotted and the
   contracts are frozen.
3. **Fan out.** One worker per package, each in its own copy of the snapshot, on its own lane
   (`<session>/<package>`), with one more layer: the full text of the files it owns. A patch to a file it doesn't own
   fails, telling it why. A worker ends with `report` (done, or blocked with the reason). Workers run concurrently,
   at most `OTTO_BOT_MAX_WORKERS` at a time. A worker that needs a contract changed reports itself blocked, and the
   orchestrator settles it in the next phase.
4. **Integrate.** Each worker's changed files are copied back (ownership is disjoint, so nothing is merged). The
   orchestrator writes the manager's coordinations and settles blocked packages; then the full checks run, and each
   error goes to the owner of the file it is in, on its still-warm lane, now in the main workspace.
5. **Review.** A reviewer lane whose second layer is the diff, not the project map: read-only, focused on security,
   ending with `report`. High-severity findings go to their owners for one round of fixes; the rest are listed in
   the run's summary.

Workers' copies are copy-on-write clones of the workspace (`cp -c` on APFS, `--reflink=auto` on Linux), build output
included, so each worker's builds start warm. They need no git: the workspace needn't be a repository, and nothing is
committed to its branch. (On a file system without reflinks the build output is copied in full.)

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
            workspace/              reads many files, applies V4A patches, lists the tree; workers' copy-on-write copies
            command_runner/         runs a pack's declared commands
        data_layer/
            data_facade.rs, managers/data_manager.rs
            managers/gateways/
                session_store/      JSONL files: sessions, plans, lane item logs, usage ledger
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
4. The multi-agent workflow, in parts: (1) API probes: `allowed_tools` with `apply_patch`, per-role reasoning effort,
   prewarm, concurrent lanes; (2) lanes and roles in the engine; (3) the plan; (4) the skeleton; (5) fan out;
   (6) integrate; (7) review.
5. The Python pack, then the TypeScript pack, ported by the harness; starting new projects.
6. Resume, compaction, a second client.

A fixed set of feature tasks on template apps is run against every change to the harness, recording wall time,
cost, cache hit rate, lint violations and whether the checks pass, so speed and quality trade-offs are measured.

## Verified

What the design depends on from the API, probed with `gpt-6.1-sol` on a ~6.3k-token prefix (`store=false`):

- **The 12h TTL** works with `store=false`: the stack layer is a cache hit across sessions and workspaces.
- **`tool_choice: {"type": "allowed_tools", "mode": "auto"|"required", "tools": [...]}`** restricts calls to the
  listed tools, native `{"type": "apply_patch"}` included, and keeps the cached prefix: `auto` reads everything up to
  the last message; `required` reads the prompt layers but not the last message.
- **A `configuration_update` item** (`{"type": "configuration_update", "reasoning": {"effort": "low"}}`), appended
  to the input, sets the effort of later responses and keeps the cached prefix (low: 0 reasoning tokens, max: about
  900, on the same question); the response still reports the top-level effort. Changing the top-level
  `reasoning.effort` misses the cache. The model's efforts are `low`, `medium` (the default), `high`, `xhigh`, `max`.
- **Service tiers have separate caches**: the first request on another tier writes its own entry, and the first
  tier's entry is still there when it returns. Every role of a session uses the session's tier.
- **Concurrent requests share one cache write**: six requests (HTTP, or six WebSocket connections) started together
  on a cold prefix: one writes it and five read it. `prompt_cache_options.prewarm: true` writes a prefix with no
  output (about 0.7s), but a fan-out doesn't need it. A request sent right after another's write can still miss.
- **Six WebSocket connections at once** open in 0.4-0.7s each and continue with `previous_response_id`.

Not yet verified: forking a WebSocket lane (not needed while each lane has its own connection).

## Open

- Library choices for the Python and TypeScript templates. Suggested: FastAPI, pydantic, httpx, asyncpg, uv, ruff,
  pyright (strict); Node, Fastify or Hono, zod, `pg`, `tsc` (strict), eslint.
- Whether the `software-engineer-python` skill holds the Python conventions for the architecture.
- Tests in generated apps.
