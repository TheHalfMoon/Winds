CREATE TABLE IF NOT EXISTS shell_commands (
    execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id),
    executable TEXT NOT NULL,
    arguments_json TEXT NOT NULL,
    command_source TEXT NOT NULL,
    requested_cwd TEXT NOT NULL,
    cwd_source TEXT NOT NULL,
    exit_code INTEGER,
    exit_source TEXT,
    CHECK (exit_code IS NULL OR exit_source IS NOT NULL)
);

