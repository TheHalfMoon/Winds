use super::*;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::persistent_runtime::domain::{
    ClientAuthority, LifecycleProofClass, RuntimeAlias, RuntimeLifecycleEventKind,
};
use crate::persistent_runtime::protocol::ProtocolPayload;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

static NEXT_T154_OWNER_ROOT: AtomicU64 = AtomicU64::new(1);

fn t154_owner_root(label: &str) -> PathBuf {
    let sequence = NEXT_T154_OWNER_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t154-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn t154_runtime_root() -> PathBuf {
    let sequence = NEXT_T154_OWNER_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("w154r-{sequence}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

fn t154_shell_profile(root: &Path) -> ShellProfile {
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
fn t154_start_owner(home: &Path, runtime_root: &Path, now_unix_ms: i64) -> PersistentOwner {
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .unwrap();
    PersistentOwner::start_for_test(home, &runtime_directory, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn t154_start_owner(home: &Path, _runtime_root: &Path, now_unix_ms: i64) -> PersistentOwner {
    PersistentOwner::start(home, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn t154_prime_windows_terminal(
    owner: &mut PersistentOwner,
    attachment: &PersistentTerminalAttachment,
) {
    const CURSOR_QUERY: &[u8] = b"[6n";
    const CURSOR_RESPONSE: &[u8] = b"[1;1R";
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
fn t154_prime_windows_terminal(
    _owner: &mut PersistentOwner,
    _attachment: &PersistentTerminalAttachment,
) {
}

fn t154_wait_for_output(
    owner: &mut PersistentOwner,
    attachment: &PersistentTerminalAttachment,
    marker: &[u8],
) {
    let mut observed = Vec::new();
    for _ in 0..96 {
        let mut buffer = [0_u8; 4096];
        let count = owner
            .read_terminal_runtime_output(attachment, &mut buffer)
            .unwrap();
        if count > 0 {
            observed.extend_from_slice(&buffer[..count]);
            if observed
                .windows(marker.len())
                .any(|window| window == marker)
            {
                return;
            }
        }
        assert!(observed.len() <= 128 * 1024);
        thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "persistent terminal marker {:?} not observed; output {:?}",
        String::from_utf8_lossy(marker),
        String::from_utf8_lossy(&observed)
    );
}

fn t154_enter_terminable_workload(
    owner: &mut PersistentOwner,
    client: &ClientConnectionId,
    runtime_namespace_id: RuntimeNamespaceId,
    attachment: &PersistentTerminalAttachment,
    now_unix_ms: i64,
    now_monotonic_ms: u64,
) {
    #[cfg(windows)]
    let bytes = b"echo WINDS_T154_READY
set /p WINDS_T154_BLOCK=
"
    .as_slice();
    #[cfg(not(windows))]
    let bytes = b"printf 'WINDS_T154_READY\n'; exec sleep 30
"
    .as_slice();

    owner
        .controller_send_terminal_input(
            client,
            runtime_namespace_id,
            bytes,
            now_unix_ms,
            now_monotonic_ms,
        )
        .unwrap();
    t154_wait_for_output(owner, attachment, b"WINDS_T154_READY");
}

fn t154_cleanup(home: PathBuf, runtime_root: PathBuf) {
    fs::remove_dir_all(home).unwrap();
    let _ = fs::remove_dir_all(runtime_root);
}

#[test]
fn t154_owner_path_rejects_observer_mutation_and_serializes_release_takeover_stop() {
    let home = t154_owner_root("owner-control");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let runtime_root = t154_runtime_root();
    #[cfg(windows)]
    let runtime_root = t154_owner_root("windows-runtime-placeholder");

    let profile = t154_shell_profile(&home);
    let mut owner = t154_start_owner(&home, &runtime_root, 10);
    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("controlled-shell").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            11,
            100,
        )
        .unwrap();
    t154_prime_windows_terminal(&mut owner, &attachment);
    let runtime_namespace_id = attachment.runtime_namespace_id();
    let controller_a = ClientConnectionId::new("controller-a").unwrap();
    let controller_b = ClientConnectionId::new("controller-b").unwrap();
    let observer = ClientConnectionId::new("observer").unwrap();
    let observer_handle = owner
        .attach_terminal_observer(observer.clone(), runtime_namespace_id)
        .unwrap();

    let denied = owner
        .controller_send_terminal_input(&observer, runtime_namespace_id, b"forbidden", 12, 101)
        .unwrap_err();
    assert!(
        denied
            .to_string()
            .contains("does not hold the active controller lease")
    );

    let state = owner
        .request_terminal_control(controller_a.clone(), runtime_namespace_id, 13, 102)
        .unwrap();
    assert_eq!(state.authority, ClientAuthority::Controller);
    assert_eq!(state.controller_client_id, Some(controller_a.clone()));

    let conflict = owner
        .request_terminal_control(controller_b.clone(), runtime_namespace_id, 14, 103)
        .unwrap_err();
    assert!(
        conflict
            .to_string()
            .contains("already held by controller-a")
    );

    owner
        .fill_terminal_observer_queue(&observer_handle)
        .unwrap();
    let first_events = owner.drain_terminal_observer(&observer_handle).unwrap();
    assert!(first_events.iter().any(|message| {
        matches!(
            &message.payload,
            ProtocolPayload::RuntimeEvent { event }
                if event.kind == RuntimeLifecycleEventKind::ControllerChanged
                    && event.proof_class == LifecycleProofClass::WindsObserved
                    && event.controller_client_id.as_ref() == Some(&controller_a)
        )
    }));

    let resized = TerminalSize {
        rows: 40,
        cols: 120,
    };
    owner
        .controller_resize_terminal(&controller_a, runtime_namespace_id, resized, 15, 104)
        .unwrap();
    assert_eq!(owner.terminal_runtime_size(&attachment).unwrap(), resized);
    assert!(
        owner
            .controller_resize_terminal(
                &controller_b,
                runtime_namespace_id,
                TerminalSize {
                    rows: 41,
                    cols: 121
                },
                16,
                105,
            )
            .is_err()
    );
    assert_eq!(owner.terminal_runtime_size(&attachment).unwrap(), resized);

    let released = owner
        .release_terminal_control(&controller_a, runtime_namespace_id, 17, 106)
        .unwrap();
    assert_eq!(released.authority, ClientAuthority::Observer);
    assert!(released.controller_client_id.is_none());

    let takeover = owner
        .request_terminal_control(controller_b.clone(), runtime_namespace_id, 18, 107)
        .unwrap();
    assert_eq!(takeover.authority, ClientAuthority::Controller);
    assert_eq!(takeover.controller_client_id, Some(controller_b.clone()));
    assert!(
        owner
            .controller_send_terminal_input(
                &controller_a,
                runtime_namespace_id,
                b"stale-controller",
                19,
                108,
            )
            .is_err()
    );

    t154_enter_terminable_workload(
        &mut owner,
        &controller_b,
        runtime_namespace_id,
        &attachment,
        20,
        109,
    );
    let stopped = owner
        .controller_stop_terminal(&controller_b, runtime_namespace_id, 21, 110)
        .unwrap();
    assert_eq!(
        stopped.truth.process_liveness,
        crate::persistent_runtime::domain::ProcessLiveness::Exited
    );

    let final_state = owner
        .terminal_control_state(&controller_b, runtime_namespace_id, 22, 111)
        .unwrap();
    assert_eq!(final_state.authority, ClientAuthority::Observer);
    assert!(final_state.controller_client_id.is_none());

    owner
        .fill_terminal_observer_queue(&observer_handle)
        .unwrap();
    let final_events = owner.drain_terminal_observer(&observer_handle).unwrap();
    assert!(final_events.iter().any(|message| {
        matches!(
            &message.payload,
            ProtocolPayload::RuntimeEvent { event }
                if event.kind == RuntimeLifecycleEventKind::RuntimeStopped
                    && event.proof_class == LifecycleProofClass::WindsObserved
        )
    }));
    assert!(final_events.iter().any(|message| {
        matches!(
            &message.payload,
            ProtocolPayload::RuntimeEvent { event }
                if event.kind == RuntimeLifecycleEventKind::ControllerChanged
                    && event.controller_client_id.is_none()
        )
    }));

    owner.detach_terminal_observer(&observer_handle).unwrap();
    drop(owner);
    t154_cleanup(home, runtime_root);
}

#[test]
fn t154_owner_poll_expires_lease_without_automatic_observer_promotion() {
    let home = t154_owner_root("owner-expiry");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let runtime_root = t154_runtime_root();
    #[cfg(windows)]
    let runtime_root = t154_owner_root("windows-runtime-placeholder-expiry");

    let profile = t154_shell_profile(&home);
    let mut owner = t154_start_owner(&home, &runtime_root, 30);
    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("expiry-shell").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            31,
            1_000,
        )
        .unwrap();
    t154_prime_windows_terminal(&mut owner, &attachment);
    let runtime_namespace_id = attachment.runtime_namespace_id();
    let controller = ClientConnectionId::new("expiry-controller").unwrap();
    let observer = ClientConnectionId::new("expiry-observer").unwrap();

    owner
        .request_terminal_control(controller.clone(), runtime_namespace_id, 32, 1_000)
        .unwrap();
    owner
        .poll_terminal_runtimes(
            33,
            1_000_u64
                .saturating_add(crate::persistent_runtime::controller::CONTROLLER_LEASE_EXPIRY_MS),
        )
        .unwrap();

    let observer_state = owner
        .terminal_control_state(
            &observer,
            runtime_namespace_id,
            34,
            1_000_u64
                .saturating_add(crate::persistent_runtime::controller::CONTROLLER_LEASE_EXPIRY_MS),
        )
        .unwrap();
    assert_eq!(observer_state.authority, ClientAuthority::Observer);
    assert!(observer_state.controller_client_id.is_none());
    assert!(
        owner
            .controller_send_terminal_input(
                &controller,
                runtime_namespace_id,
                b"expired-controller",
                35,
                1_001_u64.saturating_add(
                    crate::persistent_runtime::controller::CONTROLLER_LEASE_EXPIRY_MS,
                ),
            )
            .is_err()
    );

    owner
        .close_terminal_runtime(
            &attachment,
            36,
            1_002_u64
                .saturating_add(crate::persistent_runtime::controller::CONTROLLER_LEASE_EXPIRY_MS),
        )
        .unwrap();
    drop(owner);
    t154_cleanup(home, runtime_root);
}
