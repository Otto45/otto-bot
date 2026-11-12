-- migrate:up
-- The steps a session's runs got to, in order: what a run was doing when it stopped is known to the session's next
-- run. A run stopped with its process has no row for its end.
CREATE TABLE run_records (
    run_record_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    status TEXT NOT NULL CHECK (status IN ('started', 'building', 'built', 'completed', 'failed', 'cancelled')),
    recorded_on TIMESTAMPTZ NOT NULL
);

CREATE INDEX run_records_session_id_index ON run_records (session_id, run_record_id);

-- What a session's next review is made against; only its latest row is used. Either everything the session built
-- has been reviewed, or `files` holds the workspace's text files as they were when everything in it had been: a
-- JSON array of {"path", "content"}, kept as text because a file's content is whatever the file holds.
CREATE TABLE review_baselines (
    review_baseline_id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions (session_id),
    reviewed BOOLEAN NOT NULL,
    files TEXT,
    recorded_on TIMESTAMPTZ NOT NULL,
    CONSTRAINT review_baselines_files_unless_reviewed CHECK (reviewed OR files IS NOT NULL)
);

CREATE INDEX review_baselines_session_id_index ON review_baselines (session_id, review_baseline_id);

-- migrate:down
DROP TABLE review_baselines;
DROP TABLE run_records;
