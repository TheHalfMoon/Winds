CREATE TABLE IF NOT EXISTS workspaces (
    workspace_id TEXT PRIMARY KEY,
    canonical_worktree_root TEXT NOT NULL UNIQUE,
    git_common_dir TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL,
    last_opened_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS executions (
    execution_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    kind TEXT NOT NULL,
    request_source TEXT NOT NULL,
    execution_domain TEXT NOT NULL,
    status TEXT NOT NULL,
    status_source TEXT NOT NULL,
    requested_unix_ms INTEGER NOT NULL,
    started_unix_ms INTEGER,
    ended_unix_ms INTEGER,
    duration_ms INTEGER,
    CHECK (duration_ms IS NULL OR duration_ms >= 0),
    CHECK (duration_ms IS NULL OR (started_unix_ms IS NOT NULL AND ended_unix_ms IS NOT NULL)),
    CHECK (started_unix_ms IS NULL OR ended_unix_ms IS NULL OR ended_unix_ms >= started_unix_ms)
);

CREATE TABLE IF NOT EXISTS execution_events (
    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
    execution_id TEXT NOT NULL REFERENCES executions(execution_id),
    kind TEXT NOT NULL,
    fact_source TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS terminal_sessions (
    execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id),
    profile_id TEXT NOT NULL,
    shell_executable TEXT NOT NULL,
    shell_arguments_json TEXT NOT NULL,
    requested_cwd TEXT NOT NULL,
    initial_cols INTEGER CHECK (initial_cols IS NULL OR initial_cols > 0),
    initial_rows INTEGER CHECK (initial_rows IS NULL OR initial_rows > 0),
    close_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_executions_workspace_time
    ON executions(workspace_id, requested_unix_ms, execution_id);

CREATE INDEX IF NOT EXISTS idx_execution_events_execution_time
    ON execution_events(execution_id, created_unix_ms, event_id);
