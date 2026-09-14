use crate::desktop::{
    DesktopBridgeCreateSessionRequest, DesktopBridgeProjectPresentationRequest,
    DesktopBridgeRenameSessionRequest, DesktopBridgeSessionPresentationRequest,
    desktop_bridge_create_session, desktop_bridge_rename_session, desktop_bridge_snapshot,
    desktop_bridge_update_project, desktop_bridge_update_session,
};
use crate::store::{NewWorkspace, NewWorkstream, Store};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t132-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn cleanup_owned_root(root: &Path) {
    let canonical_root = root.canonicalize().unwrap();
    let canonical_temp = std::env::temp_dir().canonicalize().unwrap();
    let owned_name = canonical_root
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("winds-t132-"));
    assert!(canonical_root.starts_with(&canonical_temp));
    assert!(owned_name);
    fs::remove_dir_all(canonical_root).unwrap();
}

fn seed_workspace(store: &Store, suffix: &str, now_ms: i64) -> (String, String) {
    let workspace_id = format!("workspace-{suffix}");
    let workstream_id = format!("workstream-{suffix}");
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: &workspace_id,
                canonical_worktree_root: &format!("/fixture/{workspace_id}"),
                git_common_dir: &format!("/fixture/{workspace_id}/.git"),
            },
            now_ms,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: &workstream_id,
                workspace_id: &workspace_id,
                display_name: "Desktop bridge workstream",
            },
            now_ms + 1,
        )
        .unwrap();
    (workspace_id, workstream_id)
}

#[test]
fn bridge_snapshot_preserves_empty_project_workstream_inventory() {
    let root = test_root("empty-project");
    let store = Store::open(&root).unwrap();
    let (workspace_id, workstream_id) = seed_workspace(&store, "empty", 100);
    drop(store);

    let snapshot = desktop_bridge_snapshot(&root).unwrap();
    assert_eq!(snapshot.projects.len(), 1);
    let project = &snapshot.projects[0];
    assert_eq!(project.project.canonical_workspace_id, workspace_id);
    assert!(project.sessions.is_empty());
    assert_eq!(project.available_workstreams.len(), 1);
    assert_eq!(
        project.available_workstreams[0].workstream_id,
        workstream_id
    );
    assert_eq!(
        project.available_workstreams[0].display_name,
        "Desktop bridge workstream"
    );
    cleanup_owned_root(&root);
}

#[test]
fn bridge_project_presentation_create_and_update_are_cas_guarded() {
    let root = test_root("project-presentation");
    let store = Store::open(&root).unwrap();
    let (workspace_id, _) = seed_workspace(&store, "project", 200);
    drop(store);

    let created = desktop_bridge_update_project(
        &root,
        DesktopBridgeProjectPresentationRequest {
            workspace_id: workspace_id.clone(),
            display_name: "Pinned Project".to_owned(),
            pinned: true,
            sort_order: 7,
            collapsed: true,
            expected_revision: None,
        },
    )
    .unwrap();
    assert_eq!(created.presentation_revision, Some(1));
    assert!(created.pinned);
    assert!(created.collapsed);
    assert_eq!(created.presentation_order, 7);

    let updated = desktop_bridge_update_project(
        &root,
        DesktopBridgeProjectPresentationRequest {
            workspace_id: workspace_id.clone(),
            display_name: "Renamed Project".to_owned(),
            pinned: false,
            sort_order: 3,
            collapsed: false,
            expected_revision: Some(1),
        },
    )
    .unwrap();
    assert_eq!(updated.presentation_revision, Some(2));

    let stale = desktop_bridge_update_project(
        &root,
        DesktopBridgeProjectPresentationRequest {
            workspace_id,
            display_name: "Stale".to_owned(),
            pinned: false,
            sort_order: 0,
            collapsed: false,
            expected_revision: Some(1),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");
    cleanup_owned_root(&root);
}

#[test]
fn bridge_session_mutations_preserve_identity_scope_and_rename_revision() {
    let root = test_root("session-commands");
    let store = Store::open(&root).unwrap();
    let (workspace_a, workstream_a) = seed_workspace(&store, "a", 300);
    let (_, workstream_b) = seed_workspace(&store, "b", 400);
    drop(store);

    let cross_project = desktop_bridge_create_session(
        &root,
        DesktopBridgeCreateSessionRequest {
            workspace_id: workspace_a.clone(),
            workstream_id: workstream_b,
            display_name: "Wrong Project".to_owned(),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(cross_project.contains("does not belong to the selected project"));

    let created = desktop_bridge_create_session(
        &root,
        DesktopBridgeCreateSessionRequest {
            workspace_id: workspace_a.clone(),
            workstream_id: workstream_a.clone(),
            display_name: "Canonical Session".to_owned(),
        },
    )
    .unwrap();
    let session_id = created.canonical_session_id.clone();
    assert_eq!(created.canonical_workspace_id, workspace_a);
    assert_eq!(created.canonical_workstream_id, workstream_a);

    let renamed = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Renamed Canonical Session".to_owned(),
            presentation: None,
        },
    )
    .unwrap();
    assert_eq!(renamed.canonical_session_id, session_id);
    assert_eq!(renamed.canonical_display_name, "Renamed Canonical Session");

    let presented = desktop_bridge_update_session(
        &root,
        DesktopBridgeSessionPresentationRequest {
            session_id: session_id.clone(),
            display_alias: "Pinned Alias".to_owned(),
            pinned: true,
            sort_order: 9,
            archived: false,
            expected_revision: None,
        },
    )
    .unwrap();
    assert_eq!(presented.display_name, "Pinned Alias");
    assert_eq!(presented.presentation_revision, Some(1));

    let synchronized = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Unified Session Name".to_owned(),
            presentation: Some(DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Unified Session Name".to_owned(),
                pinned: true,
                sort_order: 9,
                archived: false,
                expected_revision: Some(1),
            }),
        },
    )
    .unwrap();
    assert_eq!(synchronized.display_name, "Unified Session Name");
    assert_eq!(synchronized.canonical_display_name, "Unified Session Name");
    assert_eq!(
        synchronized.display_alias.as_deref(),
        Some("Unified Session Name")
    );
    assert_eq!(synchronized.presentation_revision, Some(2));

    let stale_rename = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Stale Rename".to_owned(),
            presentation: Some(DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Stale Rename".to_owned(),
                pinned: true,
                sort_order: 9,
                archived: false,
                expected_revision: Some(1),
            }),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(
        stale_rename.contains("lost revision/update race"),
        "{stale_rename}"
    );
    let after_stale = Store::open(&root)
        .unwrap()
        .load_winds_session(&session_id)
        .unwrap();
    assert_eq!(after_stale.display_name, "Unified Session Name");

    let updated = desktop_bridge_update_session(
        &root,
        DesktopBridgeSessionPresentationRequest {
            session_id: session_id.clone(),
            display_alias: "Updated Alias".to_owned(),
            pinned: false,
            sort_order: 4,
            archived: true,
            expected_revision: Some(2),
        },
    )
    .unwrap();
    assert_eq!(updated.presentation_revision, Some(3));
    assert!(updated.archived);

    let stale = desktop_bridge_update_session(
        &root,
        DesktopBridgeSessionPresentationRequest {
            session_id: session_id.clone(),
            display_alias: "Stale Alias".to_owned(),
            pinned: false,
            sort_order: 5,
            archived: false,
            expected_revision: Some(1),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");

    let snapshot = desktop_bridge_snapshot(&root).unwrap();
    let projected = snapshot
        .projects
        .iter()
        .flat_map(|project| project.sessions.iter())
        .find(|session| session.canonical_session_id == session_id)
        .unwrap();
    assert_eq!(projected.display_name, "Updated Alias");
    assert!(projected.archived);
    cleanup_owned_root(&root);
}
