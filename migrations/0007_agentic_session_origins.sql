-- T071 keeps the T070 ownership chain intact. The normalized relation uses
-- workstream_id only to structurally prove that a fork and its origin belong to
-- the same canonical workstream; workspace identity remains derived through
-- workstreams and is deliberately not duplicated here.
CREATE UNIQUE INDEX IF NOT EXISTS idx_winds_sessions_identity_workstream
    ON winds_sessions(session_id, workstream_id);

CREATE TABLE IF NOT EXISTS winds_session_origins (
    session_id TEXT PRIMARY KEY,
    workstream_id TEXT NOT NULL,
    origin_session_id TEXT NOT NULL,
    FOREIGN KEY (session_id, workstream_id)
        REFERENCES winds_sessions(session_id, workstream_id),
    FOREIGN KEY (origin_session_id, workstream_id)
        REFERENCES winds_sessions(session_id, workstream_id),
    CHECK (length(trim(session_id)) > 0),
    CHECK (length(trim(workstream_id)) > 0),
    CHECK (length(trim(origin_session_id)) > 0),
    CHECK (session_id <> origin_session_id)
);

CREATE INDEX IF NOT EXISTS idx_winds_session_origins_origin
    ON winds_session_origins(origin_session_id, workstream_id, session_id);
