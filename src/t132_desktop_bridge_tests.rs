use crate::desktop::{
    DesktopBridgeCreateSessionRequest, DesktopBridgeProjectPresentationBatchRequest,
    DesktopBridgeProjectPresentationRequest, DesktopBridgeRenameSessionRequest,
    DesktopBridgeSessionPresentationBatchRequest, DesktopBridgeSessionPresentationRequest,
    desktop_bridge_create_session, desktop_bridge_rename_session,
    desktop_bridge_resolve_default_home, desktop_bridge_snapshot, desktop_bridge_update_project,
    desktop_bridge_update_session,
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

fn project_batch(
    update: DesktopBridgeProjectPresentationRequest,
) -> DesktopBridgeProjectPresentationBatchRequest {
    DesktopBridgeProjectPresentationBatchRequest {
        updates: vec![update],
    }
}

fn session_batch(
    update: DesktopBridgeSessionPresentationRequest,
) -> DesktopBridgeSessionPresentationBatchRequest {
    DesktopBridgeSessionPresentationBatchRequest {
        updates: vec![update],
    }
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
fn bridge_default_home_skips_non_absolute_fallbacks_but_keeps_explicit_override_strict() {
    let absolute_profile = std::env::current_dir().unwrap();
    let resolved = desktop_bridge_resolve_default_home(
        None,
        Some(PathBuf::from("relative-home")),
        Some(absolute_profile.clone()),
    )
    .unwrap();
    assert_eq!(resolved, absolute_profile.join(".winds"));

    let explicit = absolute_profile.join("explicit-winds-home");
    assert_eq!(
        desktop_bridge_resolve_default_home(
            Some(explicit.clone()),
            Some(absolute_profile.clone()),
            None,
        )
        .unwrap(),
        explicit
    );

    let error = desktop_bridge_resolve_default_home(
        Some(PathBuf::from("relative-explicit")),
        Some(absolute_profile),
        None,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("WINDS_HOME must resolve to an absolute path"));
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
        project_batch(DesktopBridgeProjectPresentationRequest {
            workspace_id: workspace_id.clone(),
            display_name: "Pinned Project".to_owned(),
            pinned: true,
            sort_order: 7,
            collapsed: true,
            expected_revision: None,
        }),
    )
    .unwrap()
    .into_iter()
    .next()
    .unwrap();
    assert_eq!(created.presentation_revision, Some(1));
    assert!(created.pinned);
    assert!(created.collapsed);
    assert_eq!(created.presentation_order, 7);

    let updated = desktop_bridge_update_project(
        &root,
        project_batch(DesktopBridgeProjectPresentationRequest {
            workspace_id: workspace_id.clone(),
            display_name: "Renamed Project".to_owned(),
            pinned: false,
            sort_order: 3,
            collapsed: false,
            expected_revision: Some(1),
        }),
    )
    .unwrap()
    .into_iter()
    .next()
    .unwrap();
    assert_eq!(updated.presentation_revision, Some(2));

    let stale = desktop_bridge_update_project(
        &root,
        project_batch(DesktopBridgeProjectPresentationRequest {
            workspace_id,
            display_name: "Stale".to_owned(),
            pinned: false,
            sort_order: 0,
            collapsed: false,
            expected_revision: Some(1),
        }),
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");
    cleanup_owned_root(&root);
}

#[test]
fn bridge_project_batch_rolls_back_if_a_later_cas_is_stale() {
    let root = test_root("project-batch-rollback");
    let store = Store::open(&root).unwrap();
    let (workspace_a, _) = seed_workspace(&store, "batch-project-a", 500);
    let (workspace_b, _) = seed_workspace(&store, "batch-project-b", 600);
    drop(store);

    desktop_bridge_update_project(
        &root,
        DesktopBridgeProjectPresentationBatchRequest {
            updates: vec![
                DesktopBridgeProjectPresentationRequest {
                    workspace_id: workspace_a.clone(),
                    display_name: "Project A".to_owned(),
                    pinned: false,
                    sort_order: 10,
                    collapsed: false,
                    expected_revision: None,
                },
                DesktopBridgeProjectPresentationRequest {
                    workspace_id: workspace_b.clone(),
                    display_name: "Project B".to_owned(),
                    pinned: false,
                    sort_order: 20,
                    collapsed: false,
                    expected_revision: None,
                },
            ],
        },
    )
    .unwrap();
    desktop_bridge_update_project(
        &root,
        project_batch(DesktopBridgeProjectPresentationRequest {
            workspace_id: workspace_b.clone(),
            display_name: "Project B fresh".to_owned(),
            pinned: false,
            sort_order: 20,
            collapsed: false,
            expected_revision: Some(1),
        }),
    )
    .unwrap();

    let stale = desktop_bridge_update_project(
        &root,
        DesktopBridgeProjectPresentationBatchRequest {
            updates: vec![
                DesktopBridgeProjectPresentationRequest {
                    workspace_id: workspace_a.clone(),
                    display_name: "Project A moved".to_owned(),
                    pinned: false,
                    sort_order: 20,
                    collapsed: false,
                    expected_revision: Some(1),
                },
                DesktopBridgeProjectPresentationRequest {
                    workspace_id: workspace_b.clone(),
                    display_name: "Project B stale".to_owned(),
                    pinned: false,
                    sort_order: 10,
                    collapsed: false,
                    expected_revision: Some(1),
                },
            ],
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");

    let store = Store::open(&root).unwrap();
    let project_a = store
        .load_desktop_project_presentation(&workspace_a)
        .unwrap()
        .unwrap();
    let project_b = store
        .load_desktop_project_presentation(&workspace_b)
        .unwrap()
        .unwrap();
    assert_eq!(project_a.display_name, "Project A");
    assert_eq!(project_a.sort_order, 10);
    assert_eq!(project_a.revision, 1);
    assert_eq!(project_b.display_name, "Project B fresh");
    assert_eq!(project_b.sort_order, 20);
    assert_eq!(project_b.revision, 2);
    drop(store);
    cleanup_owned_root(&root);
}

#[test]
fn bridge_session_batch_rolls_back_if_a_later_cas_is_stale() {
    let root = test_root("session-batch-rollback");
    let store = Store::open(&root).unwrap();
    let (workspace_id, workstream_id) = seed_workspace(&store, "batch-session", 700);
    drop(store);
    let session_a = desktop_bridge_create_session(
        &root,
        DesktopBridgeCreateSessionRequest {
            workspace_id: workspace_id.clone(),
            workstream_id: workstream_id.clone(),
            display_name: "Session A".to_owned(),
        },
    )
    .unwrap();
    let session_b = desktop_bridge_create_session(
        &root,
        DesktopBridgeCreateSessionRequest {
            workspace_id,
            workstream_id,
            display_name: "Session B".to_owned(),
        },
    )
    .unwrap();

    desktop_bridge_update_session(
        &root,
        DesktopBridgeSessionPresentationBatchRequest {
            updates: vec![
                DesktopBridgeSessionPresentationRequest {
                    session_id: session_a.canonical_session_id.clone(),
                    display_alias: "Session A alias".to_owned(),
                    pinned: false,
                    sort_order: 10,
                    archived: false,
                    expected_revision: None,
                },
                DesktopBridgeSessionPresentationRequest {
                    session_id: session_b.canonical_session_id.clone(),
                    display_alias: "Session B alias".to_owned(),
                    pinned: false,
                    sort_order: 20,
                    archived: false,
                    expected_revision: None,
                },
            ],
        },
    )
    .unwrap();
    desktop_bridge_update_session(
        &root,
        session_batch(DesktopBridgeSessionPresentationRequest {
            session_id: session_b.canonical_session_id.clone(),
            display_alias: "Session B fresh".to_owned(),
            pinned: false,
            sort_order: 20,
            archived: false,
            expected_revision: Some(1),
        }),
    )
    .unwrap();

    let stale = desktop_bridge_update_session(
        &root,
        DesktopBridgeSessionPresentationBatchRequest {
            updates: vec![
                DesktopBridgeSessionPresentationRequest {
                    session_id: session_a.canonical_session_id.clone(),
                    display_alias: "Session A moved".to_owned(),
                    pinned: false,
                    sort_order: 20,
                    archived: false,
                    expected_revision: Some(1),
                },
                DesktopBridgeSessionPresentationRequest {
                    session_id: session_b.canonical_session_id.clone(),
                    display_alias: "Session B stale".to_owned(),
                    pinned: false,
                    sort_order: 10,
                    archived: false,
                    expected_revision: Some(1),
                },
            ],
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");

    let store = Store::open(&root).unwrap();
    let presentation_a = store
        .load_desktop_session_presentation(&session_a.canonical_session_id)
        .unwrap()
        .unwrap();
    let presentation_b = store
        .load_desktop_session_presentation(&session_b.canonical_session_id)
        .unwrap()
        .unwrap();
    assert_eq!(presentation_a.display_alias, "Session A alias");
    assert_eq!(presentation_a.sort_order, 10);
    assert_eq!(presentation_a.revision, 1);
    assert_eq!(presentation_b.display_alias, "Session B fresh");
    assert_eq!(presentation_b.sort_order, 20);
    assert_eq!(presentation_b.revision, 2);
    drop(store);
    cleanup_owned_root(&root);
}

#[test]
fn bridge_rename_is_presentation_only_and_cas_guarded() {
    let root = test_root("rename-presentation-only");
    let store = Store::open(&root).unwrap();
    let (workspace_id, workstream_id) = seed_workspace(&store, "rename-only", 800);
    drop(store);

    let created = desktop_bridge_create_session(
        &root,
        DesktopBridgeCreateSessionRequest {
            workspace_id,
            workstream_id,
            display_name: "Canonical Before".to_owned(),
        },
    )
    .unwrap();
    let session_id = created.canonical_session_id.clone();

    let renamed = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Alias One".to_owned(),
            presentation: DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Alias One".to_owned(),
                pinned: false,
                sort_order: 0,
                archived: false,
                expected_revision: None,
            },
        },
    )
    .unwrap();
    assert_eq!(renamed.display_name, "Alias One");
    assert_eq!(renamed.display_alias.as_deref(), Some("Alias One"));
    assert_eq!(renamed.canonical_display_name, "Canonical Before");
    assert_eq!(renamed.presentation_revision, Some(1));

    let updated = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Alias Two".to_owned(),
            presentation: DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Alias Two".to_owned(),
                pinned: false,
                sort_order: 0,
                archived: false,
                expected_revision: Some(1),
            },
        },
    )
    .unwrap();
    assert_eq!(updated.display_name, "Alias Two");
    assert_eq!(updated.canonical_display_name, "Canonical Before");
    assert_eq!(updated.presentation_revision, Some(2));

    let stale = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Stale Alias".to_owned(),
            presentation: DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Stale Alias".to_owned(),
                pinned: false,
                sort_order: 0,
                archived: false,
                expected_revision: Some(1),
            },
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("lost revision/update race"), "{stale}");

    let store = Store::open(&root).unwrap();
    let canonical = store.load_winds_session(&session_id).unwrap();
    let presentation = store
        .load_desktop_session_presentation(&session_id)
        .unwrap()
        .unwrap();
    assert_eq!(canonical.display_name, "Canonical Before");
    assert_eq!(presentation.display_alias, "Alias Two");
    assert_eq!(presentation.revision, 2);
    drop(store);
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
            display_name: "Renamed Alias".to_owned(),
            presentation: DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Renamed Alias".to_owned(),
                pinned: false,
                sort_order: 0,
                archived: false,
                expected_revision: None,
            },
        },
    )
    .unwrap();
    assert_eq!(renamed.canonical_session_id, session_id);
    assert_eq!(renamed.display_name, "Renamed Alias");
    assert_eq!(renamed.canonical_display_name, "Canonical Session");
    assert_eq!(renamed.presentation_revision, Some(1));

    let presented = desktop_bridge_update_session(
        &root,
        session_batch(DesktopBridgeSessionPresentationRequest {
            session_id: session_id.clone(),
            display_alias: "Pinned Alias".to_owned(),
            pinned: true,
            sort_order: 9,
            archived: false,
            expected_revision: Some(1),
        }),
    )
    .unwrap()
    .into_iter()
    .next()
    .unwrap();
    assert_eq!(presented.display_name, "Pinned Alias");
    assert_eq!(presented.presentation_revision, Some(2));

    let synchronized = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Unified Session Name".to_owned(),
            presentation: DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Unified Session Name".to_owned(),
                pinned: true,
                sort_order: 9,
                archived: false,
                expected_revision: Some(2),
            },
        },
    )
    .unwrap();
    assert_eq!(synchronized.display_name, "Unified Session Name");
    assert_eq!(synchronized.canonical_display_name, "Canonical Session");
    assert_eq!(
        synchronized.display_alias.as_deref(),
        Some("Unified Session Name")
    );
    assert_eq!(synchronized.presentation_revision, Some(3));

    let stale_rename = desktop_bridge_rename_session(
        &root,
        DesktopBridgeRenameSessionRequest {
            session_id: session_id.clone(),
            display_name: "Stale Rename".to_owned(),
            presentation: DesktopBridgeSessionPresentationRequest {
                session_id: session_id.clone(),
                display_alias: "Stale Rename".to_owned(),
                pinned: true,
                sort_order: 9,
                archived: false,
                expected_revision: Some(2),
            },
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
    assert_eq!(after_stale.display_name, "Canonical Session");

    let updated = desktop_bridge_update_session(
        &root,
        session_batch(DesktopBridgeSessionPresentationRequest {
            session_id: session_id.clone(),
            display_alias: "Updated Alias".to_owned(),
            pinned: false,
            sort_order: 4,
            archived: true,
            expected_revision: Some(3),
        }),
    )
    .unwrap()
    .into_iter()
    .next()
    .unwrap();
    assert_eq!(updated.presentation_revision, Some(4));
    assert!(updated.archived);

    let stale = desktop_bridge_update_session(
        &root,
        session_batch(DesktopBridgeSessionPresentationRequest {
            session_id: session_id.clone(),
            display_alias: "Stale Alias".to_owned(),
            pinned: false,
            sort_order: 5,
            archived: false,
            expected_revision: Some(1),
        }),
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
