CREATE TABLE IF NOT EXISTS workspace_clone_origins (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    remote_identity TEXT NOT NULL,
    recorded_unix_ms INTEGER NOT NULL
);

-- `remote_identity` is sanitized before persistence. Raw clone URLs, user-info,
-- query strings, fragments, credentials, and credential-helper output must never
-- be written to this table.
