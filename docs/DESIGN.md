# otto-bot design

otto-bot is a coding agent harness, written in Rust, that builds and changes apps following the Domain Container
architecture, in Rust, Python and TypeScript. It is itself a Domain Container app.

Status: build steps 1-4 are done: the layered cache, WebSocket lanes, the project map, the architecture lint
(steps 1-3), and the three roles (step 4): the orchestrator, which makes a small change itself and plans a
bigger one; the implementor, which builds the plan; and the reviewer, whose high-severity findings the implementor
fixes. The Python and TypeScript packs (step 5) are next.

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
- **compute the division of work**: component boundaries and dependency rules split a feature into components with
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
    pack.toml       detection (e.g. Cargo.toml), check commands, formatters, the project map and lint queries,
                    rename map
    rules.md        the stack's architecture rules (the template's AGENTS.md)
    template/       the starting app: one example of every component kind
    queries/        tree-sitter queries that extract public signatures, imports and component declarations
```

| | Rust | Python | TypeScript |
| --- | --- | --- | --- |
| Template | the existing Rust template | written from the Rust template | written from the Rust template |
| Libraries | axum, utoipa, sqlx, reqwest, serde | FastAPI, pydantic, asyncpg, httpx, uv | Node 24, Hono with `@hono/zod-openapi`, zod, `pg`, pnpm |
| Module wiring | `mod.rs` | `__init__.py` | `index.ts` barrels, or none |
| Checks | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `export-openapi --check` | `ruff format --check`, `ruff check`, `pyright` (strict), contract check | prettier, eslint (typescript-eslint strict type-checked), `tsc --noEmit` (strict), contract check |

- Without tests, a strict type checker is part of the Python and TypeScript packs' rules.
- The Python and TypeScript templates are written from the Rust template (not ported by the harness), each mapping
  the architecture onto its language's own inheritance: Protocols, interfaces and base classes where Rust has traits
  and enums (e.g. an exception family is a base class, not an enum). Every template's relational example is Postgres
  with dbmate migrations.
- A pack's rules are its template's `AGENTS.md`; the Python one takes its Python conventions from the
  `software-engineer-python` skill (`@final`, explicit `Protocol` inheritance, `slots` and `frozen` dataclasses).

## Prompt and cache design

Every request, for every agent, is built in the same layers, from most shared to least shared, each ending in an
explicit cache breakpoint (`prompt_cache_options.mode: "explicit"`, `prompt_cache_breakpoint` on an `input_text`
block of a developer message):

| Breakpoint | Content | TTL | Shared by |
| --- | --- | --- | --- |
| BP1: stack prefix | the fixed tool set, harness instructions, `rules.md`, the pack's template code | 12h | every agent and project of the stack |
| BP2: project map | tree; signatures of the facade, service locator and DataFacade; models; exception kinds; settings | 12h / 30m | every agent in the session |
| BP3: role | what the agent's role is (the implementor's, the reviewer's); the orchestrator has none | 30m | one agent |
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
- **The implementor shares the orchestrator's prefix.** Its requests open with the orchestrator's kept project map,
  already cached, and a note of what changed since it was built, rather than a map of its own.
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
  | `run_checks` | runs the pack's check commands |
  | `submit_plan` | the orchestrator's work plan, checked by the harness (a rejected plan is the call's error) |
  | `report` | the implementor's structured, terse completion report |
  | `submit_review` | the reviewer's findings, graded high, medium or low |

- **Compaction** (`/responses/compact`, or `context_management`) only at wave boundaries, since it resets cache reuse.
- **Service tier** comes from the session's options; the ledger records the tier each response actually ran at,
  since `fast` can be downgraded under load.

## Workflow

### Implementing a feature

There are three roles, and the manager keeps the order: each phase is one call to `AgentRunEngine`, which runs one
agent (a lane) in a role.

1. **Orchestrator.** The session's agent, with one more tool than steps 1-3 gave it: `submit_plan`. A small change it
   makes itself, so a small request pays nothing for the workflow. A bigger one it plans, before it changes any file:
   a summary, the assumptions it made where the request left things open, and the steps that build the change. The
   `WorkPlanProcessor` checks the plan without a model; a rejected plan is the tool call's error, so the orchestrator
   fixes it in the same turn loop.
2. **Implementor.** On a lane of its own, the implementor builds the change from the plan: the only role that
   changes files under a plan, in the session's own workspace. A new component follows the template's example of its
   kind; the formatters and the checks catch what wiring it gets wrong. It ends
   with `report` (done, or blocked with what it needs); once it reports done, the harness runs every check itself and
   sends it what fails, for up to three rounds.
3. **Reviewer.** Once the implementor reports the change done with every check passing, the reviewer, on a read-only
   lane at high reasoning effort, is given the person's request, the plan and the change's diff (from snapshots of the
   workspace before and after), and reads what else it needs. It judges what the code does, not what its comments or
   the plan say about it, looking for security problems first, then correctness against the request (money kept
   exact, no data left inconsistent), then the architecture's rules, and ends with `submit_review`. High-severity findings go back to the implementor for one round
   of fixes (held to the checks again); every finding is listed in the run's result. The orchestrator's own small
   changes aren't reviewed.

Scaffolding (the harness creating a planned component and its wiring from the pack's templates, without a model) was
built in step 3 and removed after measuring it: the implementor never called it, it wrote about a tenth of a plan's
lines, its per-component formatting cost 16-18 seconds a plan, and each pack would have had to keep templates and anchor
lines in step with its template. The model wires components correctly on its own, and the checks catch what it doesn't.

One implementor keeps the workflow simple: no file ownership, no copies of the workspace, nothing to merge. Building a
plan's parts in parallel (an implementor per part, each in a copy-on-write copy of the workspace, with ownership or a
merge to keep them apart) was built and measured (step 4, part 5) and set aside for now: the parts were cut along file
boundaries the plan couldn't foresee, and most of them blocked on a file another part owned.

### Starting a project

`otto new <name> --stack <stack>` (the `ProjectBootstrapEngine`) copies the pack's template into a new or empty folder
named after the app, leaving out what the template's `.gitignore` ignores, and renames it without a model: the pack's
`[rename]` map, applied to file contents and paths in one pass, the longest name first. The app's name gives every form
a stack needs (`orders-app`, `orders_app`, `OrdersApp`, `ORDERS_APP`, `Orders App`). Then the pack's formatters run,
since a renamed name sorts and wraps differently, so the new app passes its checks before any request. Its examples
stay: they show the implementor every component kind, and the first plan replaces them. Stripping them without a model
would need a second, empty template per pack, kept in step by hand.

## Speed levers

In order of expected impact:

1. Generate instead of infer (formatting, contract exports, renames).
2. Pre-load context, so no turn is spent exploring.
3. Every role sharing the cached BP1 and BP2.
4. Small output: diffs, `text.verbosity: low`, structured reports, reasoning effort matched to the role.
5. The harness formats code and regenerates generated files itself, before every run of the checks, and holds the
   implementor to the checks passing.
6. WebSocket transport, and `fast` / `ultrafast` when speed is worth the price.
7. Warm incremental builds.

## Security

The sandbox VM the harness runs in is the security boundary: its isolation and outbound network policy contain what
an agent does. Inside it, the harness keeps what protects the work and what keeps secrets from leaving:

- **Workspace confinement and roles**: paths resolve to their real location, symlinks can't escape, and each role
  may call only its own tools (the reviewer changes nothing).
- **Declared commands only**: agents run the pack's check commands, not a general shell.
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
        otto_bot_manager.rs         coordinates orchestrator -> (plan ->) implementor -> reviewer
        processors/
            work_plan/              checks a plan: it has a summary and steps
            prompt_assembly/        builds the layered prompt, renders it deterministically, places breakpoints
            architecture_map/       the project map, from tree-sitter parses
            architecture_lint/      deterministic rule checks
        engines/
            agent_run/              runs one agent to completion: model turn -> tool calls -> results; applies
                                    patches, holds each role to its tools
            project_bootstrap/      copies and renames a pack's template (no model)
        gateways/
            openai_responses/       WebSocket and HTTP transport, lanes, replay from the session log
            workspace/              reads many files, applies V4A patches, lists the tree
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
  stream of `SessionEvent`, a domain enum: plan accepted, tool used, plan implemented, usage, done.
  Each service maps the events to its own wire format.
- **Any client can resume any session**, because the session log is the whole record of it.
- **Session options** (such as the service tier) are passed to `start_session` and override configured defaults,
  which only the configuration provider reads from the environment.

## Build order

1. The harness skeleton from the Rust template; the Rust pack; the facade's session and event API; the CLI service.
2. A single-agent loop over HTTP, replaying the item list from the session log; the usage ledger; `--service-tier`.
3. WebSocket lanes with replay; layered breakpoints; the project map; `scaffold` (since removed); the architecture lint.
4. The multi-agent workflow, in parts: (1) API probes: `allowed_tools` with `apply_patch`, per-role reasoning effort,
   prewarm, concurrent lanes; (2) lanes and roles in the engine; (3) the plan; (4) the skeleton; (5) fan out (set
   aside: the roles are now the orchestrator, one implementor and the reviewer, and the implementor builds the plan
   with no skeleton step); (6) review.
5. The Python pack, then the TypeScript pack, their templates written from the Rust template; starting new projects.
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

- Tests in generated apps.
