CREATE TABLE IF NOT EXISTS candidate_runs (
    run_id TEXT PRIMARY KEY,
    repo_path TEXT NOT NULL,
    base_oid TEXT NOT NULL,
    candidate_ref TEXT NOT NULL,
    candidate_oid TEXT NOT NULL,
    candidate_tree TEXT NOT NULL,
    worktree_path TEXT NOT NULL,
    check_command TEXT NOT NULL,
    timeout_secs INTEGER NOT NULL,
    state TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
    event_id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    authority TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS evidence_reports (
    run_id TEXT PRIMARY KEY REFERENCES candidate_runs(run_id),
    eligibility TEXT NOT NULL,
    report_json TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS promotions (
    run_id TEXT PRIMARY KEY REFERENCES candidate_runs(run_id),
    branch TEXT NOT NULL,
    commit_oid TEXT NOT NULL,
    created_unix_ms INTEGER NOT NULL
);
