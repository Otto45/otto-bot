# otto-bot design

otto-bot is a coding agent harness, written in Rust, that builds and changes apps following the Domain Container
architecture, in Rust, Python and TypeScript. It is itself a Domain Container app.

Status: build steps 1-5 are done: the layered cache, WebSocket lanes, the project map, the architecture lint
(steps 1-3), and the three roles (step 4, reworked in step 5): the orchestrator, which holds the conversation with
the person and briefs the implementor; the implementor, which plans and builds each brief from a fresh start; and the
reviewer, whose high-severity findings the implementor fixes. Step 5 is done too: the Python and TypeScript packs,
`otto new`, and the benchmarks (see Benchmarks).

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
| Clients | several, all over one domain facade: the interactive CLI (`otto`), and the HTTP API (`otto-http`) for the software factory platform |
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
- The TypeScript template runs on Node 24 with its types stripped (no build step, so only erasable syntax), imports
  through a `#src/` alias so the lint's import rules see every path, and pins TypeScript 6.0: typescript-eslint doesn't
  support TypeScript 7 yet.
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
| BP3: role | what the agent's role is (the orchestrator's, the implementor's, the reviewer's) | 30m | the lanes of one role |
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
  | `brief_implementor` | the orchestrator's brief to the implementor, checked by the harness (a rejected brief is the call's error) |
  | `report` | the implementor's structured, terse completion report |
  | `submit_review` | the reviewer's findings, graded high, medium or low |

- **Retries.** A request the API failed on its side (a 429 or 5xx, an error event or failed response of type
  `server_error` or a rate limit, a connection that couldn't be made) is sent again, up to `OTTO_BOT_OPENAI_MAX_RETRIES`
  times (3), after a wait doubling from a second or the `Retry-After` the API asks for, whichever transport it went by;
  a refusal sending again won't change (a malformed request, a bad key, an exhausted quota) is not. Before this, one
  `server_error` from the API ended the whole run.
- **Compaction**, explicit (`/responses/compact`, never the server-side `context_management`), at boundaries, so
  the person never waits on it and no request is compacted under them (see Verified):
  - the orchestrator's lane, once a run has ended, if its last request had `OTTO_BOT_ORCHESTRATOR_COMPACT_TOKENS`
    (400k) input tokens or more: the person reads the reply while it compacts (8-25 seconds), and a request sent
    meanwhile starts when it is done;
  - any lane between turns, if its last request had `OTTO_BOT_LANE_COMPACT_TOKENS` (600k; the model takes 922k): a
    safety net for a task long enough to near the limit.

  What comes back is appended to the lane as one `compaction` item, with the role's reasoning effort set again after
  it (the API refuses an effort before a compaction), and the lane is loaded from its last compaction on. Its first
  item, the prompt layers it was sent, must come back unchanged and is left out, since every request opens with them:
  they stay cached. A compaction's usage is in the ledger, and it is reported as `LaneCompacted`.
- **Service tier** comes from the session's options; the ledger records the tier each response actually ran at,
  since `fast` can be downgraded under load.

## Workflow

### Implementing a feature

There are three roles, and the manager keeps the order: each phase is one call to `AgentRunEngine`, which runs one
agent (a lane) in a role.

1. **Orchestrator.** The session's agent holds the conversation with the person: its lane keeps the whole session.
   otto-bot is part of a software factory that builds apps for non-technical people, so the orchestrator speaks in
   plain words about what the app does, never about files, code, databases or checks; it asks only what the app
   should do, and decides the technical rest itself. Its briefs to the implementor are technical.
   It reads the code but changes no file and runs no check. When a request leaves open what only the person can
   decide, it asks, by ending its turn with the question; the answer is the next request, so no run waits on a
   person. When the request is clear, it briefs the implementor (`brief_implementor`): the task, complete enough to
   build from alone; what was decided; what done looks like; and the files to start from. The `BriefProcessor` checks
   the brief without a model; a rejected brief is the tool call's error. Once the brief is built and reviewed, the
   result is told to the orchestrator, and it gets a turn, with only the reading tools, to tell the person in a few
   plain sentences what their app can do now, and anything they need to decide.
2. **Implementor.** Each brief gets a fresh lane (`implementor_<n>`): the implementor starts from the brief alone,
   with the files it names already read into its first message, plans the change, and builds it. It is the only role
   that changes files, in the session's own workspace. A new component follows the template's example of its kind;
   the formatters and the checks catch what wiring it gets wrong. It ends with `report` (done, or blocked with what it
   needs); once it reports done, the harness runs every check itself and sends it what fails, for up to three rounds.
3. **Reviewer.** Once the implementor reports the change done with every check passing, the reviewer, on a fresh
   read-only lane (`reviewer_<n>`) at high reasoning effort, is given the person's last message, the brief and the
   change's diff (from snapshots of the workspace before and after), and reads what else it needs. It judges what the
   code does, not what its comments or the brief say about it, looking for security problems first, then correctness
   against what was asked (money kept exact, writes that belong together in one database never left half done), then
   the architecture's rules, and ends with `submit_review`. It judges the change against the app as it is: a way of
   doing things the app's code already uses is not a finding unless the change makes it worse or the person asked for
   more. High-severity findings go back to the same implementor for one round of fixes (held to the checks again);
   every finding is listed in the run's result.

Until step 5 the orchestrator made a small change itself and planned a bigger one as steps for the implementor. The
step 5 benchmarks showed why that was set aside: on small tasks the orchestrator often made the change and then
submitted a plan anyway, so an implementor started only to find nothing to do; and an implementor that kept one
conversation across plans carried every earlier plan's context. Now the conversation is the orchestrator's, each build
starts fresh, and every change is reviewed.

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
a stack needs (`orders-app`, `orders_app`, `OrdersApp`, `ordersApp`, `ORDERS_APP`, `Orders App`). Then the pack's formatters run,
since a renamed name sorts and wraps differently, so the new app passes its checks before any request. Its examples
stay: they show the implementor every component kind, and the first brief replaces them. Stripping them without a model
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
        otto_bot_manager.rs         coordinates orchestrator -> (brief ->) implementor -> reviewer -> orchestrator
        processors/
            brief/                  checks a brief: it has a task and says what done looks like
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
                session_store/      JSONL files: sessions, briefs, lane item logs, usage ledger
                stack_pack_store/   the stack packs
        common/                     diagnostics parsing (used by the engine and the manager)
        models/, exceptions/, logger/, configuration_providers/, service_locator/
stack_packs/rust/, stack_packs/python/, stack_packs/typescript/
```

- **One engine talks to the model.** An engine never uses another engine, so `agent_run` is the only one, and the
  manager calls it for every role.
- **The facade works for any client.** `submit` runs to completion on its own in a background task. Progress is a
  stream of `SessionEvent`, a domain enum: brief accepted, tool used, brief implemented, usage, done.
  Each service maps the events to its own wire format.
- **Any client can resume any session**, because the session log is the whole record of it.
  `resume_session(id)` serves a kept session again: the orchestrator's lane, the kept project map (so the cached
  prefix carries over) and the brief numbering go on where they were; a WebSocket lane opens a new connection and
  sends the whole conversation once. `list_sessions(workspace)` lists a workspace's sessions, most recently active
  first, from `transcript.jsonl` (what the person and the orchestrator said, and when). The CLI: `otto sessions`,
  `otto --session <id>`.
- **The HTTP API** (`otto-http`) is the platform's client. Apps are addressed by name, never by path: the facade
  keeps them under `OTTO_BOT_WORKSPACES_DIRECTORY`, one folder per kebab-case name, and a session of another app is
  not found. Every request but `/health` bears a shared bearer token. Each session route resumes its session first,
  so restarts and idle eviction (`OTTO_BOT_IDLE_SESSION_MINUTES`) are invisible to the platform.
- **Events are numbered and replayable.** A live session's runs report on a channel that a relay task numbers and
  records in its event log, which keeps the current run's events and fans them out to clients. A subscriber says
  what to replay (nothing, the run from its start, or what came after an id), so the HTTP API's server-sent events
  survive a dropped connection (`Last-Event-ID`) and a stream opened just after a message misses nothing. The
  orchestrator's replies are the events meant for the person (`to_person`); the platform shows those, and an operator
  view can show the rest.
- **One run per app**: two sessions of one workspace never run at once, since both would change the same files.
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
6. Resume (`otto --session`, `otto sessions`); compaction (`/responses/compact` at boundaries); the HTTP API
   (`otto-http`, server-sent events, apps by name).

A fixed set of feature tasks on template apps is run against every change to the harness, recording wall time,
cost, cache hit rate, lint violations and whether the checks pass, so speed and quality trade-offs are measured.

## Benchmarks

The tasks are in `benchmarks/tasks/` (farewell and loyalty points, small; orders, the multi-component one), worded for
any stack. `benchmarks/run.sh <stack> <task>` starts a new app with `otto new`, gives otto the task, then runs the
stack's checks on what it left, independently of otto (`benchmarks/check.sh`), and summarizes the session
(`benchmarks/summarize.py`: wall time, requests and tokens per lane, cache hit rate, briefs, the implementor's and the
reviewer's reports). Results go to `target/benchmarks/`.

Step 5, all nine runs at once (so the times are under load), with the roles as they are now:

| Task | Rust | Python | TypeScript |
| --- | --- | --- | --- |
| farewell | 55s, 9 requests | 35s, 8 requests | 36s, 8 requests |
| loyalty points | 62s, 9 requests | 49s, 9 requests | 40s, 9 requests |
| orders | 260s, 22 requests | 259s, 14 requests | 247s, 22 requests |

Every run passed its checks, every task went through one brief, and the cache hit rate was 80-94%. Each orders app
was then run against Postgres: the discount (none at exactly 100 dollars), the conversion to EUR, and the errors for
an unknown currency, order or customer were all right. On orders the reviewer's high findings were floating-point
money (the currency conversion, and TypeScript's discount), fixed in the one round. With the roles as they were before
(the orchestrator making small changes itself and planning bigger ones), the same runs took 68/43/41s, 88/68/67s and
412/306/284s: the small tasks were planned anyway, and the reviewer graded the app's own pattern of saving and then
recording an audit event in another database as high, so the fix round over-built (an outbox, a blocked report) and
the Python app's fix failed at run time (a datetime bound as text, now a rule of the Python pack).

Step 6 (resume, compaction, the HTTP API), the same nine runs at once:

| Task | Rust | Python | TypeScript |
| --- | --- | --- | --- |
| farewell | 55s, 8 requests | 34s, 8 requests | 40s, 9 requests |
| loyalty points | 63s, 8 requests | 44s, 8 requests | 41s, 8 requests |
| orders | 347s, 21 requests | 261s, 16 requests | 378s, 22 requests |

Every run passed its checks, through one brief, at 79-92% cache hits; no lane came near a compaction threshold. The
small tasks are where they were. The Rust and TypeScript orders runs took longer for more work, not a slower harness:
the packs' gateway rules now ask a gateway to retry what failed on the other side (step 5, part 6), so the currency
gateway grew, and they wrote 25-33% more output (24.5k and 23k tokens, against 18.4k each) in the same number of
requests; one TypeScript review response took 103 seconds. Both reviews sent two high findings back, as before: the
exchange rates parsed as floating point (money again), and, in Rust, the retry backoff copied from the example gateway
rather than shared, since every template keeps it inside its JSON Placeholder gateway.

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

**Compaction** (`POST /responses/compact`, probed 2026-11-10 on an orchestrator-like lane: the three prompt layers,
a fact, two parallel `read_files` calls with their outputs, replies; about 14.9k tokens):

- **It works without `store`**, which it rejects as an unknown parameter (it keeps nothing anyway). It takes `model`,
  `input`, `tools`, `service_tier` (`default`, `fast`, `ultrafast`, `priority`, `flex`; the response doesn't say which
  served it), `include` and `prompt_cache_options`; it returns `output` and `usage`, with no `status`.
- **What comes back is a new conversation**: the developer message with the prompt layers, unchanged (cache
  breakpoints included), every user message and every developer message (harness notes) word for word, then one
  `compaction` item (`encrypted_content`, opaque). Tool calls, their outputs, reasoning and the model's replies are
  folded into the compaction item: 14.9k tokens became 8.0k, of which 6.85k are the layers. Compacting a compacted
  conversation replaces its compaction item with a new one, and keeps the messages again.
- **The prompt layers stay cached**: the first request after a compaction reads the layers from the cache (6,846
  tokens) and writes only the compacted rest. A compaction itself reads nothing the requests cached (it keeps a cache
  of its own) and writes what it was sent, so it is billed as a cold request plus its output.
- **It is slow**: 8-25 seconds, mostly its output (250-1,900 tokens, growing with what it folds in).
- **The model carries on**: it recalled a fact from a user message, a harness note and a tool output it read after an
  earlier compaction; a detail only in a file it had read (a constant's name) was lost, its value kept.
- **`configuration_update` is refused in what is compacted**, and in a request where it comes before the compaction
  item with no update after it. One after the compaction item sets the effort as before (low: 67 reasoning tokens,
  max: 154, on the same question).
- **Both transports continue from it**: HTTP, and WebSocket with the whole compacted conversation and then
  `previous_response_id`. The items keep their ids, which `store=false` doesn't mind.
- **Parallel tool calls compact** (call, call, output, output), unlike the server-side compaction bug reported.

On the orders benchmark the orchestrator's lane holds 60-76KB of file contents per request and 2-3KB of messages and
notes, so a compaction keeps about 3% of what a request adds.

Not yet verified: forking a WebSocket lane (not needed while each lane has its own connection).

## Open

- Tests in generated apps.
