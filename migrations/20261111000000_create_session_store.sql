-- migrate:up
-- The session store, when otto-bot is given a Postgres database (OTTO_BOT_RELATIONAL_DATABASE_CONNECTION_STRING):
-- one table per file of a session's folder. Rows are only ever inserted, never updated, and each table but sessions
-- is ordered by its identity column. The schema refuses what the domain can't hold: a required column is NOT NULL,
-- a column holding one of a fixed set of names lists them, and a conversation item has what its type requires. A
-- name added to one of those sets (a service tier, a reasoning effort, a role) needs a migration changing its CHECK.

CREATE TABLE sessions (
    session_id TEXT PRIMARY KEY,
    workspace_directory TEXT NOT NULL CHECK (workspace_directory <> ''),
    service_tier TEXT NOT NULL CHECK (service_tier IN ('standard', 'fast', 'ultrafast')),
    started_on TIMESTAMPTZ NOT NULL
);

-- What the person and the orchestrator said.
CREATE TABLE transcript_entries (
    transcript_entry_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    speaker TEXT NOT NULL CHECK (speaker IN ('person', 'assistant')),
    text TEXT NOT NULL,
    said_on TIMESTAMPTZ NOT NULL
);

CREATE INDEX transcript_entries_session_id_index ON transcript_entries (session_id, transcript_entry_id);

-- Each lane's conversation, every item in the order the model saw it. A lane is read from its last compaction on.
CREATE TABLE conversation_items (
    conversation_item_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    lane_name TEXT NOT NULL CHECK (lane_name <> ''),
    item_type TEXT NOT NULL CHECK (
        item_type IN (
            'user_message',
            'harness_note',
            'configuration_update',
            'model_output',
            'function_call_output',
            'apply_patch_call_output',
            'compaction'
        )
    ),
    text TEXT,
    item TEXT,
    call_id TEXT,
    call_kind TEXT CHECK (call_kind IN ('function', 'apply_patch')),
    output TEXT,
    succeeded BOOLEAN,
    reasoning_effort TEXT CHECK (reasoning_effort IN ('low', 'medium', 'high', 'xhigh', 'max')),
    items TEXT[],
    CONSTRAINT conversation_items_fields_of_type CHECK (
        CASE item_type
            WHEN 'user_message' THEN text IS NOT NULL
            WHEN 'harness_note' THEN text IS NOT NULL
            WHEN 'configuration_update' THEN reasoning_effort IS NOT NULL
            -- A tool call is answered by its call_id.
            WHEN 'model_output' THEN item IS NOT NULL AND (call_kind IS NULL OR call_id IS NOT NULL)
            WHEN 'function_call_output' THEN call_id IS NOT NULL AND output IS NOT NULL
            WHEN 'apply_patch_call_output' THEN call_id IS NOT NULL AND succeeded IS NOT NULL AND output IS NOT NULL
            WHEN 'compaction' THEN items IS NOT NULL AND cardinality(items) >= 1
            ELSE FALSE
        END
    )
);

CREATE INDEX conversation_items_lane_index ON conversation_items (session_id, lane_name, conversation_item_id);

-- The usage ledger: one row per model response.
CREATE TABLE usage_records (
    usage_record_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    lane_name TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('orchestrator', 'implementor', 'reviewer')),
    response_id TEXT NOT NULL,
    model TEXT NOT NULL,
    requested_service_tier TEXT NOT NULL CHECK (requested_service_tier IN ('standard', 'fast', 'ultrafast')),
    -- None when the API didn't say which tier served the response.
    served_service_tier TEXT CHECK (served_service_tier IN ('standard', 'fast', 'ultrafast')),
    input_tokens BIGINT NOT NULL,
    cached_input_tokens BIGINT NOT NULL,
    cache_write_tokens BIGINT NOT NULL,
    output_tokens BIGINT NOT NULL,
    reasoning_tokens BIGINT NOT NULL,
    recorded_on TIMESTAMPTZ NOT NULL
);

CREATE INDEX usage_records_session_id_index ON usage_records (session_id, usage_record_id);

-- The project maps a session's runs started with: the latest 'layer' one is what its requests open with.
CREATE TABLE project_maps (
    project_map_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    project_map TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('layer', 'seen')),
    built_on TIMESTAMPTZ NOT NULL
);

CREATE INDEX project_maps_session_id_index ON project_maps (session_id, project_map_id);

-- The briefs the harness accepted, in the order it accepted them; their count numbers the next.
CREATE TABLE implementation_briefs (
    implementation_brief_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    task TEXT NOT NULL,
    decisions TEXT[] NOT NULL,
    done_when TEXT NOT NULL,
    files TEXT[] NOT NULL,
    accepted_on TIMESTAMPTZ NOT NULL
);

CREATE INDEX implementation_briefs_session_id_index ON implementation_briefs (session_id, implementation_brief_id);

-- migrate:down
DROP TABLE implementation_briefs;
DROP TABLE project_maps;
DROP TABLE usage_records;
DROP TABLE conversation_items;
DROP TABLE transcript_entries;
DROP TABLE sessions;
