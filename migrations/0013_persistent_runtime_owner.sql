-- Spec 011 T147: durable runtime namespace metadata only.
-- Persisted rows explain restart truth; they never prove a live owner or child process.

CREATE TABLE IF NOT EXISTS persistent_runtime_owner_generations (
    owner_generation_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    started_unix_ms INTEGER NOT NULL CHECK (started_unix_ms >= 0),
    CHECK (
        length(owner_generation_id) = 32
        AND owner_generation_id NOT GLOB '*[^0-9a-f]*'
        AND owner_generation_id <> '00000000000000000000000000000000'
    )
);

CREATE TABLE IF NOT EXISTS persistent_runtime_namespaces (
    runtime_namespace_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    runtime_alias TEXT NOT NULL,
    workspace_id TEXT REFERENCES workspaces(workspace_id) ON DELETE SET NULL,
    session_id TEXT REFERENCES winds_sessions(session_id) ON DELETE SET NULL,
    terminal_execution_id TEXT REFERENCES terminal_sessions(execution_id) ON DELETE SET NULL,
    owner_generation_id TEXT REFERENCES persistent_runtime_owner_generations(owner_generation_id),
    ownership_state TEXT NOT NULL CHECK (
        ownership_state IN ('LIVE_OWNED', 'OWNERSHIP_LOST', 'UNOWNED')
    ),
    process_liveness TEXT NOT NULL CHECK (
        process_liveness IN ('UNKNOWN', 'RUNNING', 'EXITED', 'UNAVAILABLE')
    ),
    endpoint_availability TEXT NOT NULL CHECK (
        endpoint_availability IN ('UNKNOWN', 'AVAILABLE', 'UNAVAILABLE')
    ),
    continuity_class TEXT NOT NULL CHECK (
        continuity_class IN (
            'RETAINED_LIVE_PROCESS',
            'PROVIDER_NATIVE_RESUME',
            'WINDS_RECONSTRUCTION',
            'FRESH_PROCESS',
            'UNKNOWN',
            'UNAVAILABLE'
        )
    ),
    last_lifecycle_event_kind TEXT NOT NULL CHECK (
        last_lifecycle_event_kind IN (
            'NAMESPACE_CREATED',
            'OWNERSHIP_ESTABLISHED',
            'OWNERSHIP_LOST',
            'PROCESS_STATE_OBSERVED',
            'CONTINUITY_CLASSIFIED',
            'CONTROLLER_CHANGED',
            'RUNTIME_STOPPED'
        )
    ),
    created_unix_ms INTEGER NOT NULL CHECK (created_unix_ms >= 0),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= created_unix_ms),
    last_observed_unix_ms INTEGER CHECK (
        last_observed_unix_ms IS NULL OR last_observed_unix_ms >= created_unix_ms
    ),
    ownership_lost_unix_ms INTEGER CHECK (
        ownership_lost_unix_ms IS NULL OR ownership_lost_unix_ms >= created_unix_ms
    ),
    recovery_reason TEXT CHECK (
        recovery_reason IS NULL OR recovery_reason IN (
            'OWNER_GENERATION_CHANGED',
            'OWNER_GENERATION_UNPROVEN'
        )
    ),
    CHECK (
        length(runtime_namespace_id) = 32
        AND runtime_namespace_id NOT GLOB '*[^0-9a-f]*'
        AND runtime_namespace_id <> '00000000000000000000000000000000'
    ),
    CHECK (length(trim(runtime_alias)) BETWEEN 1 AND 256),
    CHECK (instr(runtime_alias, char(0)) = 0),
    CHECK (workspace_id IS NULL OR length(trim(workspace_id)) > 0),
    CHECK (session_id IS NULL OR length(trim(session_id)) > 0),
    CHECK (terminal_execution_id IS NULL OR length(trim(terminal_execution_id)) > 0),
    CHECK (ownership_state <> 'LIVE_OWNED' OR owner_generation_id IS NOT NULL),
    CHECK (
        (ownership_state = 'OWNERSHIP_LOST'
            AND ownership_lost_unix_ms IS NOT NULL
            AND recovery_reason IS NOT NULL)
        OR
        (ownership_state <> 'OWNERSHIP_LOST'
            AND ownership_lost_unix_ms IS NULL
            AND recovery_reason IS NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_persistent_runtime_generation
    ON persistent_runtime_namespaces(owner_generation_id, runtime_namespace_id);

CREATE TRIGGER IF NOT EXISTS trg_persistent_runtime_reject_time_regression
BEFORE UPDATE ON persistent_runtime_namespaces
WHEN NEW.updated_unix_ms < OLD.updated_unix_ms
BEGIN
    SELECT RAISE(ABORT, 'persistent runtime update time regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_persistent_runtime_reject_observation_regression
BEFORE UPDATE ON persistent_runtime_namespaces
WHEN OLD.last_observed_unix_ms IS NOT NULL
 AND (NEW.last_observed_unix_ms IS NULL
      OR NEW.last_observed_unix_ms < OLD.last_observed_unix_ms)
BEGIN
    SELECT RAISE(ABORT, 'persistent runtime observation time regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_persistent_runtime_insert_scope
BEFORE INSERT ON persistent_runtime_namespaces
WHEN
    (NEW.workspace_id IS NOT NULL AND NEW.session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM winds_sessions s
        JOIN workstreams w ON w.workstream_id = s.workstream_id
        WHERE s.session_id = NEW.session_id AND w.workspace_id = NEW.workspace_id
    ))
    OR
    (NEW.workspace_id IS NOT NULL AND NEW.terminal_execution_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM executions e
        WHERE e.execution_id = NEW.terminal_execution_id
          AND e.workspace_id = NEW.workspace_id
    ))
    OR
    (NEW.session_id IS NOT NULL AND NEW.terminal_execution_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM winds_sessions s
        JOIN workstreams w ON w.workstream_id = s.workstream_id
        JOIN executions e ON e.execution_id = NEW.terminal_execution_id
        WHERE s.session_id = NEW.session_id
          AND w.workspace_id = e.workspace_id
    ))
BEGIN
    SELECT RAISE(ABORT, 'persistent runtime canonical reference scope mismatch');
END;

CREATE TRIGGER IF NOT EXISTS trg_persistent_runtime_update_scope
BEFORE UPDATE OF workspace_id, session_id, terminal_execution_id
ON persistent_runtime_namespaces
WHEN
    (NEW.workspace_id IS NOT NULL AND NEW.session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM winds_sessions s
        JOIN workstreams w ON w.workstream_id = s.workstream_id
        WHERE s.session_id = NEW.session_id AND w.workspace_id = NEW.workspace_id
    ))
    OR
    (NEW.workspace_id IS NOT NULL AND NEW.terminal_execution_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM executions e
        WHERE e.execution_id = NEW.terminal_execution_id
          AND e.workspace_id = NEW.workspace_id
    ))
    OR
    (NEW.session_id IS NOT NULL AND NEW.terminal_execution_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM winds_sessions s
        JOIN workstreams w ON w.workstream_id = s.workstream_id
        JOIN executions e ON e.execution_id = NEW.terminal_execution_id
        WHERE s.session_id = NEW.session_id
          AND w.workspace_id = e.workspace_id
    ))
BEGIN
    SELECT RAISE(ABORT, 'persistent runtime canonical reference scope mismatch');
END;
