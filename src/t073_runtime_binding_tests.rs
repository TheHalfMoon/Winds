use crate::agentic_runtime::{
    AgentExecutionObservation, RuntimeBindingOwnership, RuntimeDiscovery, RuntimeKind,
    RuntimeResumeResolution, RuntimeVersionState, SafeVersionObservation,
    discover_runtime_from_safe_observations,
};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::ffi::OsStr;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t073-{name}-{}-{sequence}",
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
        .is_some_and(|name| name.starts_with("winds-t073-"));
    assert!(canonical_root.starts_with(&canonical_temp));
    assert!(owned_name);
    fs::remove_dir_all(&canonical_root).unwrap();
}

fn fake_executable_path(root: &Path, stem: &str) -> PathBuf {
    #[cfg(windows)]
    {
        root.join(format!("{stem}.exe"))
    }
    #[cfg(not(windows))]
    {
        root.join(stem)
    }
}

fn create_fake_executable(root: &Path, stem: &str, bytes: &[u8]) -> PathBuf {
    let path = fake_executable_path(root, stem);
    fs::write(&path, bytes).unwrap();
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }
    path
}

fn discover_present(runtime: RuntimeKind, executable: &Path, version: &str) -> RuntimeDiscovery {
    let discovery = discover_runtime_from_safe_observations(
        runtime,
        executable,
        SafeVersionObservation::Observed(version.to_owned()),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );
    discovery
}

fn seed_store(home: &Path) -> Store {
    let store = Store::open(home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/fixture/workspace",
                git_common_dir: "/fixture/workspace/.git",
            },
            10,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "Task",
            },
            20,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "Planner",
            },
            30,
        )
        .unwrap();
    store
}

#[test]
fn exact_runtime_binding_persists_provenance_and_survives_reopen() {
    let root = test_root("persist-reopen");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");
    let discovery = discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");
    let expected_executable = discovery.executable.clone().unwrap();
    let expected_version = discovery.version.clone();

    let store = seed_store(&root);
    store
        .create_runtime_session_binding(
            "binding-1",
            "session-1",
            &discovery,
            Some("native-thread-1"),
            40,
        )
        .unwrap();
    drop(store);

    let reopened = Store::open(&root).unwrap();
    let binding = reopened.load_runtime_session_binding("binding-1").unwrap();
    assert_eq!(binding.binding_id, "binding-1");
    assert_eq!(binding.session_id, "session-1");
    assert_eq!(binding.runtime, RuntimeKind::Codex);
    assert_eq!(binding.executable, expected_executable);
    assert_eq!(binding.version, expected_version);
    assert_eq!(
        binding.native_session_id.as_deref(),
        Some("native-thread-1")
    );
    assert_eq!(binding.ownership, RuntimeBindingOwnership::Unproven);
    assert_eq!(binding.ownership_observed_unix_ms, None);
    assert_eq!(binding.bound_unix_ms, 40);

    match reopened
        .resolve_runtime_resume_candidate("session-1", &discovery)
        .unwrap()
    {
        RuntimeResumeResolution::Candidate(candidate) => {
            assert_eq!(candidate.binding_id, "binding-1");
            assert_eq!(candidate.ownership, RuntimeBindingOwnership::Unproven);
        }
        other => panic!("expected exact future resume candidate, got {other:?}"),
    }

    drop(reopened);
    cleanup_owned_root(&root);
}

#[test]
fn executable_or_version_drift_makes_persisted_mapping_stale() {
    let root = test_root("stale-drift");
    let original = b"fixture-codex-v1\n";
    let replacement = b"fixture-codex-x1\n";
    assert_eq!(original.len(), replacement.len());
    let executable = create_fake_executable(&root, "fixture-codex", original);
    let discovery_v1 = discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");

    let store = seed_store(&root);
    store
        .create_runtime_session_binding(
            "binding-1",
            "session-1",
            &discovery_v1,
            Some("native-thread-1"),
            40,
        )
        .unwrap();

    fs::write(&executable, replacement).unwrap();
    let replaced_discovery =
        discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");
    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &replaced_discovery)
            .unwrap(),
        RuntimeResumeResolution::Stale
    );

    fs::write(&executable, original).unwrap();
    let changed_version =
        discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.4-fixture");
    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &changed_version)
            .unwrap(),
        RuntimeResumeResolution::Stale
    );

    drop(store);
    cleanup_owned_root(&root);
}

#[test]
fn missing_or_non_present_mapping_is_unavailable_not_resumed() {
    let root = test_root("missing-native");
    let executable = create_fake_executable(&root, "fixture-claude", b"fixture-claude-v1\n");
    let discovery = discover_present(RuntimeKind::Claude, &executable, "claude-code 1.0-fixture");
    let store = seed_store(&root);

    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &discovery)
            .unwrap(),
        RuntimeResumeResolution::Unavailable
    );

    store
        .create_runtime_session_binding("binding-no-native", "session-1", &discovery, None, 40)
        .unwrap();
    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &discovery)
            .unwrap(),
        RuntimeResumeResolution::Unavailable
    );

    let unsupported = discover_runtime_from_safe_observations(
        RuntimeKind::Claude,
        &executable,
        SafeVersionObservation::Unsupported("claude-code unsupported-fixture".to_owned()),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &unsupported)
            .unwrap(),
        RuntimeResumeResolution::Unavailable
    );

    let version_unavailable = discover_runtime_from_safe_observations(
        RuntimeKind::Claude,
        &executable,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &version_unavailable)
            .unwrap(),
        RuntimeResumeResolution::Unavailable
    );

    let absent_executable = fake_executable_path(&root, "missing-claude");
    let unavailable = discover_runtime_from_safe_observations(
        RuntimeKind::Claude,
        &absent_executable,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        store
            .resolve_runtime_resume_candidate("session-1", &unavailable)
            .unwrap(),
        RuntimeResumeResolution::Unavailable
    );

    drop(store);
    cleanup_owned_root(&root);
}

#[test]
fn multiple_exact_native_mappings_are_deterministically_ambiguous() {
    let root = test_root("ambiguous");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");
    let discovery = discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");
    let store = seed_store(&root);

    for (binding_id, native_session_id) in [
        ("binding-b", "native-thread-b"),
        ("binding-a", "native-thread-a"),
    ] {
        store
            .create_runtime_session_binding(
                binding_id,
                "session-1",
                &discovery,
                Some(native_session_id),
                40,
            )
            .unwrap();
    }

    match store
        .resolve_runtime_resume_candidate("session-1", &discovery)
        .unwrap()
    {
        RuntimeResumeResolution::Ambiguous(candidates) => {
            assert_eq!(candidates.len(), 2);
            assert_eq!(candidates[0].binding_id, "binding-a");
            assert_eq!(candidates[1].binding_id, "binding-b");
        }
        other => panic!("expected deterministic ambiguity, got {other:?}"),
    }

    drop(store);
    cleanup_owned_root(&root);
}

#[test]
fn ownership_loss_is_durable_and_native_id_never_recreates_live_truth() {
    let root = test_root("ownership-lost");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");
    let discovery = discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");
    let store = seed_store(&root);
    store
        .create_runtime_session_binding(
            "binding-1",
            "session-1",
            &discovery,
            Some("native-thread-1"),
            40,
        )
        .unwrap();
    store
        .mark_runtime_binding_ownership_lost("binding-1", 60)
        .unwrap();
    store
        .mark_runtime_binding_ownership_lost("binding-1", 60)
        .unwrap();
    store
        .mark_runtime_binding_ownership_lost("binding-1", 70)
        .unwrap();
    assert!(
        store
            .mark_runtime_binding_ownership_lost("binding-1", 59)
            .is_err()
    );
    assert!(
        store
            .mark_runtime_binding_ownership_lost("binding-1", 39)
            .is_err()
    );
    let before_reopen = store.load_runtime_session_binding("binding-1").unwrap();
    assert_eq!(
        before_reopen.ownership,
        RuntimeBindingOwnership::OwnershipLost
    );
    assert_eq!(before_reopen.ownership_observed_unix_ms, Some(60));
    drop(store);

    let reopened = Store::open(&root).unwrap();
    let binding = reopened.load_runtime_session_binding("binding-1").unwrap();
    assert_eq!(binding.ownership, RuntimeBindingOwnership::OwnershipLost);
    assert_eq!(binding.ownership_observed_unix_ms, Some(60));
    assert_eq!(
        binding.native_session_id.as_deref(),
        Some("native-thread-1")
    );

    match reopened
        .resolve_runtime_resume_candidate("session-1", &discovery)
        .unwrap()
    {
        RuntimeResumeResolution::Candidate(candidate) => {
            assert_eq!(candidate.ownership, RuntimeBindingOwnership::OwnershipLost);
            assert_eq!(
                candidate.native_session_id.as_deref(),
                Some("native-thread-1")
            );
        }
        other => panic!("durable native ID should remain only a resume candidate, got {other:?}"),
    }

    drop(reopened);
    cleanup_owned_root(&root);
}

#[test]
fn exact_native_identity_cannot_alias_multiple_winds_sessions() {
    let root = test_root("native-alias");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");
    let discovery = discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");
    let store = seed_store(&root);
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-2",
                workstream_id: "workstream-1",
                display_name: "Reviewer",
            },
            31,
        )
        .unwrap();
    store
        .create_runtime_session_binding(
            "binding-1",
            "session-1",
            &discovery,
            Some("native-thread-1"),
            40,
        )
        .unwrap();
    let alias = store.create_runtime_session_binding(
        "binding-2",
        "session-2",
        &discovery,
        Some("native-thread-1"),
        41,
    );
    assert!(alias.is_err());
    assert!(store.load_runtime_session_binding("binding-2").is_err());

    drop(store);
    cleanup_owned_root(&root);
}

#[test]
fn invalid_binding_facts_and_schema_identity_expansion_fail_closed() {
    let root = test_root("fail-closed");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");
    let supported = discover_present(RuntimeKind::Codex, &executable, "codex-cli 1.2.3-fixture");
    let unsupported = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Unsupported("codex-cli old-fixture".to_owned()),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(unsupported.version.state, RuntimeVersionState::Unsupported);

    let store = seed_store(&root);
    assert!(
        store
            .create_runtime_session_binding(
                "binding-unsupported",
                "session-1",
                &unsupported,
                Some("native-thread-1"),
                40,
            )
            .is_err()
    );
    assert!(
        store
            .create_runtime_session_binding(
                "binding-unknown-session",
                "missing-session",
                &supported,
                Some("native-thread-1"),
                40,
            )
            .is_err()
    );
    assert!(
        store
            .create_runtime_session_binding(
                "binding-negative-time",
                "session-1",
                &supported,
                Some("native-thread-1"),
                -1,
            )
            .is_err()
    );
    let mut relative = supported.clone();
    let relative_executable = relative
        .executable
        .as_mut()
        .expect("present discovery has executable identity");
    relative_executable.observed_path = PathBuf::from("relative-codex");
    relative_executable.canonical_path = PathBuf::from("relative-codex");
    assert!(
        store
            .create_runtime_session_binding(
                "binding-relative-path",
                "session-1",
                &relative,
                Some("native-thread-relative"),
                40,
            )
            .is_err()
    );
    assert!(
        store
            .load_runtime_session_binding("binding-relative-path")
            .is_err()
    );
    drop(store);

    let connection = rusqlite::Connection::open(root.join("winds.db")).unwrap();
    let columns = {
        let mut statement = connection
            .prepare("PRAGMA table_info(runtime_session_bindings)")
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    for forbidden in [
        "workspace_id",
        "workstream_id",
        "pid",
        "model",
        "provider",
        "live",
    ] {
        assert!(
            !columns
                .iter()
                .any(|column| column.to_ascii_lowercase().contains(forbidden)),
            "unexpected T073 schema expansion: {forbidden}"
        );
    }
    let foreign_targets = {
        let mut statement = connection
            .prepare("PRAGMA foreign_key_list(runtime_session_bindings)")
            .unwrap();
        statement
            .query_map([], |row| row.get::<_, String>(2))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    };
    assert_eq!(foreign_targets, vec!["winds_sessions"]);
    drop(connection);

    cleanup_owned_root(&root);
}