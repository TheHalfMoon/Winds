use super::{
    DesktopLayoutPresentationInput, DesktopProjectPresentationInput,
    DesktopSessionPresentationInput, NewWindsSession, NewWorkspace, NewWorkstream, Store,
    desktop_presentation_schema_objects,
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t130-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn cleanup(home: &Path) {
    let _ = fs::remove_dir_all(home);
}
fn seed_canonical(store: &Store) {
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-a",
                canonical_worktree_root: "/tmp/workspace-a",
                git_common_dir: "/tmp/workspace-a/.git",
            },
            10,
        )
        .unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-b",
                canonical_worktree_root: "/tmp/workspace-b",
                git_common_dir: "/tmp/workspace-b/.git",
            },
            10,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-a",
                workspace_id: "workspace-a",
                display_name: "Canonical A",
            },
            20,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-b",
                workspace_id: "workspace-b",
                display_name: "Canonical B",
            },
            20,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-a1",
                workstream_id: "workstream-a",
                display_name: "Canonical Session A1",
            },
            30,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-a2",
                workstream_id: "workstream-a",
                display_name: "Canonical Session A2",
            },
            30,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-b1",
                workstream_id: "workstream-b",
                display_name: "Canonical Session B1",
            },
            30,
        )
        .unwrap();
}

fn project_input(name: &str) -> DesktopProjectPresentationInput<'_> {
    DesktopProjectPresentationInput {
        workspace_id: "workspace-a",
        display_name: name,
        pinned: true,
        sort_order: 2,
        collapsed: false,
    }
}
fn session_input(alias: &str) -> DesktopSessionPresentationInput<'_> {
    DesktopSessionPresentationInput {
        session_id: "session-a1",
        display_alias: alias,
        pinned: false,
        sort_order: 3,
        archived: false,
    }
}

fn dual_layout() -> DesktopLayoutPresentationInput<'static> {
    DesktopLayoutPresentationInput {
        workspace_id: "workspace-a",
        layout_mode: "DUAL",
        left_session_id: Some("session-a1"),
        right_session_id: Some("session-a2"),
        split_basis_points: 4700,
        right_dock_surface: "CHANGES",
        right_dock_binding: "FOLLOW_FOCUS",
        left_dock_collapsed: false,
        left_dock_width_px: 280,
        right_dock_collapsed: false,
        right_dock_width_px: 360,
        appearance: "SYSTEM",
        contrast: "STANDARD",
        density: "COMPACT",
        reduced_motion: false,
    }
}

#[test]
fn fresh_database_persists_exact_presentation_state_across_restart() {
    let home = test_home("restart");
    {
        let store = Store::open(&home).unwrap();
        seed_canonical(&store);
        let project = store
            .save_desktop_project_presentation(project_input("Project Alias"), None, 100)
            .unwrap();
        let session = store
            .save_desktop_session_presentation(session_input("Session Alias"), None, 101)
            .unwrap();
        let layout = store
            .save_desktop_layout_presentation(dual_layout(), None, 102)
            .unwrap();
        assert_eq!(project.revision, 1);
        assert_eq!(session.revision, 1);
        assert_eq!(layout.schema_version, 1);
    }
    {
        let store = Store::open(&home).unwrap();
        assert_eq!(
            store
                .load_desktop_project_presentation("workspace-a")
                .unwrap()
                .unwrap()
                .display_name,
            "Project Alias"
        );
        assert_eq!(
            store
                .load_desktop_session_presentation("session-a1")
                .unwrap()
                .unwrap()
                .display_alias,
            "Session Alias"
        );
        let layout = store
            .load_desktop_layout_presentation("workspace-a")
            .unwrap()
            .unwrap();
        assert_eq!(layout.right_session_id.as_deref(), Some("session-a2"));
        assert_eq!(layout.right_dock_surface, "CHANGES");
    }
    cleanup(&home);
}

#[test]
fn migration_inventory_is_exact_and_model_mesh_migration_is_unchanged() {
    let home = test_home("inventory");
    let store = Store::open(&home).unwrap();
    let objects = desktop_presentation_schema_objects(&store.connection).unwrap();
    let names = objects.keys().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "desktop_layout_presentation",
            "desktop_project_presentation",
            "desktop_session_presentation",
            "idx_desktop_project_order",
            "idx_desktop_session_order",
            "trg_desktop_layout_insert_scope",
            "trg_desktop_layout_update_scope",
        ]
    );
    let frozen_migration = include_str!("../migrations/0011_model_mesh_continuity.sql");
    let canonical_bytes = frozen_migration.replace("\r\n", "\n");
    let digest = Sha256::digest(canonical_bytes.as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(
        hex,
        "9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d"
    );
    drop(store);
    cleanup(&home);
}

#[test]
fn presentation_updates_reject_stale_revision_and_retrograde_time() {
    let home = test_home("race");
    let store = Store::open(&home).unwrap();
    seed_canonical(&store);
    let created = store
        .save_desktop_project_presentation(project_input("Initial"), None, 100)
        .unwrap();
    assert_eq!(created.revision, 1);
    let updated = store
        .save_desktop_project_presentation(project_input("Updated"), Some(1), 110)
        .unwrap();
    assert_eq!(updated.revision, 2);
    assert_eq!(updated.display_name, "Updated");
    let stale = store.save_desktop_project_presentation(project_input("Stale"), Some(1), 120);
    assert!(
        stale
            .unwrap_err()
            .to_string()
            .contains("lost revision/update race")
    );
    let retrograde =
        store.save_desktop_project_presentation(project_input("Retrograde"), Some(2), 105);
    assert!(
        retrograde
            .unwrap_err()
            .to_string()
            .contains("lost revision/update race")
    );
    let retained = store
        .load_desktop_project_presentation("workspace-a")
        .unwrap()
        .unwrap();
    assert_eq!(retained.display_name, "Updated");
    assert_eq!(retained.revision, 2);
    drop(store);
    cleanup(&home);
}

#[test]
fn deleting_presentation_records_preserves_canonical_truth() {
    let home = test_home("delete");
    let store = Store::open(&home).unwrap();
    seed_canonical(&store);
    store
        .save_desktop_project_presentation(project_input("Alias"), None, 100)
        .unwrap();
    store
        .save_desktop_session_presentation(session_input("Alias"), None, 101)
        .unwrap();
    store
        .save_desktop_layout_presentation(dual_layout(), None, 102)
        .unwrap();
    assert!(
        store
            .delete_desktop_layout_presentation("workspace-a")
            .unwrap()
    );
    assert!(
        store
            .delete_desktop_session_presentation("session-a1")
            .unwrap()
    );
    assert!(
        store
            .delete_desktop_project_presentation("workspace-a")
            .unwrap()
    );
    assert_eq!(
        store.load_workspace("workspace-a").unwrap().workspace_id,
        "workspace-a"
    );
    assert_eq!(
        store.load_workstream("workstream-a").unwrap().workspace_id,
        "workspace-a"
    );
    assert_eq!(
        store.load_winds_session("session-a1").unwrap().session_id,
        "session-a1"
    );
    assert!(
        store
            .load_desktop_project_presentation("workspace-a")
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .load_desktop_session_presentation("session-a1")
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .load_desktop_layout_presentation("workspace-a")
            .unwrap()
            .is_none()
    );
    drop(store);
    cleanup(&home);
}

#[test]
fn cross_workspace_layout_binding_fails_closed() {
    let home = test_home("scope");
    let store = Store::open(&home).unwrap();
    seed_canonical(&store);
    let input = DesktopLayoutPresentationInput {
        workspace_id: "workspace-b",
        layout_mode: "SINGLE",
        left_session_id: Some("session-a1"),
        right_session_id: None,
        split_basis_points: 5000,
        right_dock_surface: "FILES",
        right_dock_binding: "LEFT_SESSION",
        left_dock_collapsed: false,
        left_dock_width_px: 260,
        right_dock_collapsed: false,
        right_dock_width_px: 340,
        appearance: "DARK",
        contrast: "STANDARD",
        density: "STANDARD",
        reduced_motion: false,
    };
    assert!(
        store
            .save_desktop_layout_presentation(input, None, 100)
            .is_err()
    );
    assert!(
        store
            .load_desktop_layout_presentation("workspace-b")
            .unwrap()
            .is_none()
    );
    drop(store);
    cleanup(&home);
}

#[test]
fn migrated_database_without_presentation_objects_installs_0012() {
    let home = test_home("migrated");
    {
        let store = Store::open(&home).unwrap();
        seed_canonical(&store);
    }
    {
        let connection = Connection::open(home.join("winds.db")).unwrap();
        connection
            .execute_batch(
                "DROP TABLE desktop_layout_presentation;
             DROP TABLE desktop_session_presentation;
             DROP TABLE desktop_project_presentation;",
            )
            .unwrap();
    }
    {
        let store = Store::open(&home).unwrap();
        let objects = desktop_presentation_schema_objects(&store.connection).unwrap();
        assert_eq!(objects.len(), 7);
        assert_eq!(
            store.load_workspace("workspace-a").unwrap().workspace_id,
            "workspace-a"
        );
        store
            .save_desktop_project_presentation(project_input("Migrated"), None, 100)
            .unwrap();
    }
    cleanup(&home);
}

#[test]
fn partial_presentation_schema_fails_safe_without_canonical_damage() {
    let home = test_home("partial");
    {
        let store = Store::open(&home).unwrap();
        seed_canonical(&store);
    }
    {
        let connection = Connection::open(home.join("winds.db")).unwrap();
        connection
            .execute_batch("DROP INDEX idx_desktop_project_order;")
            .unwrap();
    }
    let error = match Store::open(&home) {
        Ok(_) => panic!("partial desktop presentation schema unexpectedly opened"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("desktop presentation schema object inventory mismatch"));
    let connection = Connection::open(home.join("winds.db")).unwrap();
    let workspace_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))
        .unwrap();
    let session_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM winds_sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(workspace_count, 2);
    assert_eq!(session_count, 3);
    drop(connection);
    cleanup(&home);
}
#[test]
fn corrupt_presentation_rows_fail_closed_on_load_without_canonical_damage() {
    let home = test_home("corrupt-row");
    {
        let store = Store::open(&home).unwrap();
        seed_canonical(&store);
        store
            .save_desktop_project_presentation(project_input("Valid"), None, 100)
            .unwrap();
    }
    {
        let connection = Connection::open(home.join("winds.db")).unwrap();
        connection
            .execute_batch(
                "PRAGMA ignore_check_constraints = ON;
                 UPDATE desktop_project_presentation
                 SET pinned = 2, display_name = char(9)
                 WHERE workspace_id = 'workspace-a';",
            )
            .unwrap();
    }
    let store = Store::open(&home).unwrap();
    let error = store
        .load_desktop_project_presentation("workspace-a")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("must be stored as 0 or 1")
            || error.contains("must not contain control characters")
    );
    assert_eq!(
        store.load_workspace("workspace-a").unwrap().workspace_id,
        "workspace-a"
    );
    assert_eq!(
        store.load_winds_session("session-a1").unwrap().session_id,
        "session-a1"
    );
    drop(store);
    cleanup(&home);
}
