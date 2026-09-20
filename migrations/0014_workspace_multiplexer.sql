-- Spec 012 T164: bounded multiplexer presentation persistence only.
-- These rows are durable presentation/recovery metadata. They never prove a live
-- process, controller lease, provider session, verification result, or Git authority.

CREATE TABLE IF NOT EXISTS multiplexer_workspaces (
    multiplexer_workspace_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    workspace_alias TEXT NOT NULL,
    topology_generation INTEGER NOT NULL CHECK (topology_generation > 0),
    snapshot_json TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL CHECK (created_unix_ms >= 0),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= created_unix_ms),
    CHECK (
        length(multiplexer_workspace_id) = 32
        AND multiplexer_workspace_id NOT GLOB '*[^0-9a-f]*'
        AND multiplexer_workspace_id <> '00000000000000000000000000000000'
    ),
    CHECK (length(CAST(workspace_alias AS BLOB)) <= 128),
    CHECK (instr(workspace_alias, char(0)) = 0),
    CHECK (length(CAST(snapshot_json AS BLOB)) BETWEEN 1 AND 262140)
);

CREATE TABLE IF NOT EXISTS multiplexer_layout_templates (
    layout_template_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    template_name TEXT NOT NULL,
    template_json TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL CHECK (created_unix_ms >= 0),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= created_unix_ms),
    CHECK (
        length(layout_template_id) = 32
        AND layout_template_id NOT GLOB '*[^0-9a-f]*'
        AND layout_template_id <> '00000000000000000000000000000000'
    ),
    CHECK (length(CAST(template_name AS BLOB)) <= 128),
    CHECK (instr(template_name, char(0)) = 0),
    CHECK (length(CAST(template_json AS BLOB)) BETWEEN 1 AND 262140)
);

CREATE TABLE IF NOT EXISTS multiplexer_worktree_memberships (
    multiplexer_workspace_id TEXT NOT NULL
        REFERENCES multiplexer_workspaces(multiplexer_workspace_id) ON DELETE CASCADE,
    git_workspace_id TEXT NOT NULL,
    membership_source TEXT NOT NULL CHECK (
        membership_source IN ('EXPLICIT_USER', 'CREATED_BY_WINDS', 'IMPORTED_EXPLICIT')
    ),
    membership_state TEXT NOT NULL CHECK (
        membership_state IN ('PRESENT', 'STALE', 'REMOVED')
    ),
    created_unix_ms INTEGER NOT NULL CHECK (created_unix_ms >= 0),
    last_confirmed_unix_ms INTEGER CHECK (
        last_confirmed_unix_ms IS NULL OR last_confirmed_unix_ms >= created_unix_ms
    ),
    PRIMARY KEY (multiplexer_workspace_id, git_workspace_id),
    CHECK (length(CAST(git_workspace_id AS BLOB)) BETWEEN 1 AND 256),
    CHECK (instr(git_workspace_id, char(0)) = 0)
);

CREATE INDEX IF NOT EXISTS idx_multiplexer_worktree_membership_state
    ON multiplexer_worktree_memberships(
        multiplexer_workspace_id,
        membership_state,
        git_workspace_id
    );

CREATE TABLE IF NOT EXISTS multiplexer_repository_trust (
    repository_identity TEXT PRIMARY KEY,
    canonical_git_common_dir TEXT NOT NULL,
    trust_revision INTEGER NOT NULL CHECK (trust_revision > 0),
    created_unix_ms INTEGER NOT NULL CHECK (created_unix_ms >= 0),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= created_unix_ms),
    CHECK (length(CAST(repository_identity AS BLOB)) BETWEEN 1 AND 512),
    CHECK (instr(repository_identity, char(0)) = 0),
    CHECK (length(CAST(canonical_git_common_dir AS BLOB)) BETWEEN 1 AND 4096),
    CHECK (instr(canonical_git_common_dir, char(0)) = 0)
);

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_workspace_time_regression
BEFORE UPDATE ON multiplexer_workspaces
WHEN NEW.updated_unix_ms < OLD.updated_unix_ms
BEGIN
    SELECT RAISE(ABORT, 'multiplexer workspace update time regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_workspace_generation_regression
BEFORE UPDATE ON multiplexer_workspaces
WHEN NEW.topology_generation < OLD.topology_generation
BEGIN
    SELECT RAISE(ABORT, 'multiplexer topology generation regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_workspace_generation_reuse
BEFORE UPDATE ON multiplexer_workspaces
WHEN NEW.topology_generation = OLD.topology_generation
 AND (
    NEW.workspace_alias <> OLD.workspace_alias
    OR NEW.snapshot_json <> OLD.snapshot_json
 )
BEGIN
    SELECT RAISE(ABORT, 'multiplexer topology generation reused for changed snapshot');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_layout_template_time_regression
BEFORE UPDATE ON multiplexer_layout_templates
WHEN NEW.updated_unix_ms < OLD.updated_unix_ms
BEGIN
    SELECT RAISE(ABORT, 'multiplexer layout template update time regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_membership_confirmation_regression
BEFORE UPDATE ON multiplexer_worktree_memberships
WHEN OLD.last_confirmed_unix_ms IS NOT NULL
 AND (
    NEW.last_confirmed_unix_ms IS NULL
    OR NEW.last_confirmed_unix_ms < OLD.last_confirmed_unix_ms
 )
BEGIN
    SELECT RAISE(ABORT, 'multiplexer worktree membership confirmation regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_repository_trust_time_regression
BEFORE UPDATE ON multiplexer_repository_trust
WHEN NEW.updated_unix_ms < OLD.updated_unix_ms
BEGIN
    SELECT RAISE(ABORT, 'multiplexer repository trust update time regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_repository_trust_revision_regression
BEFORE UPDATE ON multiplexer_repository_trust
WHEN NEW.trust_revision < OLD.trust_revision
BEGIN
    SELECT RAISE(ABORT, 'multiplexer repository trust revision regression');
END;

CREATE TRIGGER IF NOT EXISTS trg_multiplexer_repository_trust_revision_reuse
BEFORE UPDATE ON multiplexer_repository_trust
WHEN NEW.trust_revision = OLD.trust_revision
 AND NEW.canonical_git_common_dir <> OLD.canonical_git_common_dir
BEGIN
    SELECT RAISE(ABORT, 'multiplexer repository trust revision reused for changed identity');
END;
