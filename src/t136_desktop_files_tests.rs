use crate::desktop_files::{
    DesktopBoundDockRequest, DesktopFilePreviewRequest, DesktopFilePreviewState,
    DesktopRightDockTarget, desktop_right_dock_bind, desktop_right_dock_changes,
    desktop_right_dock_files, desktop_right_dock_preview_file,
    desktop_right_dock_preview_file_with_open_hook,
};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    home: PathBuf,
    repo: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn fixture(name: &str) -> Fixture {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t136-{name}-{}-{sequence}",
        std::process::id()
    ));
    let repo = root.join("repo");
    let home = root.join("state");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "t136@example.invalid"]);
    git(&repo, &["config", "user.name", "T136 Fixture"]);
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(repo.join("README.md"), "hello from Winds\n").unwrap();
    fs::write(repo.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-qm", "fixture"]);

    let canonical_repo = repo.canonicalize().unwrap();
    let common = canonical_repo.join(".git").canonicalize().unwrap();
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-t136",
                canonical_worktree_root: canonical_repo.to_str().unwrap(),
                git_common_dir: common.to_str().unwrap(),
            },
            100,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-t136",
                workspace_id: "workspace-t136",
                display_name: "T136 workstream",
            },
            101,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-t136",
                workstream_id: "workstream-t136",
                display_name: "T136 session",
            },
            102,
        )
        .unwrap();
    drop(store);
    Fixture {
        root,
        home,
        repo: canonical_repo,
    }
}

fn bind(fixture: &Fixture) -> crate::desktop_files::DesktopRightDockBinding {
    desktop_right_dock_bind(
        &fixture.home,
        DesktopRightDockTarget {
            workspace_id: "workspace-t136".to_owned(),
            session_id: "session-t136".to_owned(),
        },
    )
    .unwrap()
}

#[test]
fn files_and_preview_are_exact_binding_read_only_projections() {
    let fixture = fixture("files-preview");
    let binding = bind(&fixture);
    assert_eq!(binding.binding_digest.len(), 64);
    assert!(binding.head_oid.is_some());
    assert!(binding.tree_oid.is_some());

    let files = desktop_right_dock_files(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap();
    assert_eq!(files.binding, binding);
    assert!(!files.truncated);
    assert!(files.entries.iter().any(|entry| entry.path == "README.md"));
    assert!(files.entries.iter().any(|entry| entry.path == "src/lib.rs"));

    let preview = desktop_right_dock_preview_file(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding,
            path: "README.md".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(preview.state, DesktopFilePreviewState::Text);
    assert_eq!(preview.content.as_deref(), Some("hello from Winds\n"));
}

#[test]
fn stale_or_tampered_binding_fails_closed() {
    let fixture = fixture("stale-binding");
    let binding = bind(&fixture);
    fs::write(fixture.repo.join("README.md"), "changed after binding\n").unwrap();
    let stale = desktop_right_dock_files(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(stale.contains("binding is stale"), "{stale}");

    let fresh = bind(&fixture);
    let mut tampered = fresh;
    tampered.session_id = "forged-session".to_owned();
    let rejected =
        desktop_right_dock_files(&fixture.home, DesktopBoundDockRequest { binding: tampered })
            .unwrap_err()
            .to_string();
    assert!(rejected.contains("binding digest is invalid"), "{rejected}");
}

#[test]
fn changes_come_from_git_and_never_claim_verification() {
    let fixture = fixture("changes");
    fs::write(fixture.repo.join("README.md"), "working tree change\n").unwrap();
    fs::write(fixture.repo.join("new.txt"), "untracked\n").unwrap();
    let binding = bind(&fixture);
    let response =
        desktop_right_dock_changes(&fixture.home, DesktopBoundDockRequest { binding }).unwrap();
    assert!(
        response
            .entries
            .iter()
            .any(|entry| entry.path == "README.md" && entry.status.contains('M'))
    );
    assert!(
        response
            .entries
            .iter()
            .any(|entry| entry.path == "new.txt" && entry.status == "??")
    );
    assert!(response.diff.contains("working tree change"));
    assert!(!response.diff.to_ascii_lowercase().contains("verified=true"));
}

#[test]
fn files_skip_tracked_paths_deleted_from_the_bound_worktree() {
    let fixture = fixture("deleted-tracked-file");
    fs::remove_file(fixture.repo.join("README.md")).unwrap();
    let binding = bind(&fixture);
    let files = desktop_right_dock_files(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap();
    assert!(!files.entries.iter().any(|entry| entry.path == "README.md"));
    assert!(files.entries.iter().any(|entry| entry.path == "src/lib.rs"));

    let changes =
        desktop_right_dock_changes(&fixture.home, DesktopBoundDockRequest { binding }).unwrap();
    assert!(
        changes
            .entries
            .iter()
            .any(|entry| entry.path == "README.md" && entry.status.contains('D'))
    );
}

#[test]
fn traversal_binary_and_large_preview_states_are_bounded() {
    let fixture = fixture("preview-safety");
    fs::write(fixture.repo.join("binary.bin"), [0_u8, 1, 2, 3]).unwrap();
    fs::write(fixture.repo.join("large.txt"), vec![b'x'; 256 * 1024 + 1]).unwrap();
    let binding = bind(&fixture);

    let traversal = desktop_right_dock_preview_file(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding: binding.clone(),
            path: "../outside".to_owned(),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(traversal.contains("traversal"), "{traversal}");

    let binary = desktop_right_dock_preview_file(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding: binding.clone(),
            path: "binary.bin".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(binary.state, DesktopFilePreviewState::Binary);
    assert!(binary.content.is_none());

    let large = desktop_right_dock_preview_file(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding,
            path: "large.txt".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(large.state, DesktopFilePreviewState::TooLarge);
    assert!(large.content.is_none());
}

#[cfg(unix)]
#[test]
fn symlink_preview_fails_closed_even_when_git_lists_the_path() {
    use std::os::unix::fs::symlink;
    let fixture = fixture("symlink");
    let outside = fixture.root.join("outside.txt");
    fs::write(&outside, "outside\n").unwrap();
    symlink(&outside, fixture.repo.join("escape-link")).unwrap();
    let binding = bind(&fixture);
    let files = desktop_right_dock_files(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap();
    assert!(
        files
            .entries
            .iter()
            .any(|entry| entry.path == "escape-link")
    );
    let error = desktop_right_dock_preview_file(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding,
            path: "escape-link".to_owned(),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("refuses symlink"), "{error}");
}

#[cfg(unix)]
#[test]
fn regular_file_to_symlink_swap_during_preview_fails_closed() {
    use std::os::unix::fs::symlink;

    let fixture = fixture("preview-swap");
    let preview_path = fixture.repo.join("swap.txt");
    let outside = fixture.root.join("outside-secret.txt");
    fs::write(&preview_path, "inside\n").unwrap();
    fs::write(&outside, "outside secret must never render\n").unwrap();
    let binding = bind(&fixture);

    let preview_path_for_swap = preview_path.clone();
    let outside_for_swap = outside.clone();
    let error = desktop_right_dock_preview_file_with_open_hook(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding,
            path: "swap.txt".to_owned(),
        },
        move || {
            fs::remove_file(&preview_path_for_swap).unwrap();
            symlink(&outside_for_swap, &preview_path_for_swap).unwrap();
        },
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("refuses symlink"), "{error}");
    assert!(!error.contains("outside secret must never render"));
}

#[cfg(windows)]
#[test]
fn windows_preview_root_handle_supports_attribute_revalidation() {
    let fixture = fixture("windows-root-access");
    let binding = bind(&fixture);
    let preview = desktop_right_dock_preview_file(
        &fixture.home,
        DesktopFilePreviewRequest {
            binding,
            path: "README.md".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(preview.state, DesktopFilePreviewState::Text);
    assert_eq!(preview.content.as_deref(), Some("hello from Winds\n"));
}

#[test]
fn canonical_git_root_mismatch_is_rejected_before_projection() {
    let fixture = fixture("root-mismatch");
    let store = Store::open(&fixture.home).unwrap();
    let fake_common = fixture.root.join("fake-common");
    fs::create_dir_all(&fake_common).unwrap();
    store
        .connection
        .execute(
            "UPDATE workspaces SET git_common_dir = ?1 WHERE workspace_id = 'workspace-t136'",
            rusqlite::params![fake_common.to_str().unwrap()],
        )
        .unwrap();
    drop(store);
    let error = desktop_right_dock_bind(
        &fixture.home,
        DesktopRightDockTarget {
            workspace_id: "workspace-t136".to_owned(),
            session_id: "session-t136".to_owned(),
        },
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("Git common directory"), "{error}");
}

#[test]
fn binding_includes_available_canonical_workflow_stage_and_candidate_identity() {
    let fixture = fixture("candidate-binding");
    let store = Store::open(&fixture.home).unwrap();
    store.connection.execute(
        "INSERT INTO workflow_runs(
            workflow_run_id, workspace_id, workstream_id, schema_version, terminal_state, created_unix_ms
         ) VALUES ('workflow-t136', 'workspace-t136', 'workstream-t136', 1, NULL, 110)",
        [],
    ).unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_stage_runs(
            stage_run_id, workflow_run_id, stage_key, attempt_ordinal,
            predecessor_stage_run_id, relation_kind, lifecycle_state, failure_class,
            checkpoint_identity, material_progress_basis, outcome_reason,
            last_transition_operation_id, last_transition_from_state,
            last_transition_source, last_transition_authority,
            created_unix_ms, updated_unix_ms
         ) VALUES (
            'stage-t136', 'workflow-t136', 'inspect', 1,
            NULL, NULL, 'PREPARED', NULL, NULL, NULL, NULL,
            NULL, NULL, NULL, NULL, 111, 111
         )",
            [],
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
            binding_id, stage_run_id, winds_session_id, runtime_binding_id,
            continuation_class, bound_unix_ms
         ) VALUES ('actor-t136', 'stage-t136', 'session-t136', NULL, 'UNPROVEN', 112)",
            [],
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_artifact_baselines(
            baseline_id, stage_run_id, baseline_kind, stable_reference,
            candidate_oid, candidate_tree, created_unix_ms
         ) VALUES (
            'baseline-t136', 'stage-t136', 'EXACT_GIT_CANDIDATE', 'fixture',
            '1111111111111111111111111111111111111111',
            '2222222222222222222222222222222222222222', 113
         )",
            [],
        )
        .unwrap();
    drop(store);

    let binding = bind(&fixture);
    assert_eq!(binding.workflow_run_id.as_deref(), Some("workflow-t136"));
    assert_eq!(binding.stage_run_id.as_deref(), Some("stage-t136"));
    assert_eq!(
        binding.candidate_oid.as_deref(),
        Some("1111111111111111111111111111111111111111")
    );
    assert_eq!(
        binding.candidate_tree.as_deref(),
        Some("2222222222222222222222222222222222222222")
    );
}
