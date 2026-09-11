-- Spec 009 T116: append-only Model Mesh target, identity, and continuity substrate.
-- This migration intentionally reuses canonical workflow/runtime/approval relations in winds.db.
-- It does not persist credentials, provider-private state, prompts, transcripts, or model output.

CREATE TABLE IF NOT EXISTS model_mesh_target_requests (
    target_request_id TEXT PRIMARY KEY,
    stage_run_id TEXT NOT NULL REFERENCES workflow_stage_runs(stage_run_id),
    actor_binding_id TEXT NOT NULL REFERENCES workflow_actor_bindings(binding_id),
    actor_role TEXT NOT NULL,
    runtime_kind TEXT NOT NULL,
    requested_provider_id TEXT,
    requested_model_id TEXT,
    selector_class TEXT NOT NULL,
    target_descriptor_digest TEXT NOT NULL,
    selection_approval_id TEXT NOT NULL REFERENCES agentic_delegation_approvals(approval_id),
    created_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(target_request_id)) BETWEEN 1 AND 256),
    CHECK (length(trim(actor_role)) BETWEEN 1 AND 64 AND actor_role = trim(actor_role) AND actor_role = upper(actor_role)),
    CHECK (runtime_kind IN ('CODEX', 'CLAUDE')),
    CHECK (requested_provider_id IS NULL OR (length(requested_provider_id) BETWEEN 1 AND 256 AND requested_provider_id = trim(requested_provider_id) AND instr(requested_provider_id, char(0)) = 0 AND instr(requested_provider_id, char(10)) = 0 AND instr(requested_provider_id, char(13)) = 0)),
    CHECK (requested_model_id IS NULL OR (length(requested_model_id) BETWEEN 1 AND 256 AND requested_model_id = trim(requested_model_id) AND instr(requested_model_id, char(0)) = 0 AND instr(requested_model_id, char(10)) = 0 AND instr(requested_model_id, char(13)) = 0)),
    CHECK (selector_class = 'HUMAN'),
    CHECK (length(target_descriptor_digest) = 64 AND target_descriptor_digest NOT GLOB '*[^0-9a-f]*'),
    CHECK (created_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS model_mesh_identity_claims (
    identity_claim_id TEXT PRIMARY KEY,
    target_request_id TEXT REFERENCES model_mesh_target_requests(target_request_id),
    actor_binding_id TEXT REFERENCES workflow_actor_bindings(binding_id),
    claim_subject TEXT NOT NULL,
    dimension TEXT NOT NULL,
    normalized_value TEXT,
    source_class TEXT NOT NULL,
    observation_basis TEXT,
    runtime_binding_id TEXT REFERENCES runtime_session_bindings(binding_id),
    observed_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(identity_claim_id)) BETWEEN 1 AND 256),
    CHECK (claim_subject IN ('REQUEST_TARGET', 'ACTOR')),
    CHECK (
        (claim_subject = 'REQUEST_TARGET' AND target_request_id IS NOT NULL AND actor_binding_id IS NULL)
        OR
        (claim_subject = 'ACTOR' AND target_request_id IS NULL AND actor_binding_id IS NOT NULL)
    ),
    CHECK (dimension IN ('RUNTIME', 'PROVIDER', 'MODEL', 'NATIVE_SESSION')),
    CHECK (normalized_value IS NULL OR (length(normalized_value) BETWEEN 1 AND 256 AND normalized_value = trim(normalized_value) AND instr(normalized_value, char(0)) = 0 AND instr(normalized_value, char(10)) = 0 AND instr(normalized_value, char(13)) = 0)),
    CHECK (source_class IN ('WINDS_LOCALLY_OBSERVED', 'VENDOR_DECLARED', 'CATALOG_DECLARED', 'AGENT_REPORTED', 'HUMAN_DECIDED', 'UNAVAILABLE')),
    CHECK (source_class <> 'UNAVAILABLE' OR normalized_value IS NULL),
    CHECK (observation_basis IS NULL OR (
        length(trim(observation_basis)) BETWEEN 1 AND 512
        AND observation_basis = trim(observation_basis)
        AND instr(observation_basis, char(0)) = 0
        AND instr(observation_basis, '=') = 0
        AND instr(observation_basis, '{') = 0
        AND instr(observation_basis, '}') = 0
        AND instr(observation_basis, '[') = 0
        AND instr(observation_basis, ']') = 0
        AND instr(lower(observation_basis), 'api_key') = 0
        AND instr(lower(observation_basis), 'apikey') = 0
        AND instr(lower(observation_basis), 'authorization') = 0
        AND instr(lower(observation_basis), 'bearer ') = 0
        AND instr(lower(observation_basis), 'credential') = 0
        AND instr(lower(observation_basis), 'password') = 0
        AND instr(lower(observation_basis), 'secret') = 0
        AND instr(lower(observation_basis), 'token') = 0
        AND instr(lower(observation_basis), 'provider-private') = 0
        AND instr(lower(observation_basis), 'provider_private') = 0
    )),
    CHECK (observed_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS model_mesh_continuity_events (
    continuity_event_id TEXT PRIMARY KEY,
    target_request_id TEXT NOT NULL REFERENCES model_mesh_target_requests(target_request_id),
    source_actor_binding_id TEXT REFERENCES workflow_actor_bindings(binding_id),
    destination_actor_binding_id TEXT REFERENCES workflow_actor_bindings(binding_id),
    continuity_class TEXT NOT NULL,
    context_digest TEXT,
    completeness_state TEXT NOT NULL,
    continuity_permission_digest TEXT,
    authority_approval_id TEXT REFERENCES agentic_delegation_approvals(approval_id),
    authority_claim TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(continuity_event_id)) BETWEEN 1 AND 256),
    CHECK (continuity_class IN ('NATIVE_RESUME', 'RECONSTRUCTED', 'REASSIGNED', 'HANDOFF', 'OWNERSHIP_LOST', 'UNAVAILABLE', 'UNPROVEN')),
    CHECK (context_digest IS NULL OR (length(context_digest) = 64 AND context_digest NOT GLOB '*[^0-9a-f]*')),
    CHECK (completeness_state IN ('COMPLETE', 'INCOMPLETE', 'REDACTED', 'OMITTED', 'UNAVAILABLE', 'MATERIAL_LOSS')),
    CHECK (continuity_permission_digest IS NULL OR (length(continuity_permission_digest) = 64 AND continuity_permission_digest NOT GLOB '*[^0-9a-f]*')),
    CHECK (authority_claim IN ('REQUIRED', 'NO_AUTHORITY_CLAIM')),
    CHECK (
        (authority_claim = 'REQUIRED' AND continuity_permission_digest IS NOT NULL AND authority_approval_id IS NOT NULL)
        OR
        (authority_claim = 'NO_AUTHORITY_CLAIM' AND continuity_permission_digest IS NULL AND authority_approval_id IS NULL)
    ),
    CHECK (created_unix_ms >= 0)
);

CREATE TABLE IF NOT EXISTS model_mesh_continuity_identity_claims (
    continuity_event_id TEXT NOT NULL REFERENCES model_mesh_continuity_events(continuity_event_id),
    actor_role TEXT NOT NULL,
    identity_claim_id TEXT NOT NULL REFERENCES model_mesh_identity_claims(identity_claim_id),
    CHECK (actor_role IN ('SOURCE', 'DESTINATION')),
    PRIMARY KEY (continuity_event_id, actor_role, identity_claim_id)
);

CREATE INDEX IF NOT EXISTS idx_model_mesh_target_requests_stage_time
    ON model_mesh_target_requests(stage_run_id, created_unix_ms, target_request_id);
CREATE INDEX IF NOT EXISTS idx_model_mesh_target_requests_actor_time
    ON model_mesh_target_requests(actor_binding_id, created_unix_ms, target_request_id);
CREATE INDEX IF NOT EXISTS idx_model_mesh_identity_claims_request_time
    ON model_mesh_identity_claims(target_request_id, observed_unix_ms, identity_claim_id);
CREATE INDEX IF NOT EXISTS idx_model_mesh_identity_claims_actor_time
    ON model_mesh_identity_claims(actor_binding_id, observed_unix_ms, identity_claim_id);
CREATE INDEX IF NOT EXISTS idx_model_mesh_continuity_events_request_time
    ON model_mesh_continuity_events(target_request_id, created_unix_ms, continuity_event_id);
CREATE INDEX IF NOT EXISTS idx_model_mesh_continuity_identity_claims_claim
    ON model_mesh_continuity_identity_claims(identity_claim_id, continuity_event_id, actor_role);

-- Target requests must resolve to one exact StageRun actor, concrete Winds session,
-- canonical workflow/workstream/workspace hierarchy, and compatible runtime binding.
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_target_request_scope_insert
BEFORE INSERT ON model_mesh_target_requests
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM workflow_actor_bindings actor
    JOIN workflow_stage_runs stage ON stage.stage_run_id = actor.stage_run_id
    JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
    JOIN workstreams workstream ON workstream.workstream_id = workflow.workstream_id
    JOIN winds_sessions session ON session.session_id = actor.winds_session_id
    LEFT JOIN runtime_session_bindings runtime ON runtime.binding_id = actor.runtime_binding_id
    WHERE actor.binding_id = NEW.actor_binding_id
      AND actor.stage_run_id = NEW.stage_run_id
      AND actor.winds_session_id IS NOT NULL
      AND session.workstream_id = workflow.workstream_id
      AND workstream.workspace_id = workflow.workspace_id
      AND (actor.runtime_binding_id IS NULL OR (
          runtime.session_id = actor.winds_session_id
          AND runtime.runtime_kind = NEW.runtime_kind
      ))
)
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh target request does not match canonical actor/stage/session/work scope');
END;

-- Database-level structural authority binding complements Store SHA-256/canonical-content revalidation.
-- Generic T076 approval JSON cannot satisfy these dedicated Model Mesh fields.
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_target_request_approval_insert
BEFORE INSERT ON model_mesh_target_requests
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM workflow_actor_bindings actor
    JOIN workflow_stage_runs stage ON stage.stage_run_id = actor.stage_run_id
    JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
    JOIN winds_sessions session ON session.session_id = actor.winds_session_id
    JOIN agentic_delegation_approvals approval ON approval.approval_id = NEW.selection_approval_id
    WHERE actor.binding_id = NEW.actor_binding_id
      AND stage.stage_run_id = NEW.stage_run_id
      AND approval.workspace_id = workflow.workspace_id
      AND approval.workstream_id = workflow.workstream_id
      AND approval.session_id = session.session_id
      AND approval.approved_unix_ms <= NEW.created_unix_ms
      AND json_valid(approval.canonical_content_json)
      AND json_extract(approval.canonical_content_json, '$.schema_version') = 1
      AND json_extract(approval.canonical_content_json, '$.purpose') = 'TARGET_SELECTION'
      AND json_extract(approval.canonical_content_json, '$.workspace_id') = workflow.workspace_id
      AND json_extract(approval.canonical_content_json, '$.workstream_id') = workflow.workstream_id
      AND json_extract(approval.canonical_content_json, '$.session_id') = session.session_id
      AND json_extract(approval.canonical_content_json, '$.workflow_run_id') = workflow.workflow_run_id
      AND json_extract(approval.canonical_content_json, '$.stage_run_id') = stage.stage_run_id
      AND json_extract(approval.canonical_content_json, '$.actor_binding_id') = actor.binding_id
      AND json_extract(approval.canonical_content_json, '$.actor_role') = NEW.actor_role
      AND json_extract(approval.canonical_content_json, '$.target_descriptor_digest') = NEW.target_descriptor_digest
      AND json_type(approval.canonical_content_json, '$.continuity_permission_digest') = 'null'
)
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh target request lacks exact TARGET_SELECTION approval scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_model_mesh_target_requests_no_update
BEFORE UPDATE ON model_mesh_target_requests
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh target requests are append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_target_requests_no_delete
BEFORE DELETE ON model_mesh_target_requests
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh target requests are append-only');
END;

-- Claims may reference a runtime binding only when it is the exact binding of the claim actor
-- (or target request actor), with the same concrete Winds session and runtime context.
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_identity_claim_scope_insert
BEFORE INSERT ON model_mesh_identity_claims
FOR EACH ROW
WHEN
    (NEW.claim_subject = 'ACTOR' AND NOT EXISTS (
        SELECT 1
        FROM workflow_actor_bindings actor
        WHERE actor.binding_id = NEW.actor_binding_id
          AND actor.winds_session_id IS NOT NULL
          AND (NEW.runtime_binding_id IS NULL OR (
              actor.runtime_binding_id = NEW.runtime_binding_id
              AND EXISTS (
                  SELECT 1 FROM runtime_session_bindings runtime
                  WHERE runtime.binding_id = NEW.runtime_binding_id
                    AND runtime.session_id = actor.winds_session_id
              )
          ))
    ))
    OR
    (NEW.claim_subject = 'REQUEST_TARGET' AND NOT EXISTS (
        SELECT 1
        FROM model_mesh_target_requests request
        JOIN workflow_actor_bindings actor ON actor.binding_id = request.actor_binding_id
        WHERE request.target_request_id = NEW.target_request_id
          AND actor.winds_session_id IS NOT NULL
          AND (NEW.runtime_binding_id IS NULL OR (
              actor.runtime_binding_id = NEW.runtime_binding_id
              AND EXISTS (
                  SELECT 1 FROM runtime_session_bindings runtime
                  WHERE runtime.binding_id = NEW.runtime_binding_id
                    AND runtime.session_id = actor.winds_session_id
                    AND runtime.runtime_kind = request.runtime_kind
              )
          ))
    ))
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh identity claim does not match canonical subject/runtime context');
END;

CREATE TRIGGER IF NOT EXISTS trg_model_mesh_identity_claims_no_update
BEFORE UPDATE ON model_mesh_identity_claims
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh identity claims are append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_identity_claims_no_delete
BEFORE DELETE ON model_mesh_identity_claims
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh identity claims are append-only');
END;

-- Event actors may be on the exact target stage or on that StageRun's canonical predecessor lineage.
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_event_lineage_insert
BEFORE INSERT ON model_mesh_continuity_events
FOR EACH ROW
WHEN
    (NEW.source_actor_binding_id IS NOT NULL AND NOT EXISTS (
        WITH RECURSIVE lineage(stage_run_id, predecessor_stage_run_id) AS (
            SELECT stage.stage_run_id, stage.predecessor_stage_run_id
            FROM model_mesh_target_requests request
            JOIN workflow_stage_runs stage ON stage.stage_run_id = request.stage_run_id
            WHERE request.target_request_id = NEW.target_request_id
            UNION ALL
            SELECT predecessor.stage_run_id, predecessor.predecessor_stage_run_id
            FROM workflow_stage_runs predecessor
            JOIN lineage current ON predecessor.stage_run_id = current.predecessor_stage_run_id
        )
        SELECT 1
        FROM workflow_actor_bindings actor
        JOIN lineage ON lineage.stage_run_id = actor.stage_run_id
        WHERE actor.binding_id = NEW.source_actor_binding_id
          AND actor.winds_session_id IS NOT NULL
    ))
    OR
    (NEW.destination_actor_binding_id IS NOT NULL AND NOT EXISTS (
        WITH RECURSIVE lineage(stage_run_id, predecessor_stage_run_id) AS (
            SELECT stage.stage_run_id, stage.predecessor_stage_run_id
            FROM model_mesh_target_requests request
            JOIN workflow_stage_runs stage ON stage.stage_run_id = request.stage_run_id
            WHERE request.target_request_id = NEW.target_request_id
            UNION ALL
            SELECT predecessor.stage_run_id, predecessor.predecessor_stage_run_id
            FROM workflow_stage_runs predecessor
            JOIN lineage current ON predecessor.stage_run_id = current.predecessor_stage_run_id
        )
        SELECT 1
        FROM workflow_actor_bindings actor
        JOIN lineage ON lineage.stage_run_id = actor.stage_run_id
        WHERE actor.binding_id = NEW.destination_actor_binding_id
          AND actor.winds_session_id IS NOT NULL
    ))
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity actor is outside canonical target StageRun lineage');
END;

-- REQUIRED event authority must match the exact target request scope and permission digest.
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_event_authority_insert
BEFORE INSERT ON model_mesh_continuity_events
FOR EACH ROW
WHEN NEW.authority_claim = 'REQUIRED' AND NOT EXISTS (
    SELECT 1
    FROM model_mesh_target_requests request
    JOIN workflow_actor_bindings actor ON actor.binding_id = request.actor_binding_id
    JOIN workflow_stage_runs stage ON stage.stage_run_id = request.stage_run_id
    JOIN workflow_runs workflow ON workflow.workflow_run_id = stage.workflow_run_id
    JOIN winds_sessions session ON session.session_id = actor.winds_session_id
    JOIN agentic_delegation_approvals approval ON approval.approval_id = NEW.authority_approval_id
    WHERE request.target_request_id = NEW.target_request_id
      AND approval.workspace_id = workflow.workspace_id
      AND approval.workstream_id = workflow.workstream_id
      AND approval.session_id = session.session_id
      AND approval.approved_unix_ms <= NEW.created_unix_ms
      AND json_valid(approval.canonical_content_json)
      AND json_extract(approval.canonical_content_json, '$.schema_version') = 1
      AND json_extract(approval.canonical_content_json, '$.purpose') = 'CONTINUITY_PERMISSION'
      AND json_extract(approval.canonical_content_json, '$.workspace_id') = workflow.workspace_id
      AND json_extract(approval.canonical_content_json, '$.workstream_id') = workflow.workstream_id
      AND json_extract(approval.canonical_content_json, '$.session_id') = session.session_id
      AND json_extract(approval.canonical_content_json, '$.workflow_run_id') = workflow.workflow_run_id
      AND json_extract(approval.canonical_content_json, '$.stage_run_id') = stage.stage_run_id
      AND json_extract(approval.canonical_content_json, '$.actor_binding_id') = actor.binding_id
      AND json_extract(approval.canonical_content_json, '$.actor_role') = request.actor_role
      AND json_extract(approval.canonical_content_json, '$.target_descriptor_digest') = request.target_descriptor_digest
      AND json_extract(approval.canonical_content_json, '$.continuity_permission_digest') = NEW.continuity_permission_digest
)
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity event lacks exact CONTINUITY_PERMISSION approval scope');
END;

CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_events_no_update
BEFORE UPDATE ON model_mesh_continuity_events
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity events are append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_events_no_delete
BEFORE DELETE ON model_mesh_continuity_events
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity events are append-only');
END;

-- Role-specific association may never borrow identity from the opposite actor.
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_identity_claim_role_insert
BEFORE INSERT ON model_mesh_continuity_identity_claims
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM model_mesh_continuity_events event
    JOIN model_mesh_identity_claims claim ON claim.identity_claim_id = NEW.identity_claim_id
    WHERE event.continuity_event_id = NEW.continuity_event_id
      AND claim.claim_subject = 'ACTOR'
      AND (
          (NEW.actor_role = 'SOURCE'
           AND event.source_actor_binding_id IS NOT NULL
           AND claim.actor_binding_id = event.source_actor_binding_id)
          OR
          (NEW.actor_role = 'DESTINATION'
           AND event.destination_actor_binding_id IS NOT NULL
           AND claim.actor_binding_id = event.destination_actor_binding_id)
      )
)
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity identity claim does not match event actor role');
END;

CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_identity_claims_no_update
BEFORE UPDATE ON model_mesh_continuity_identity_claims
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity identity associations are append-only');
END;
CREATE TRIGGER IF NOT EXISTS trg_model_mesh_continuity_identity_claims_no_delete
BEFORE DELETE ON model_mesh_continuity_identity_claims
BEGIN
    SELECT RAISE(ABORT, 'Model Mesh continuity identity associations are append-only');
END;
