CREATE TABLE IF NOT EXISTS desktop_project_presentation (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    pinned INTEGER NOT NULL CHECK (pinned IN (0, 1)),
    sort_order INTEGER NOT NULL CHECK (sort_order >= 0),
    collapsed INTEGER NOT NULL CHECK (collapsed IN (0, 1)),
    revision INTEGER NOT NULL CHECK (revision >= 1),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= 0),
    CHECK (length(trim(display_name)) BETWEEN 1 AND 256)
);

CREATE TABLE IF NOT EXISTS desktop_session_presentation (
    session_id TEXT PRIMARY KEY REFERENCES winds_sessions(session_id) ON DELETE CASCADE,
    display_alias TEXT NOT NULL,
    pinned INTEGER NOT NULL CHECK (pinned IN (0, 1)),
    sort_order INTEGER NOT NULL CHECK (sort_order >= 0),
    archived INTEGER NOT NULL CHECK (archived IN (0, 1)),
    revision INTEGER NOT NULL CHECK (revision >= 1),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= 0),
    CHECK (length(trim(display_alias)) BETWEEN 1 AND 256)
);

CREATE TABLE IF NOT EXISTS desktop_layout_presentation (
    workspace_id TEXT PRIMARY KEY REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    layout_mode TEXT NOT NULL CHECK (layout_mode IN ('SINGLE', 'DUAL')),
    left_session_id TEXT REFERENCES winds_sessions(session_id) ON DELETE CASCADE,
    right_session_id TEXT REFERENCES winds_sessions(session_id) ON DELETE CASCADE,
    split_basis_points INTEGER NOT NULL CHECK (split_basis_points BETWEEN 1000 AND 9000),
    right_dock_surface TEXT NOT NULL CHECK (
        right_dock_surface IN ('FILES', 'CHANGES', 'EVIDENCE', 'CONTEXT', 'ARTIFACTS', 'NEEDS_YOU')
    ),
    right_dock_binding TEXT NOT NULL CHECK (
        right_dock_binding IN ('FOLLOW_FOCUS', 'PROJECT', 'LEFT_SESSION', 'RIGHT_SESSION')
    ),
    left_dock_collapsed INTEGER NOT NULL CHECK (left_dock_collapsed IN (0, 1)),
    left_dock_width_px INTEGER NOT NULL CHECK (left_dock_width_px BETWEEN 160 AND 640),
    right_dock_collapsed INTEGER NOT NULL CHECK (right_dock_collapsed IN (0, 1)),
    right_dock_width_px INTEGER NOT NULL CHECK (right_dock_width_px BETWEEN 240 AND 720),
    appearance TEXT NOT NULL CHECK (appearance IN ('SYSTEM', 'DARK', 'LIGHT')),
    contrast TEXT NOT NULL CHECK (contrast IN ('STANDARD', 'HIGH')),
    density TEXT NOT NULL CHECK (density IN ('COMPACT', 'STANDARD')),
    reduced_motion INTEGER NOT NULL CHECK (reduced_motion IN (0, 1)),
    revision INTEGER NOT NULL CHECK (revision >= 1),
    updated_unix_ms INTEGER NOT NULL CHECK (updated_unix_ms >= 0),
    CHECK (
        (layout_mode = 'SINGLE' AND right_session_id IS NULL)
        OR (layout_mode = 'DUAL' AND left_session_id IS NOT NULL
            AND right_session_id IS NOT NULL AND left_session_id <> right_session_id)
    ),
    CHECK (right_dock_binding <> 'LEFT_SESSION' OR left_session_id IS NOT NULL),
    CHECK (right_dock_binding <> 'RIGHT_SESSION' OR right_session_id IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_desktop_project_order
    ON desktop_project_presentation(pinned DESC, sort_order, workspace_id);

CREATE INDEX IF NOT EXISTS idx_desktop_session_order
    ON desktop_session_presentation(pinned DESC, sort_order, session_id);

CREATE TRIGGER IF NOT EXISTS trg_desktop_layout_insert_scope
BEFORE INSERT ON desktop_layout_presentation
WHEN (NEW.left_session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM winds_sessions s JOIN workstreams w ON w.workstream_id = s.workstream_id
        WHERE s.session_id = NEW.left_session_id AND w.workspace_id = NEW.workspace_id
    ))
    OR (NEW.right_session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM winds_sessions s JOIN workstreams w ON w.workstream_id = s.workstream_id
        WHERE s.session_id = NEW.right_session_id AND w.workspace_id = NEW.workspace_id
    ))
BEGIN
    SELECT RAISE(ABORT, 'desktop layout session/workspace mismatch');
END;

CREATE TRIGGER IF NOT EXISTS trg_desktop_layout_update_scope
BEFORE UPDATE ON desktop_layout_presentation
WHEN (NEW.left_session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM winds_sessions s JOIN workstreams w ON w.workstream_id = s.workstream_id
        WHERE s.session_id = NEW.left_session_id AND w.workspace_id = NEW.workspace_id
    ))
    OR (NEW.right_session_id IS NOT NULL AND NOT EXISTS (
        SELECT 1 FROM winds_sessions s JOIN workstreams w ON w.workstream_id = s.workstream_id
        WHERE s.session_id = NEW.right_session_id AND w.workspace_id = NEW.workspace_id
    ))
BEGIN
    SELECT RAISE(ABORT, 'desktop layout session/workspace mismatch');
END;
