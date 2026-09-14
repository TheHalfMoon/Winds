use crate::desktop::{
    DesktopBridgeLayoutRequest, desktop_bridge_load_layout, desktop_bridge_save_layout,
};
use crate::store::{
    DesktopLayoutPresentationInput, NewWindsSession, NewWorkspace, NewWorkstream, Store,
};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t134-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn cleanup(root: &Path) {
    let canonical_root = root.canonicalize().unwrap();
    let canonical_temp = std::env::temp_dir().canonicalize().unwrap();
    assert!(canonical_root.starts_with(&canonical_temp));
    assert!(
        canonical_root
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with("winds-t134-"))
    );
    fs::remove_dir_all(canonical_root).unwrap();
}

fn seed_workspace(store: &Store, workspace_id: &str, workstream_id: &str, now_ms: i64) {
    store
        .create_workspace(
            NewWorkspace {
                workspace_id,
                canonical_worktree_root: &format!("/fixture/{workspace_id}"),
                git_common_dir: &format!("/fixture/{workspace_id}/.git"),
            },
            now_ms,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id,
                workspace_id,
                display_name: "Dual Session workstream",
            },
            now_ms + 1,
        )
        .unwrap();
}

fn seed_session(store: &Store, workstream_id: &str, session_id: &str, now_ms: i64) {
    store
        .create_winds_session(
            NewWindsSession {
                session_id,
                workstream_id,
                display_name: session_id,
            },
            now_ms,
        )
        .unwrap();
}

#[test]
fn layout_bridge_round_trips_two_distinct_session_identities_and_preserves_other_layout_fields() {
    let root = test_root("roundtrip");
    let store = Store::open(&root).unwrap();
    seed_workspace(&store, "workspace-a", "workstream-a", 10);
    seed_session(&store, "workstream-a", "session-left", 20);
    seed_session(&store, "workstream-a", "session-right", 21);
    store
        .save_desktop_layout_presentation(
            DesktopLayoutPresentationInput {
                workspace_id: "workspace-a",
                layout_mode: "DUAL",
                left_session_id: Some("session-left"),
                right_session_id: Some("session-right"),
                split_basis_points: 4600,
                right_dock_surface: "CHANGES",
                right_dock_binding: "RIGHT_SESSION",
                left_dock_collapsed: true,
                left_dock_width_px: 300,
                right_dock_collapsed: true,
                right_dock_width_px: 420,
                appearance: "DARK",
                contrast: "HIGH",
                density: "STANDARD",
                reduced_motion: true,
            },
            None,
            30,
        )
        .unwrap();
    drop(store);

    let loaded = desktop_bridge_load_layout(&root, "workspace-a")
        .unwrap()
        .unwrap();
    assert_eq!(loaded.left_session_id.as_deref(), Some("session-left"));
    assert_eq!(loaded.right_session_id.as_deref(), Some("session-right"));
    assert_eq!(loaded.split_basis_points, 4600);
    assert_eq!(loaded.revision, 1);

    let swapped = desktop_bridge_save_layout(
        &root,
        DesktopBridgeLayoutRequest {
            workspace_id: "workspace-a".to_owned(),
            layout_mode: "DUAL".to_owned(),
            left_session_id: Some("session-right".to_owned()),
            right_session_id: Some("session-left".to_owned()),
            split_basis_points: 5400,
            expected_revision: Some(1),
        },
    )
    .unwrap();
    assert_eq!(swapped.revision, 2);
    assert_eq!(swapped.left_session_id.as_deref(), Some("session-right"));

    let store = Store::open(&root).unwrap();
    let persisted = store
        .load_desktop_layout_presentation("workspace-a")
        .unwrap()
        .unwrap();
    assert_eq!(persisted.right_dock_surface, "CHANGES");
    assert_eq!(persisted.right_dock_binding, "RIGHT_SESSION");
    assert!(persisted.left_dock_collapsed);
    assert!(persisted.right_dock_collapsed);
    assert_eq!(persisted.appearance, "DARK");
    assert_eq!(persisted.contrast, "HIGH");
    assert_eq!(persisted.density, "STANDARD");
    assert!(persisted.reduced_motion);
    drop(store);

    let single = desktop_bridge_save_layout(
        &root,
        DesktopBridgeLayoutRequest {
            workspace_id: "workspace-a".to_owned(),
            layout_mode: "SINGLE".to_owned(),
            left_session_id: Some("session-right".to_owned()),
            right_session_id: None,
            split_basis_points: 5400,
            expected_revision: Some(2),
        },
    )
    .unwrap();
    assert_eq!(single.revision, 3);
    let store = Store::open(&root).unwrap();
    let normalized = store
        .load_desktop_layout_presentation("workspace-a")
        .unwrap()
        .unwrap();
    assert_eq!(normalized.right_dock_binding, "FOLLOW_FOCUS");
    assert_eq!(normalized.right_dock_surface, "CHANGES");
    assert_eq!(normalized.appearance, "DARK");
    drop(store);
    cleanup(&root);
}

#[test]
fn layout_bridge_is_cas_guarded_and_rejects_cross_project_session_targets() {
    let root = test_root("scope-cas");
    let store = Store::open(&root).unwrap();
    seed_workspace(&store, "workspace-a", "workstream-a", 10);
    seed_workspace(&store, "workspace-b", "workstream-b", 20);
    seed_session(&store, "workstream-a", "session-a", 30);
    seed_session(&store, "workstream-a", "session-b", 31);
    seed_session(&store, "workstream-b", "session-foreign", 32);
    drop(store);

    let created = desktop_bridge_save_layout(
        &root,
        DesktopBridgeLayoutRequest {
            workspace_id: "workspace-a".to_owned(),
            layout_mode: "DUAL".to_owned(),
            left_session_id: Some("session-a".to_owned()),
            right_session_id: Some("session-b".to_owned()),
            split_basis_points: 5000,
            expected_revision: None,
        },
    )
    .unwrap();
    assert_eq!(created.revision, 1);

    let stale = desktop_bridge_save_layout(
        &root,
        DesktopBridgeLayoutRequest {
            workspace_id: "workspace-a".to_owned(),
            layout_mode: "SINGLE".to_owned(),
            left_session_id: Some("session-a".to_owned()),
            right_session_id: None,
            split_basis_points: 5000,
            expected_revision: Some(2),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");

    let foreign = desktop_bridge_save_layout(
        &root,
        DesktopBridgeLayoutRequest {
            workspace_id: "workspace-a".to_owned(),
            layout_mode: "DUAL".to_owned(),
            left_session_id: Some("session-a".to_owned()),
            right_session_id: Some("session-foreign".to_owned()),
            split_basis_points: 5000,
            expected_revision: Some(1),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(
        foreign.contains("does not belong to the selected project"),
        "{foreign}"
    );

    let current = desktop_bridge_load_layout(&root, "workspace-a")
        .unwrap()
        .unwrap();
    assert_eq!(current.revision, 1);
    assert_eq!(current.right_session_id.as_deref(), Some("session-b"));
    cleanup(&root);
}
