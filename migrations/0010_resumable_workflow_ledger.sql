-- Spec 008 T102: one-process durable workflow/decision schema in the existing winds.db.
-- Later Spec 008 tasks activate baseline, actor, reconstruction, retry, and decision behavior;
-- their tables are created now so the Plan-selected schema is qualified once.
CREATE TABLE IF NOT EXISTS workflow_runs (
    workflow_run_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    workstream_id TEXT NOT NULL REFERENCES workstreams(workstream_id),
    schema_version INTEGER NOT NULL DEFAULT 1,
    terminal_state TEXT,
    created_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(workflow_run_id)) > 0),
    CHECK (length(trim(workspace_id)) > 0),
    CHECK (length(trim(workstream_id)) > 0),
    CHECK (schema_version = 1),
    CHECK (terminal_state IS NULL OR terminal_state IN ('COMPLETED', 'CANCELLED', 'RECOVERY_REQUIRED')),
    CHECK (created_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS workflow_stage_runs (
    stage_run_id TEXT PRIMARY KEY,
    workflow_run_id TEXT NOT NULL REFERENCES workflow_runs(workflow_run_id),
    stage_key TEXT NOT NULL,
    attempt_ordinal INTEGER NOT NULL,
    predecessor_stage_run_id TEXT REFERENCES workflow_stage_runs(stage_run_id),
    relation_kind TEXT,
    lifecycle_state TEXT NOT NULL,
    failure_class TEXT,
    checkpoint_identity TEXT,
    material_progress_basis TEXT,
    outcome_reason TEXT,
    last_transition_operation_id TEXT,
    last_transition_from_state TEXT,
    last_transition_source TEXT,
    last_transition_authority TEXT,
    created_unix_ms INTEGER NOT NULL,
    updated_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(stage_run_id)) > 0),
    CHECK (length(trim(stage_key)) > 0),
    CHECK (attempt_ordinal BETWEEN 1 AND 4294967295),
    CHECK (relation_kind IS NULL OR relation_kind IN ('RETRY_OF', 'RECONSTRUCTION_OF', 'REASSIGNMENT_OF', 'RECOVERY_OF')),
    CHECK (lifecycle_state IN ('PREPARED', 'ACTIVE', 'WAITING_APPROVAL', 'WAITING_EXTERNAL', 'BLOCKED', 'FAILED', 'STALE', 'CANCELLED', 'COMPLETED', 'RECOVERY_REQUIRED')),
    CHECK (last_transition_from_state IS NULL OR last_transition_from_state IN ('PREPARED', 'ACTIVE', 'WAITING_APPROVAL', 'WAITING_EXTERNAL', 'BLOCKED', 'FAILED', 'STALE', 'CANCELLED', 'COMPLETED', 'RECOVERY_REQUIRED')),
    CHECK (last_transition_source IS NULL OR last_transition_source IN ('AGENT_REPORTED', 'WINDS_OBSERVED', 'HUMAN_DECIDED')),
    CHECK (last_transition_authority IS NULL OR last_transition_authority IN ('NONE', 'WINDS_POLICY', 'HUMAN_DECISION')),
    CHECK ((predecessor_stage_run_id IS NULL AND relation_kind IS NULL AND attempt_ordinal = 1) OR (predecessor_stage_run_id IS NOT NULL AND relation_kind IS NOT NULL AND attempt_ordinal > 1)),
    CHECK (
        (lifecycle_state = 'PREPARED' AND last_transition_operation_id IS NULL AND last_transition_from_state IS NULL AND last_transition_source IS NULL AND last_transition_authority IS NULL)
        OR
        (lifecycle_state <> 'PREPARED' AND length(trim(last_transition_operation_id)) > 0 AND last_transition_from_state IS NOT NULL AND last_transition_source IS NOT NULL AND last_transition_authority IS NOT NULL)
    ),
    CHECK (created_unix_ms >= 0),
    CHECK (updated_unix_ms >= created_unix_ms),
    UNIQUE (workflow_run_id, stage_key, attempt_ordinal)
);

CREATE TABLE IF NOT EXISTS workflow_artifact_baselines (
    baseline_id TEXT PRIMARY KEY,
    stage_run_id TEXT NOT NULL REFERENCES workflow_stage_runs(stage_run_id),
    baseline_kind TEXT NOT NULL,
    stable_reference TEXT NOT NULL,
    candidate_oid TEXT,
    candidate_tree TEXT,
    created_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(baseline_id)) > 0),
    CHECK (baseline_kind IN ('EXACT_GIT_CANDIDATE', 'WINDS_VERIFICATION_EVIDENCE', 'PRIOR_STAGE_OUTPUT', 'CANONICAL_DECISION', 'BOUNDED_BLOB_ARTIFACT')),
    CHECK (length(trim(stable_reference)) > 0),
    CHECK ((candidate_oid IS NULL AND candidate_tree IS NULL) OR (candidate_oid IS NOT NULL AND candidate_tree IS NOT NULL)),
    CHECK (candidate_oid IS NULL OR length(candidate_oid) IN (40, 64)),
    CHECK (candidate_tree IS NULL OR length(candidate_tree) IN (40, 64)),
    CHECK (created_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS workflow_actor_bindings (
    binding_id TEXT PRIMARY KEY,
    stage_run_id TEXT NOT NULL REFERENCES workflow_stage_runs(stage_run_id),
    winds_session_id TEXT REFERENCES winds_sessions(session_id),
    runtime_binding_id TEXT REFERENCES runtime_session_bindings(binding_id),
    continuation_class TEXT NOT NULL,
    bound_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(binding_id)) > 0),
    CHECK (continuation_class IN ('RESUMED', 'RECONSTRUCTED', 'OWNERSHIP_LOST', 'UNAVAILABLE', 'UNPROVEN')),
    CHECK (bound_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS workflow_reconstruction_reports (
    reconstruction_report_id TEXT PRIMARY KEY,
    binding_id TEXT NOT NULL UNIQUE REFERENCES workflow_actor_bindings(binding_id),
    schema_version INTEGER NOT NULL DEFAULT 1,
    canonical_report_json TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(reconstruction_report_id)) > 0),
    CHECK (schema_version = 1),
    CHECK (length(trim(canonical_report_json)) > 0),
    CHECK (created_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS workflow_decisions (
    decision_id TEXT PRIMARY KEY,
    workflow_run_id TEXT NOT NULL REFERENCES workflow_runs(workflow_run_id),
    stage_run_id TEXT REFERENCES workflow_stage_runs(stage_run_id),
    source_class TEXT NOT NULL,
    authority_class TEXT NOT NULL,
    decision_type TEXT NOT NULL,
    decision_result TEXT NOT NULL,
    predecessor_decision_id TEXT REFERENCES workflow_decisions(decision_id),
    candidate_oid TEXT,
    candidate_tree TEXT,
    evidence_reference TEXT,
    content_state TEXT NOT NULL,
    safe_rationale TEXT,
    created_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(decision_id)) > 0),
    CHECK (source_class IN ('AGENT_REPORTED', 'WINDS_OBSERVED', 'HUMAN_DECIDED')),
    CHECK (authority_class IN ('NONE', 'WINDS_POLICY', 'HUMAN_DECISION')),
    CHECK (length(trim(decision_type)) > 0),
    CHECK (length(trim(decision_result)) > 0),
    CHECK ((candidate_oid IS NULL AND candidate_tree IS NULL) OR (candidate_oid IS NOT NULL AND candidate_tree IS NOT NULL)),
    CHECK (candidate_oid IS NULL OR length(candidate_oid) IN (40, 64)),
    CHECK (candidate_tree IS NULL OR length(candidate_tree) IN (40, 64)),
    CHECK (content_state IN ('FULL', 'REDACTED', 'OMITTED', 'UNAVAILABLE')),
    CHECK (created_unix_ms >= 0)
);

CREATE INDEX IF NOT EXISTS idx_workflow_runs_workstream_created
    ON workflow_runs(workstream_id, created_unix_ms, workflow_run_id);
CREATE INDEX IF NOT EXISTS idx_workflow_stage_runs_workflow_stage
    ON workflow_stage_runs(workflow_run_id, stage_key, attempt_ordinal, stage_run_id);
CREATE INDEX IF NOT EXISTS idx_workflow_stage_runs_workflow_state
    ON workflow_stage_runs(workflow_run_id, lifecycle_state, updated_unix_ms, stage_run_id);
CREATE INDEX IF NOT EXISTS idx_workflow_artifact_baselines_stage
    ON workflow_artifact_baselines(stage_run_id, created_unix_ms, baseline_id);
CREATE INDEX IF NOT EXISTS idx_workflow_actor_bindings_stage
    ON workflow_actor_bindings(stage_run_id, bound_unix_ms, binding_id);
CREATE INDEX IF NOT EXISTS idx_workflow_decisions_workflow_time
    ON workflow_decisions(workflow_run_id, created_unix_ms, decision_id);
CREATE INDEX IF NOT EXISTS idx_workflow_decisions_stage_time
    ON workflow_decisions(stage_run_id, created_unix_ms, decision_id);

-- Duplicated workflow workspace/workstream columns must describe one canonical Winds hierarchy.
CREATE TRIGGER IF NOT EXISTS trg_workflow_runs_hierarchy_insert
BEFORE INSERT ON workflow_runs
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1 FROM workstreams
    WHERE workstream_id = NEW.workstream_id AND workspace_id = NEW.workspace_id
)
BEGIN
    SELECT RAISE(ABORT, 'workflow identity does not match canonical Winds hierarchy');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_runs_identity_update
BEFORE UPDATE OF workspace_id, workstream_id, workflow_run_id, schema_version ON workflow_runs
BEGIN
    SELECT RAISE(ABORT, 'workflow canonical identity is immutable');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_runs_no_delete
BEFORE DELETE ON workflow_runs
BEGIN
    SELECT RAISE(ABORT, 'workflow history is immutable');
END;

-- A successor attempt must point to the exact previous attempt in the same workflow/logical stage.
CREATE TRIGGER IF NOT EXISTS trg_workflow_stage_runs_lineage_insert
BEFORE INSERT ON workflow_stage_runs
FOR EACH ROW
WHEN NEW.predecessor_stage_run_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM workflow_stage_runs predecessor
    WHERE predecessor.stage_run_id = NEW.predecessor_stage_run_id
      AND predecessor.workflow_run_id = NEW.workflow_run_id
      AND predecessor.stage_key = NEW.stage_key
      AND predecessor.attempt_ordinal + 1 = NEW.attempt_ordinal
)
BEGIN
    SELECT RAISE(ABORT, 'stage predecessor does not match canonical workflow/stage lineage');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_stage_runs_initial_state_insert
BEFORE INSERT ON workflow_stage_runs
FOR EACH ROW
WHEN NEW.lifecycle_state <> 'PREPARED'
  OR NEW.last_transition_operation_id IS NOT NULL
  OR NEW.last_transition_from_state IS NOT NULL
  OR NEW.last_transition_source IS NOT NULL
  OR NEW.last_transition_authority IS NOT NULL
BEGIN
    SELECT RAISE(ABORT, 'new stage attempts must begin PREPARED without transition provenance');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_stage_runs_transition_update
BEFORE UPDATE OF lifecycle_state, last_transition_operation_id, last_transition_from_state, last_transition_source, last_transition_authority ON workflow_stage_runs
FOR EACH ROW
WHEN
    (NEW.lifecycle_state = OLD.lifecycle_state AND (
        NEW.last_transition_operation_id IS NOT OLD.last_transition_operation_id
        OR NEW.last_transition_from_state IS NOT OLD.last_transition_from_state
        OR NEW.last_transition_source IS NOT OLD.last_transition_source
        OR NEW.last_transition_authority IS NOT OLD.last_transition_authority
    ))
    OR
    (NEW.lifecycle_state <> OLD.lifecycle_state AND (
        NEW.last_transition_operation_id IS NULL
        OR length(trim(NEW.last_transition_operation_id)) = 0
        OR NEW.last_transition_from_state IS NOT OLD.lifecycle_state
        OR NOT (
            (OLD.lifecycle_state = 'PREPARED' AND NEW.lifecycle_state IN ('ACTIVE', 'STALE', 'CANCELLED', 'RECOVERY_REQUIRED'))
            OR (OLD.lifecycle_state = 'ACTIVE' AND NEW.lifecycle_state IN ('WAITING_APPROVAL', 'WAITING_EXTERNAL', 'BLOCKED', 'FAILED', 'STALE', 'CANCELLED', 'COMPLETED', 'RECOVERY_REQUIRED'))
            OR (OLD.lifecycle_state = 'WAITING_APPROVAL' AND NEW.lifecycle_state IN ('ACTIVE', 'BLOCKED', 'FAILED', 'STALE', 'CANCELLED', 'RECOVERY_REQUIRED'))
            OR (OLD.lifecycle_state = 'WAITING_EXTERNAL' AND NEW.lifecycle_state IN ('ACTIVE', 'BLOCKED', 'FAILED', 'STALE', 'CANCELLED', 'RECOVERY_REQUIRED'))
            OR (OLD.lifecycle_state = 'BLOCKED' AND NEW.lifecycle_state IN ('ACTIVE', 'FAILED', 'STALE', 'CANCELLED', 'RECOVERY_REQUIRED'))
        )
        OR (NEW.lifecycle_state IN ('CANCELLED', 'COMPLETED') AND (
            NEW.last_transition_source = 'AGENT_REPORTED'
            OR NEW.last_transition_authority NOT IN ('WINDS_POLICY', 'HUMAN_DECISION')
        ))
    ))
BEGIN
    SELECT RAISE(ABORT, 'stage lifecycle update violates canonical transition truth');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_stage_runs_identity_update
BEFORE UPDATE OF stage_run_id, workflow_run_id, stage_key, attempt_ordinal, predecessor_stage_run_id, relation_kind ON workflow_stage_runs
BEGIN
    SELECT RAISE(ABORT, 'stage attempt identity and lineage are immutable');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_stage_runs_no_delete
BEFORE DELETE ON workflow_stage_runs
BEGIN
    SELECT RAISE(ABORT, 'stage attempt history is immutable');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_artifact_baselines_no_update
BEFORE UPDATE ON workflow_artifact_baselines
BEGIN
    SELECT RAISE(ABORT, 'workflow artifact baselines are immutable');
END;
CREATE TRIGGER IF NOT EXISTS trg_workflow_artifact_baselines_no_delete
BEFORE DELETE ON workflow_artifact_baselines
BEGIN
    SELECT RAISE(ABORT, 'workflow artifact baselines are immutable');
END;

-- Persisted actor identities must resolve through the stage workflow's canonical workstream.
CREATE TRIGGER IF NOT EXISTS trg_workflow_actor_bindings_identity_insert
BEFORE INSERT ON workflow_actor_bindings
FOR EACH ROW
WHEN
    (NEW.winds_session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM workflow_stage_runs stage
        JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
        JOIN winds_sessions session ON session.session_id = NEW.winds_session_id
        WHERE stage.stage_run_id = NEW.stage_run_id
          AND session.workstream_id = workflow.workstream_id
    ))
    OR
    (NEW.runtime_binding_id IS NOT NULL AND NOT EXISTS (
        SELECT 1
        FROM workflow_stage_runs stage
        JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
        JOIN runtime_session_bindings runtime ON runtime.binding_id = NEW.runtime_binding_id
        JOIN winds_sessions session ON session.session_id = runtime.session_id
        WHERE stage.stage_run_id = NEW.stage_run_id
          AND session.workstream_id = workflow.workstream_id
          AND (NEW.winds_session_id IS NULL OR NEW.winds_session_id = runtime.session_id)
    ))
BEGIN
    SELECT RAISE(ABORT, 'workflow actor binding does not match canonical stage/workstream identity');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_actor_bindings_no_update
BEFORE UPDATE ON workflow_actor_bindings
BEGIN
    SELECT RAISE(ABORT, 'workflow actor bindings are immutable');
END;
CREATE TRIGGER IF NOT EXISTS trg_workflow_actor_bindings_no_delete
BEFORE DELETE ON workflow_actor_bindings
BEGIN
    SELECT RAISE(ABORT, 'workflow actor bindings are immutable');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_reconstruction_reports_no_update
BEFORE UPDATE ON workflow_reconstruction_reports
BEGIN
    SELECT RAISE(ABORT, 'workflow reconstruction reports are immutable');
END;
CREATE TRIGGER IF NOT EXISTS trg_workflow_reconstruction_reports_no_delete
BEFORE DELETE ON workflow_reconstruction_reports
BEGIN
    SELECT RAISE(ABORT, 'workflow reconstruction reports are immutable');
END;

-- Decision stage/predecessor references may never cross their parent workflow.
CREATE TRIGGER IF NOT EXISTS trg_workflow_decisions_identity_insert
BEFORE INSERT ON workflow_decisions
FOR EACH ROW
WHEN
    (NEW.stage_run_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM workflow_stage_runs stage
        WHERE stage.stage_run_id = NEW.stage_run_id
          AND stage.workflow_run_id = NEW.workflow_run_id
    ))
    OR
    (NEW.predecessor_decision_id IS NOT NULL AND (
        NEW.predecessor_decision_id = NEW.decision_id
        OR NOT EXISTS (
            SELECT 1 FROM workflow_decisions predecessor
            WHERE predecessor.decision_id = NEW.predecessor_decision_id
              AND predecessor.workflow_run_id = NEW.workflow_run_id
        )
    ))
BEGIN
    SELECT RAISE(ABORT, 'workflow decision lineage does not match canonical workflow identity');
END;

CREATE TRIGGER IF NOT EXISTS trg_workflow_decisions_no_update
BEFORE UPDATE ON workflow_decisions
BEGIN
    SELECT RAISE(ABORT, 'workflow decisions are append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_workflow_decisions_no_delete
BEFORE DELETE ON workflow_decisions
BEGIN
    SELECT RAISE(ABORT, 'workflow decisions are append-only');
END;
