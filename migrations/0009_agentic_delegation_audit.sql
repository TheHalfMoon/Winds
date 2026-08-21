-- T076 stores content-bound human approval evidence in Winds-owned state, not in governed repo content.
-- The record is deliberately narrow: explicit canonical identity/scope content, its SHA-256 digest,
-- and approval time only. No credential, auth token, full environment, signing key, or PKI material.
CREATE TABLE IF NOT EXISTS agentic_delegation_approvals (
    approval_id TEXT PRIMARY KEY,
    workstream_id TEXT NOT NULL REFERENCES workstreams(workstream_id),
    session_id TEXT NOT NULL REFERENCES winds_sessions(session_id),
    workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
    content_digest TEXT NOT NULL,
    canonical_content_json TEXT NOT NULL,
    approved_unix_ms INTEGER NOT NULL,
    CHECK (length(trim(approval_id)) > 0),
    CHECK (length(content_digest) = 64),
    CHECK (length(canonical_content_json) > 0),
    CHECK (approved_unix_ms >= 0)
);

CREATE INDEX IF NOT EXISTS idx_agentic_delegation_approvals_session_time
    ON agentic_delegation_approvals(session_id, approved_unix_ms, approval_id);

-- Fail closed if duplicated identity columns do not describe the same canonical Winds chain.
CREATE TRIGGER IF NOT EXISTS trg_agentic_delegation_approval_identity_insert
BEFORE INSERT ON agentic_delegation_approvals
FOR EACH ROW
WHEN NOT EXISTS (
    SELECT 1
    FROM winds_sessions session
    INNER JOIN workstreams workstream
        ON workstream.workstream_id = session.workstream_id
    WHERE session.session_id = NEW.session_id
      AND session.workstream_id = NEW.workstream_id
      AND workstream.workspace_id = NEW.workspace_id
)
BEGIN
    SELECT RAISE(ABORT, 'approval identity does not match canonical Winds hierarchy');
END;

-- Approval audit rows are append-only. Content changes require a distinct fresh approval record.
CREATE TRIGGER IF NOT EXISTS trg_agentic_delegation_approval_no_update
BEFORE UPDATE ON agentic_delegation_approvals
BEGIN
    SELECT RAISE(ABORT, 'human approval audit rows are immutable');
END;

CREATE TRIGGER IF NOT EXISTS trg_agentic_delegation_approval_no_delete
BEFORE DELETE ON agentic_delegation_approvals
BEGIN
    SELECT RAISE(ABORT, 'human approval audit rows are immutable');
END;
