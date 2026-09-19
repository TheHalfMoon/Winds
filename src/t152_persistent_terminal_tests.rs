use super::*;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::persistent_runtime::domain::{
    ContinuityClass, EndpointAvailability, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeAlias, RuntimeLifecycleEventKind,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

fn test_root(label: &str) -> PathBuf {
    let sequence = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t152-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn short_runtime_root() -> PathBuf {
    let sequence = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("w152r-{sequence}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
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

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn start_owner(home: &Path, runtime_root: &Path, now_unix_ms: i64) -> PersistentOwner {
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .unwrap();
    PersistentOwner::start_for_test(home, &runtime_directory, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn start_owner(home: &Path, _runtime_root: &Path, now_unix_ms: i64) -> PersistentOwner {
    PersistentOwner::start(home, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn prime_headless_windows_terminal(
    owner: &mut PersistentOwner,
    attachment: &PersistentTerminalAttachment,
) {
    const CURSOR_QUERY: &[u8] = b"\x1b[6n";
    const CURSOR_RESPONSE: &[u8] = b"\x1b[1;1R";
    let mut observed = Vec::new();
    for _ in 0..32 {
        let mut buffer = [0_u8; 4096];
        let count = owner
            .read_terminal_runtime_output(attachment, &mut buffer)
            .unwrap();
        if count == 0 {
            break;
        }
        observed.extend_from_slice(&buffer[..count]);
        if observed
            .windows(CURSOR_QUERY.len())
            .any(|window| window == CURSOR_QUERY)
        {
            owner
                .send_terminal_runtime_input(attachment, CURSOR_RESPONSE)
                .unwrap();
            return;
        }
        assert!(observed.len() <= 128 * 1024);
    }
    panic!(
        "headless native-Windows ConPTY did not request cursor position; observed {:?}",
        String::from_utf8_lossy(&observed)
    );
}

#[cfg(not(windows))]
fn prime_headless_windows_terminal(
    _owner: &mut PersistentOwner,
    _attachment: &PersistentTerminalAttachment,
) {
}

fn send_exit(owner: &mut PersistentOwner, attachment: &PersistentTerminalAttachment) {
    #[cfg(windows)]
    let bytes = b"exit\r\n";
    #[cfg(not(windows))]
    let bytes = b"exit\n";
    owner
        .send_terminal_runtime_input(attachment, bytes)
        .unwrap();
}

fn wait_for_detached_exit(
    owner: &mut PersistentOwner,
    start_unix_ms: i64,
    start_monotonic_ms: u64,
) {
    for offset in 0..200_u64 {
        let observed = owner
            .poll_terminal_runtimes(
                start_unix_ms + i64::try_from(offset).unwrap(),
                start_monotonic_ms + offset,
            )
            .unwrap();
        if observed > 0 {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("detached persistent terminal did not report process exit inside bounded test window");
}

fn cleanup(home: PathBuf, runtime_root: PathBuf) {
    fs::remove_dir_all(home).unwrap();
    let _ = fs::remove_dir_all(runtime_root);
}

#[test]
fn t152_long_running_shell_survives_complete_attachment_drop_and_exact_reattach() {
    let home = test_root("detach-reattach");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let runtime_root = short_runtime_root();
    #[cfg(windows)]
    let runtime_root = test_root("windows-runtime-placeholder");

    let profile = native_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root, 10);
    let alias = RuntimeAlias::new("persistent-shell").unwrap();
    let size = TerminalSize { rows: 24, cols: 80 };
    let mut attachment = Some(
        owner
            .start_terminal_runtime(alias.clone(), &profile, &home, size, 20, 100)
            .unwrap(),
    );
    prime_headless_windows_terminal(&mut owner, attachment.as_ref().unwrap());

    let runtime_namespace_id = attachment.as_ref().unwrap().runtime_namespace_id();
    let owner_generation_id = attachment.as_ref().unwrap().owner_generation_id();
    let first = owner
        .terminal_runtime_snapshot(attachment.as_ref().unwrap(), 21, 101)
        .unwrap();

    assert_eq!(first.runtime_namespace_id, runtime_namespace_id);
    assert_eq!(first.owner_generation_id, owner_generation_id);
    assert_eq!(first.runtime_alias, alias);
    assert_eq!(first.truth.ownership, OwnershipState::LiveOwned);
    assert_eq!(first.truth.process_liveness, ProcessLiveness::Running);
    assert_eq!(first.truth.continuity, ContinuityClass::RetainedLiveProcess);
    assert_eq!(
        first.truth.endpoint_availability,
        EndpointAvailability::Available
    );
    assert_eq!(owner.live_terminal_runtime_count(), 1);
    assert!(!owner.should_exit(OWNER_IDLE_GRACE_MS + 100));

    assert!(attachment.take().is_some());
    thread::sleep(Duration::from_millis(50));

    let reattached = owner
        .reattach_terminal_runtime(runtime_namespace_id, owner_generation_id, 22, 102)
        .unwrap();
    let second = owner
        .terminal_runtime_snapshot(&reattached, 23, 103)
        .unwrap();

    assert_eq!(second.runtime_namespace_id, first.runtime_namespace_id);
    assert_eq!(second.owner_generation_id, first.owner_generation_id);
    assert_eq!(second.terminal_session_id, first.terminal_session_id);
    assert_eq!(second.truth.ownership, OwnershipState::LiveOwned);
    assert_eq!(second.truth.process_liveness, ProcessLiveness::Running);

    let resized = TerminalSize {
        rows: 40,
        cols: 120,
    };
    owner.resize_terminal_runtime(&reattached, resized).unwrap();
    assert_eq!(owner.terminal_runtime_size(&reattached).unwrap(), resized);

    let final_snapshot = owner
        .terminate_terminal_runtime(&reattached, 24, 104)
        .unwrap();
    assert_eq!(final_snapshot.truth.ownership, OwnershipState::Unowned);
    assert_eq!(
        final_snapshot.truth.process_liveness,
        ProcessLiveness::Exited
    );
    assert_eq!(
        final_snapshot.last_lifecycle_event_kind,
        RuntimeLifecycleEventKind::RuntimeStopped
    );
    assert!(final_snapshot.exit.is_some());
    assert_eq!(owner.live_terminal_runtime_count(), 0);

    drop(owner);
    cleanup(home, runtime_root);
}

#[test]
fn t152_process_exit_while_fully_detached_is_observed_and_persisted() {
    let home = test_root("detached-exit");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let runtime_root = short_runtime_root();
    #[cfg(windows)]
    let runtime_root = test_root("windows-runtime-placeholder-exit");

    let profile = native_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root, 30);
    let mut attachment = Some(
        owner
            .start_terminal_runtime(
                RuntimeAlias::new("detached-exit-shell").unwrap(),
                &profile,
                &home,
                TerminalSize { rows: 24, cols: 80 },
                31,
                200,
            )
            .unwrap(),
    );
    prime_headless_windows_terminal(&mut owner, attachment.as_ref().unwrap());
    let runtime_namespace_id = attachment.as_ref().unwrap().runtime_namespace_id();
    let owner_generation_id = attachment.as_ref().unwrap().owner_generation_id();

    send_exit(&mut owner, attachment.as_ref().unwrap());
    assert!(attachment.take().is_some());
    wait_for_detached_exit(&mut owner, 32, 201);

    assert_eq!(owner.live_terminal_runtime_count(), 0);
    let stored = owner
        .store
        .load_persistent_runtime_record(runtime_namespace_id)
        .unwrap();
    assert_eq!(stored.owner_generation_id, Some(owner_generation_id));
    assert_eq!(stored.truth.ownership, OwnershipState::Unowned);
    assert_eq!(stored.truth.process_liveness, ProcessLiveness::Exited);
    assert_eq!(
        stored.truth.continuity,
        ContinuityClass::RetainedLiveProcess
    );
    assert_eq!(
        stored.last_lifecycle_event_kind,
        RuntimeLifecycleEventKind::ProcessStateObserved
    );
    assert!(stored.last_observed_unix_ms.is_some());

    let error = owner
        .reattach_terminal_runtime(runtime_namespace_id, owner_generation_id, 400, 400)
        .unwrap_err();
    assert!(error.to_string().contains("not live-owned"));

    drop(owner);
    cleanup(home, runtime_root);
}

#[test]
fn t152_stale_generation_cannot_reattach_or_redirect_live_owned_terminal() {
    let home = test_root("stale-generation");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let runtime_root = short_runtime_root();
    #[cfg(windows)]
    let runtime_root = test_root("windows-runtime-placeholder-stale");

    let profile = native_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root, 50);
    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("generation-bound-shell").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            51,
            300,
        )
        .unwrap();
    prime_headless_windows_terminal(&mut owner, &attachment);

    let runtime_namespace_id = attachment.runtime_namespace_id();
    let live_generation = attachment.owner_generation_id();
    let mut stale_generation = OwnerGenerationId::from_entropy_bytes([0x7e; 16]).unwrap();
    if stale_generation == live_generation {
        stale_generation = OwnerGenerationId::from_entropy_bytes([0x7d; 16]).unwrap();
    }

    let error = owner
        .reattach_terminal_runtime(runtime_namespace_id, stale_generation, 52, 301)
        .unwrap_err();
    assert!(error.to_string().contains("owner generation"));
    assert_eq!(owner.live_terminal_runtime_count(), 1);

    let snapshot = owner
        .terminal_runtime_snapshot(&attachment, 53, 302)
        .unwrap();
    assert_eq!(snapshot.owner_generation_id, live_generation);
    assert_eq!(snapshot.truth.ownership, OwnershipState::LiveOwned);

    owner.close_terminal_runtime(&attachment, 54, 303).unwrap();
    drop(owner);
    cleanup(home, runtime_root);
}

#[cfg(windows)]
#[test]
fn t152_native_windows_interrupt_remains_explicitly_fail_closed() {
    let home = test_root("windows-interrupt");
    let runtime_root = test_root("windows-runtime-placeholder-interrupt");
    let profile = native_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root, 60);
    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("windows-interrupt-shell").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            61,
            400,
        )
        .unwrap();
    prime_headless_windows_terminal(&mut owner, &attachment);

    let error = owner.interrupt_terminal_runtime(&attachment).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("interrupt is unsupported on native Windows")
    );

    let snapshot = owner
        .terminal_runtime_snapshot(&attachment, 62, 401)
        .unwrap();
    assert_eq!(snapshot.truth.ownership, OwnershipState::LiveOwned);
    assert_eq!(snapshot.truth.process_liveness, ProcessLiveness::Running);

    owner
        .terminate_terminal_runtime(&attachment, 63, 402)
        .unwrap();
    drop(owner);
    cleanup(home, runtime_root);
}

#[test]
fn t152_runtime_layer_has_no_pid_reconstruction_provider_launch_or_second_terminal_backend() {
    let runtime_source = include_str!("persistent_runtime/runtime.rs");
    let owner_source = include_str!("persistent_runtime/owner.rs");
    for prohibited in [
        "process_id(",
        "std::process::Command",
        "Command::new",
        "portable_pty",
        "native_pty_system",
        "ConPty",
        "codex",
        "claude",
        "provider_id",
        "model_id",
    ] {
        assert!(!runtime_source.contains(prohibited), "{prohibited}");
    }
    for prohibited in [
        "process_id(",
        "Command::new",
        "portable_pty",
        "native_pty_system",
    ] {
        assert!(!owner_source.contains(prohibited), "{prohibited}");
    }
}
