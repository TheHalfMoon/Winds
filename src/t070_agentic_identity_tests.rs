use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

struct TestHome(PathBuf);

impl TestHome {
    fn new(name: &str) -> Self {
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t070-agentic-identity-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn database(&self) -> PathBuf {
        self.0.join("winds.db")
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn store_with_workspace(home: &TestHome, workspace_id: &str, root_suffix: &str) -> Store {
    let store = Store::open(home.path()).unwrap();
    let root = format!("/tmp/winds-t070-{root_suffix}");
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

#[test]
fn twenty_named_sessions_keep_stable_identity_across_renames() {
    let home = TestHome::new("many-sessions");
    let store = store_with_workspace(&home, "workspace-main", "many-sessions");
    let workstream_names = ["Task", "task", "مهمة", "任务", "Task"];
    let session_names = ["Planner", "planner", "مخطط", "规划"];

    for workstream_index in 0..5 {
        let workstream_id = format!("workstream-{workstream_index:02}");
        store
            .create_workstream(
                NewWorkstream {
                    workstream_id: &workstream_id,
                    workspace_id: "workspace-main",
                    display_name: workstream_names[workstream_index],
                },
                100 + i64::try_from(workstream_index).unwrap(),
            )
            .unwrap();

        for session_index in 0..4 {
            let session_id = format!("session-{workstream_index:02}-{session_index:02}");
            store
                .create_winds_session(
                    NewWindsSession {
                        session_id: &session_id,
                        workstream_id: &workstream_id,
                        display_name: session_names[session_index],
                    },
                    200 + i64::try_from(workstream_index * 10 + session_index).unwrap(),
                )
                .unwrap();
        }
    }

    let workstreams = store.list_workstreams("workspace-main").unwrap();
    assert_eq!(workstreams.len(), 5);
    assert_eq!(workstreams[0].workstream_id, "workstream-00");
    assert_eq!(workstreams[4].workstream_id, "workstream-04");

    let before_workstream = store.load_workstream("workstream-02").unwrap();
    let before_session = store.load_winds_session("session-02-03").unwrap();
    store
        .rename_workstream("workstream-02", "Renamed مهمة", 500)
        .unwrap();
    store
        .rename_winds_session("session-02-03", "Review 复核", 510)
        .unwrap();

    let after_workstream = store.load_workstream("workstream-02").unwrap();
    assert_eq!(after_workstream.workstream_id, before_workstream.workstream_id);
    assert_eq!(after_workstream.workspace_id, before_workstream.workspace_id);
    assert_eq!(after_workstream.created_unix_ms, before_workstream.created_unix_ms);
    assert_eq!(after_workstream.updated_unix_ms, 500);
    assert_eq!(after_workstream.display_name, "Renamed مهمة");

    let after_session = store.load_winds_session("session-02-03").unwrap();
    assert_eq!(after_session.session_id, before_session.session_id);
    assert_eq!(after_session.workstream_id, before_session.workstream_id);
    assert_eq!(after_session.created_unix_ms, before_session.created_unix_ms);
    assert_eq!(after_session.updated_unix_ms, 510);
    assert_eq!(after_session.display_name, "Review 复核");

    let mut session_count = 0_usize;
    for workstream in &workstreams {
        let sessions = store.list_winds_sessions(&workstream.workstream_id).unwrap();
        assert_eq!(sessions.len(), 4);
        session_count += sessions.len();
    }
    assert_eq!(session_count, 20);
}

#[test]
fn workspace_ownership_is_structural_and_invalid_identity_operations_fail_closed() {
    let home = TestHome::new("ownership");
    let store = store_with_workspace(&home, "workspace-a", "ownership-a");
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-b",
                canonical_worktree_root: "/tmp/winds-t070-ownership-b",
                git_common_dir: "/tmp/winds-t070-ownership-b/.git",
            },
            11,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-a",
                workspace_id: "workspace-a",
                display_name: "Shared Name",
            },
            100,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-a",
                workstream_id: "workstream-a",
                display_name: "Shared Name",
            },
            110,
        )
        .unwrap();

    assert!(store.load_workstream("missing-workstream").is_err());
    assert!(store.load_winds_session("missing-session").is_err());
    assert!(store.list_workstreams("missing-workspace").is_err());
    assert!(store.list_winds_sessions("missing-workstream").is_err());
    assert!(store.rename_workstream("missing-workstream", "x", 120).is_err());
    assert!(store.rename_winds_session("missing-session", "x", 120).is_err());
    assert!(
        store
            .create_workstream(
                NewWorkstream {
                    workstream_id: "orphan-workstream",
                    workspace_id: "missing-workspace",
                    display_name: "orphan",
                },
                120,
            )
            .is_err()
    );
    assert!(
        store
            .create_winds_session(
                NewWindsSession {
                    session_id: "orphan-session",
                    workstream_id: "missing-workstream",
                    display_name: "orphan",
                },
                120,
            )
            .is_err()
    );
    assert!(
        store
            .create_workstream(
                NewWorkstream {
                    workstream_id: " ",
                    workspace_id: "workspace-a",
                    display_name: "valid",
                },
                120,
            )
            .is_err()
    );
    assert!(
        store
            .create_winds_session(
                NewWindsSession {
                    session_id: "session-invalid-name",
                    workstream_id: "workstream-a",
                    display_name: "  ",
                },
                120,
            )
            .is_err()
    );
    assert!(
        store
            .create_workstream(
                NewWorkstream {
                    workstream_id: "negative-time",
                    workspace_id: "workspace-a",
                    display_name: "negative",
                },
                -1,
            )
            .is_err()
    );
    assert!(store.rename_workstream("workstream-a", "too-early", 99).is_err());
    assert!(store.rename_winds_session("session-a", "too-early", 109).is_err());

    let connection = Connection::open(home.database()).unwrap();
    connection.pragma_update(None, "foreign_keys", "ON").unwrap();
    let session_columns = {
        let mut statement = connection.prepare("PRAGMA table_info(winds_sessions)").unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    assert_eq!(
        session_columns,
        [
            "session_id",
            "workstream_id",
            "display_name",
            "created_unix_ms",
            "updated_unix_ms",
        ]
    );
    assert!(!session_columns.iter().any(|column| column == "workspace_id"));

    let foreign_keys = {
        let mut statement = connection
            .prepare("PRAGMA foreign_key_list(winds_sessions)")
            .unwrap();
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    assert_eq!(
        foreign_keys,
        [("workstreams".to_owned(), "workstream_id".to_owned(), "workstream_id".to_owned())]
    );

    let direct_orphan = connection.execute(
        "INSERT INTO winds_sessions(
            session_id, workstream_id, display_name, created_unix_ms, updated_unix_ms
         ) VALUES (?1, ?2, ?3, ?4, ?4)",
        params!["direct-orphan", "missing-workstream", "orphan", 200_i64],
    );
    assert!(direct_orphan.is_err());
}

#[test]
fn migration_is_idempotent_and_identity_survives_reopen() {
    let home = TestHome::new("reopen");
    {
        let store = store_with_workspace(&home, "workspace-reopen", "reopen");
        store
            .create_workstream(
                NewWorkstream {
                    workstream_id: "workstream-reopen",
                    workspace_id: "workspace-reopen",
                    display_name: "Persistent task",
                },
                100,
            )
            .unwrap();
        store
            .create_winds_session(
                NewWindsSession {
                    session_id: "session-reopen",
                    workstream_id: "workstream-reopen",
                    display_name: "Persistent session",
                },
                110,
            )
            .unwrap();
    }

    let reopened = Store::open(home.path()).unwrap();
    let workspace = reopened.load_workspace("workspace-reopen").unwrap();
    let workstream = reopened.load_workstream("workstream-reopen").unwrap();
    let session = reopened.load_winds_session("session-reopen").unwrap();

    assert_eq!(workspace.workspace_id, "workspace-reopen");
    assert_eq!(workstream.workspace_id, workspace.workspace_id);
    assert_eq!(session.workstream_id, workstream.workstream_id);
    assert_eq!(reopened.list_workstreams("workspace-reopen").unwrap().len(), 1);
    assert_eq!(reopened.list_winds_sessions("workstream-reopen").unwrap().len(), 1);
}
