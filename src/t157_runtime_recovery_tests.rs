use super::*;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::persistent_runtime::domain::{
    ContinuityClass, EndpointAvailability, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeAlias, RuntimeLifecycleEventKind,
};
use crate::persistent_runtime::persistence::PersistentRuntimeRecoveryReason;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

static NEXT_T157_RUNTIME_ROOT: AtomicU64 = AtomicU64::new(1);

fn test_root(label: &str) -> PathBuf {
    let sequence = NEXT_T157_RUNTIME_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t157-runtime-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn native_shell_profile(root: &Path) -> ShellProfile {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let candidate = "/bin/sh".to_owned();
    #[cfg(windows)]
    let candidate = std::env::var("COMSPEC").expect("Windows CI must provide COMSPEC");

    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.to_string_lossy().into_owned(),
        git_common_dir: root.to_string_lossy().into_owned(),
        shell_candidates: vec![candidate.clone()],
        detected_manifests: Vec::new(),
    };

    discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| {
            #[cfg(windows)]
            {
                profile.executable.eq_ignore_ascii_case(&candidate)
            }
            #[cfg(not(windows))]
            {
                profile.executable == candidate
            }
        })
        .expect("accepted native shell profile must be discoverable")
}

fn request_shell_exit(
    registry: &mut PersistentTerminalRegistry,
    attachment: &PersistentTerminalAttachment,
) {
    #[cfg(windows)]
    let bytes = b"exit\r\n";
    #[cfg(not(windows))]
    let bytes = b"exit\n";
    registry.send_input(attachment, bytes).unwrap();
}

#[test]
fn t157_unproven_cleanup_revokes_ownership_and_repeated_stop_never_blindly_retries() {
    let home = test_root("cleanup-unproven");
    let store = Store::open(&home).unwrap();
    let owner_generation_id = generation(0x41);
    store
        .record_persistent_runtime_owner_generation(owner_generation_id, 10)
        .unwrap();
    let profile = native_shell_profile(&home);
    let mut registry = PersistentTerminalRegistry::new(owner_generation_id);
    let attachment = registry
        .start_shell(
            &store,
            RuntimeAlias::new("cleanup-unproven").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            20,
        )
        .unwrap();

    request_shell_exit(&mut registry, &attachment);

    let error = registry
        .finish_owned(
            &store,
            &attachment,
            30,
            RuntimeLifecycleEventKind::RuntimeStopped,
            |_session| Err("forced unproven cleanup fixture".into()),
        )
        .unwrap_err();
    assert!(error.to_string().contains("live ownership was revoked"));

    let snapshot = registry.snapshot(&store, &attachment, 31).unwrap();
    assert_eq!(snapshot.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(snapshot.truth.process_liveness, ProcessLiveness::Unknown);
    assert_eq!(
        snapshot.truth.endpoint_availability,
        EndpointAvailability::Unknown
    );
    assert_eq!(snapshot.truth.continuity, ContinuityClass::Unknown);
    assert_eq!(
        snapshot.last_lifecycle_event_kind,
        RuntimeLifecycleEventKind::OwnershipLost
    );
    assert!(snapshot.exit.is_none());

    let stored = store
        .load_persistent_runtime_record(attachment.runtime_namespace_id())
        .unwrap();
    assert_eq!(stored.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(stored.truth.process_liveness, ProcessLiveness::Unknown);
    assert_eq!(stored.ownership_lost_unix_ms, Some(30));
    assert_eq!(
        stored.recovery_reason,
        Some(PersistentRuntimeRecoveryReason::OwnerGenerationUnproven)
    );

    let repeated = registry.terminate(&store, &attachment, 40).unwrap();
    assert_eq!(repeated.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(
        repeated.last_lifecycle_event_kind,
        RuntimeLifecycleEventKind::OwnershipLost
    );
    assert!(repeated.exit.is_none());

    thread::sleep(Duration::from_millis(50));
    drop(registry);
    drop(store);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t157_owner_shutdown_reports_each_runtime_independently_and_is_idempotent_for_terminal_state() {
    let home = test_root("shutdown-report");
    let store = Store::open(&home).unwrap();
    let owner_generation_id = generation(0x42);
    store
        .record_persistent_runtime_owner_generation(owner_generation_id, 10)
        .unwrap();
    let profile = native_shell_profile(&home);
    let mut registry = PersistentTerminalRegistry::new(owner_generation_id);

    let live = registry
        .start_shell(
            &store,
            RuntimeAlias::new("live-at-shutdown").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            20,
        )
        .unwrap();
    let terminal = registry
        .start_shell(
            &store,
            RuntimeAlias::new("already-terminal").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            21,
        )
        .unwrap();

    let terminal_snapshot = registry.terminate(&store, &terminal, 30).unwrap();
    assert_eq!(
        terminal_snapshot.truth.process_liveness,
        ProcessLiveness::Exited
    );

    let reports = registry.shutdown_all(&store, 300);
    assert_eq!(reports.len(), 2);

    let live_report = reports
        .iter()
        .find(|report| report.runtime_namespace_id == live.runtime_namespace_id())
        .unwrap();
    assert_eq!(
        live_report.disposition,
        PersistentRuntimeShutdownDisposition::Stopped
    );
    assert_eq!(
        live_report.snapshot.truth.ownership,
        OwnershipState::Unowned
    );
    assert_eq!(
        live_report.snapshot.truth.process_liveness,
        ProcessLiveness::Exited
    );
    assert!(live_report.error.is_none());

    let terminal_report = reports
        .iter()
        .find(|report| report.runtime_namespace_id == terminal.runtime_namespace_id())
        .unwrap();
    assert_eq!(
        terminal_report.disposition,
        PersistentRuntimeShutdownDisposition::AlreadyTerminal
    );
    assert_eq!(
        terminal_report.snapshot.truth.process_liveness,
        ProcessLiveness::Exited
    );
    assert!(terminal_report.error.is_none());

    let repeated = registry.shutdown_all(&store, 301);
    assert_eq!(repeated.len(), 2);
    assert!(repeated.iter().all(|report| {
        report.disposition == PersistentRuntimeShutdownDisposition::AlreadyTerminal
            && report.error.is_none()
    }));

    drop(registry);
    drop(store);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t157_runtime_recovery_has_no_pid_attach_or_process_enumeration_surface() {
    let runtime_source = include_str!("persistent_runtime/runtime.rs").to_ascii_lowercase();
    let persistence_source = include_str!("persistent_runtime/persistence.rs").to_ascii_lowercase();
    for forbidden in [
        "process_id(",
        "command::new",
        "sysinfo",
        "/proc/",
        "openprocess",
        "process32first",
        "process32next",
        "kill(pid",
        "pidfile",
    ] {
        assert!(!runtime_source.contains(forbidden), "{forbidden}");
        assert!(!persistence_source.contains(forbidden), "{forbidden}");
    }
}
