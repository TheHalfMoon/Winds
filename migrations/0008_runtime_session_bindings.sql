-- T073 persists only runtime/native binding provenance for an existing canonical Winds session.
-- It deliberately does not duplicate workspace/workstream identity, model/provider identity,
-- process/PID identity, or a persisted LIVE claim. Live ownership must be proven in-memory by
-- a later authorized runtime task; durable native IDs remain resume candidates only.
CREATE TABLE IF NOT EXISTS runtime_session_bindings (
    binding_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES winds_sessions(session_id),
    runtime_kind TEXT NOT NULL,
    observed_executable_path TEXT NOT NULL,
    canonical_executable_path TEXT NOT NULL,
    executable_byte_len INTEGER NOT NULL,
    executable_sha256 TEXT NOT NULL,
    runtime_version_state TEXT NOT NULL,
    runtime_version TEXT NOT NULL,
    runtime_version_source TEXT NOT NULL,
    native_session_id TEXT,
    ownership_state TEXT NOT NULL DEFAULT 'UNPROVEN',
    bound_unix_ms INTEGER NOT NULL,
    ownership_observed_unix_ms INTEGER,
    CHECK (length(trim(binding_id)) > 0),
    CHECK (length(trim(session_id)) > 0),
    CHECK (runtime_kind IN ('CODEX', 'CLAUDE')),
    CHECK (length(observed_executable_path) > 0),
    CHECK (length(canonical_executable_path) > 0),
    CHECK (executable_byte_len >= 0),
    CHECK (length(executable_sha256) = 64),
    CHECK (runtime_version_state = 'OBSERVED'),
    CHECK (length(runtime_version) > 0),
    CHECK (runtime_version_source = 'WINDS_LOCALLY_OBSERVED'),
    CHECK (native_session_id IS NULL OR length(trim(native_session_id)) > 0),
    CHECK (ownership_state IN ('UNPROVEN', 'OWNERSHIP_LOST')),
    CHECK (bound_unix_ms >= 0),
    CHECK (
        (ownership_state = 'UNPROVEN' AND ownership_observed_unix_ms IS NULL)
        OR
        (ownership_state = 'OWNERSHIP_LOST'
            AND ownership_observed_unix_ms IS NOT NULL
            AND ownership_observed_unix_ms >= bound_unix_ms)
    )
);

CREATE INDEX IF NOT EXISTS idx_runtime_session_bindings_session_runtime
    ON runtime_session_bindings(session_id, runtime_kind, bound_unix_ms, binding_id);

-- One exact concrete runtime/native identity cannot silently alias multiple Winds sessions.
-- Native IDs may still be reused after executable/version identity changes; those cases remain
-- distinct bindings and must be revalidated before any future resume attempt.
CREATE UNIQUE INDEX IF NOT EXISTS idx_runtime_session_bindings_exact_native
    ON runtime_session_bindings(
        runtime_kind,
        canonical_executable_path,
        executable_sha256,
        runtime_version,
        native_session_id
    )
    WHERE native_session_id IS NOT NULL;
