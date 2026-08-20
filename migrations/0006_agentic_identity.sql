CREATE TABLE IF NOT EXISTS workstreams (
    workstream_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    display_name TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL,
    updated_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(workstream_id)) > 0),
    CHECK (length(trim(display_name)) > 0),
    CHECK (created_unix_ms >= 0),
    CHECK (updated_unix_ms >= created_unix_ms)
);

CREATE TABLE IF NOT EXISTS winds_sessions (
    session_id TEXT PRIMARY KEY,
    workstream_id TEXT NOT NULL REFERENCES workstreams(workstream_id),
    display_name TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL,
    updated_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(session_id)) > 0),
    CHECK (length(trim(display_name)) > 0),
    CHECK (created_unix_ms >= 0),
    CHECK (updated_unix_ms >= created_unix_ms)
);

-- A Winds session deliberately does not duplicate workspace_id. Workspace ownership is
-- structurally derived through winds_sessions.workstream_id -> workstreams.workspace_id.
CREATE INDEX IF NOT EXISTS idx_workstreams_workspace_created
    ON workstreams(workspace_id, created_unix_ms, workstream_id);

CREATE INDEX IF NOT EXISTS idx_winds_sessions_workstream_created
    ON winds_sessions(workstream_id, created_unix_ms, session_id);
