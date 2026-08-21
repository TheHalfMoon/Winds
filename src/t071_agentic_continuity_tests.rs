use super::ContinuationResolution;
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

struct TestHome {
    path: PathBuf,
    owned_base: PathBuf,
}

impl TestHome {
    fn new(name: &str) -> Self {
        assert!(
            Path::new(name)
                .components()
                .all(|component| matches!(component, Component::Normal(_))),
            "test-home name must contain only normal path components"
        );
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let owned_base = std::env::temp_dir().join(format!(
            "winds-t071-agentic-continuity-owned-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&owned_base).unwrap();
        let path = owned_base.join(name);
        fs::create_dir(&path).unwrap();
        Self { path, owned_base }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn database(&self) -> PathBuf {
        self.path.join("winds.db")
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let Ok(canonical_base) = fs::canonicalize(&self.owned_base) else {
            return;
        };
        let Ok(canonical_path) = fs::canonicalize(&self.path) else {
            return;
        };
        if canonical_path.parent() != Some(canonical_base.as_path()) {
            return;
        }
        let _ = fs::remove_dir_all(&canonical_path);
        let _ = fs::remove_dir(&canonical_base);
    }
}

fn store_with_workspace(home: &TestHome, workspace_id: &str) -> Store {
    let store = Store::open(home.path()).unwrap();
    let root = format!("/tmp/winds-t071-{workspace_id}");
    let git_dir = format!("{root}/.git");
    store
        .create_workspace(
            NewWorkspace {
                workspace_id,
                canonical_worktree_root: &root,
                git_common_dir: &git_dir,
            },
            10,
        )
        .unwrap();
    store
}

fn seed_workstream_and_session(
    store: &Store,
    workspace_id: &str,
    workstream_id: &str,
    session_id: &str,
    display_name: &str,
    now_ms: i64,
) {
    store
        .create_workstream(
            NewWorkstream {
                workstream_id,
                workspace_id,
                display_name: "Shared task",
            },
            now_ms,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id,
                workstream_id,
                display_name,
            },
            now_ms + 1,
        )
        .unwrap();
}

#[test]
fn new_session_keeps_workstream_while_explicit_new_task_creates_distinct_identity_atomically() {
    let home = TestHome::new("new-session-vs-task");
    let mut store = store_with_workspace(&home, "workspace-main");
    seed_workstream_and_session(
        &store,
        "workspace-main",
        "workstream-a",
        "session-a",
        "Planner",
        100,
    );

    store
        .start_new_winds_session("workstream-a", "session-b", "Planner", 120)
        .unwrap();
    assert_eq!(store.list_workstreams("workspace-main").unwrap().len(), 1);
    assert_eq!(
        store.load_winds_session("session-b").unwrap().workstream_id,
        "workstream-a"
    );

    store
        .create_new_task_with_session(
            "workspace-main",
            "workstream-b",
            "Shared task",
            "session-c",
            "Planner",
            130,
        )
        .unwrap();
    let workstreams = store.list_workstreams("workspace-main").unwrap();
    assert_eq!(workstreams.len(), 2);
    assert_eq!(workstreams[0].display_name, "Shared task");
    assert_eq!(workstreams[1].display_name, "Shared task");
    assert_ne!(workstreams[0].workstream_id, workstreams[1].workstream_id);
    assert_eq!(
        store.load_winds_session("session-c").unwrap().workstream_id,
        "workstream-b"
    );

    let failed = store.create_new_task_with_session(
        "workspace-main",
        "workstream-rollback",
        "Shared task",
        "session-a",
        "duplicate session id",
        140,
    );
    assert!(failed.is_err());
    assert!(store.load_workstream("workstream-rollback").is_err());
}

#[test]
fn fork_origin_is_durable_same_workstream_and_survives_rename_and_reopen() {
    let home = TestHome::new("fork-origin");
    {
        let mut store = store_with_workspace(&home, "workspace-main");
        seed_workstream_and_session(
            &store,
            "workspace-main",
            "workstream-a",
            "session-origin",
            "Planner",
            100,
        );
        store
            .fork_winds_session("session-origin", "session-fork", "Reviewer", 120)
            .unwrap();
        assert!(
            store
                .fork_winds_session("session-origin", "session-origin", "invalid", 121)
                .is_err()
        );
        assert!(
            store
                .fork_winds_session("missing", "session-missing-origin", "invalid", 121)
                .is_err()
        );

        let fork = store.load_winds_session("session-fork").unwrap();
        let origin = store
            .load_winds_session_origin("session-fork")
            .unwrap()
            .unwrap();
        assert_eq!(fork.workstream_id, "workstream-a");
        assert_eq!(origin.session_id, "session-origin");
        assert_eq!(origin.workstream_id, fork.workstream_id);

        store
            .rename_winds_session("session-origin", "Planner renamed", 130)
            .unwrap();
        store
            .rename_winds_session("session-fork", "Reviewer renamed", 131)
            .unwrap();
        let renamed_origin = store
            .load_winds_session_origin("session-fork")
            .unwrap()
            .unwrap();
        assert_eq!(renamed_origin.session_id, "session-origin");
        assert_eq!(renamed_origin.display_name, "Planner renamed");

        store
            .create_workstream(
                NewWorkstream {
                    workstream_id: "workstream-b",
                    workspace_id: "workspace-main",
                    display_name: "Other task",
                },
                140,
            )
            .unwrap();
        store
            .create_winds_session(
                NewWindsSession {
                    session_id: "session-other",
                    workstream_id: "workstream-b",
                    display_name: "Other",
                },
                141,
            )
            .unwrap();
    }

    let reopened = Store::open(home.path()).unwrap();
    let origin = reopened
        .load_winds_session_origin("session-fork")
        .unwrap()
        .unwrap();
    assert_eq!(origin.session_id, "session-origin");
    assert_eq!(origin.workstream_id, "workstream-a");

    let connection = Connection::open(home.database()).unwrap();
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .unwrap();
    let columns = {
        let mut statement = connection
            .prepare("PRAGMA table_info(winds_session_origins)")
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    assert_eq!(
        columns,
        ["session_id", "workstream_id", "origin_session_id"]
    );
    assert!(!columns.iter().any(|column| {
        column.contains("workspace") || column.contains("runtime") || column.contains("native")
    }));

    let cross_workstream = connection.execute(
        "INSERT INTO winds_session_origins(session_id, workstream_id, origin_session_id)
         VALUES (?1, ?2, ?3)",
        params!["session-fork", "workstream-a", "session-other"],
    );
    assert!(cross_workstream.is_err());
}

#[test]
fn ambiguous_continuation_returns_deterministic_candidates_instead_of_recency_guessing() {
    let home = TestHome::new("continuation-selection");
    let mut store = store_with_workspace(&home, "workspace-main");
    store
        .create_new_task_with_session(
            "workspace-main",
            "workstream-a",
            "Task A",
            "session-old",
            "Shared",
            100,
        )
        .unwrap();
    store
        .create_new_task_with_session(
            "workspace-main",
            "workstream-b",
            "Task B",
            "session-new",
            "Shared",
            200,
        )
        .unwrap();
    store
        .start_new_winds_session("workstream-a", "session-unique", "Unique", 150)
        .unwrap();

    match store
        .resolve_winds_continuation("workspace-main", Some("Shared"))
        .unwrap()
    {
        ContinuationResolution::Candidates(candidates) => {
            assert_eq!(
                candidates
                    .iter()
                    .map(|candidate| candidate.session_id.as_str())
                    .collect::<Vec<_>>(),
                ["session-old", "session-new"]
            );
        }
        ContinuationResolution::Selected(_) => panic!("duplicate display name must stay ambiguous"),
    }

    match store
        .resolve_winds_continuation("workspace-main", None)
        .unwrap()
    {
        ContinuationResolution::Candidates(candidates) => {
            assert_eq!(
                candidates
                    .iter()
                    .map(|candidate| candidate.session_id.as_str())
                    .collect::<Vec<_>>(),
                ["session-old", "session-unique", "session-new"]
            );
        }
        ContinuationResolution::Selected(_) => {
            panic!("multiple sessions must not select by recency")
        }
    }

    match store
        .resolve_winds_continuation("workspace-main", Some("session-new"))
        .unwrap()
    {
        ContinuationResolution::Selected(session) => assert_eq!(session.session_id, "session-new"),
        ContinuationResolution::Candidates(_) => panic!("exact stable id must disambiguate"),
    }

    match store
        .resolve_winds_continuation("workspace-main", Some("Unique"))
        .unwrap()
    {
        ContinuationResolution::Selected(session) => {
            assert_eq!(session.session_id, "session-unique")
        }
        ContinuationResolution::Candidates(_) => panic!("unique exact display text may select"),
    }

    assert!(
        store
            .resolve_winds_continuation("workspace-main", Some("missing"))
            .is_err()
    );
    assert!(
        store
            .resolve_winds_continuation("missing-workspace", None)
            .is_err()
    );
}
