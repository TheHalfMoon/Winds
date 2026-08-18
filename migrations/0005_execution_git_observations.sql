CREATE TABLE IF NOT EXISTS execution_git_observations (
    execution_id TEXT NOT NULL REFERENCES executions(execution_id),
    boundary TEXT NOT NULL,
    availability TEXT NOT NULL,
    fact_source TEXT NOT NULL,
    head_oid TEXT,
    branch TEXT,
    detached INTEGER,
    dirty INTEGER,
    worktree_state_format TEXT,
    worktree_state_sha256 TEXT,
    observed_unix_ms INTEGER,
    PRIMARY KEY (execution_id, boundary),
    CHECK (boundary IN ('BEFORE', 'AFTER')),
    CHECK (availability IN ('OBSERVED', 'UNAVAILABLE')),
    CHECK (detached IS NULL OR detached IN (0, 1)),
    CHECK (dirty IS NULL OR dirty IN (0, 1)),
    CHECK (NOT (detached = 1 AND branch IS NOT NULL)),
    CHECK (
        (
            availability = 'UNAVAILABLE'
            AND head_oid IS NULL
            AND branch IS NULL
            AND detached IS NULL
            AND dirty IS NULL
            AND worktree_state_format IS NULL
            AND worktree_state_sha256 IS NULL
        )
        OR
        (
            availability = 'OBSERVED'
            AND detached IS NOT NULL
            AND dirty IS NOT NULL
            AND worktree_state_format IS NOT NULL
            AND worktree_state_sha256 IS NOT NULL
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_execution_git_observations_execution
    ON execution_git_observations(execution_id, boundary);
