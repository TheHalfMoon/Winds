use super::*;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::persistent_runtime::domain::{
    ContinuityClass, EndpointAvailability, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeAlias, RuntimeLifecycleEventKind, RuntimeNamespaceId, RuntimeTruth,
};
use crate::persistent_runtime::persistence::{
    PersistentRuntimeRecordInput, PersistentRuntimeRecoveryReason,
};
use crate::store::Store;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_T157_OWNER_ROOT: AtomicU64 = AtomicU64::new(1);

fn test_root(label: &str) -> PathBuf {
    let id = NEXT_T157_OWNER_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t157-owner-{label}-{}-{id}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn short_runtime_root() -> PathBuf {
    let id = NEXT_T157_OWNER_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("w157r{id}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(windows)]
fn short_runtime_root() -> PathBuf {
    test_root("windows-runtime-placeholder")
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn namespace(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
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

fn seed_live_runtime(
    home: &Path,
    generation_id: OwnerGenerationId,
    runtime_namespace_id: RuntimeNamespaceId,
    now_unix_ms: i64,
) {
    let store = Store::open(home).unwrap();
    store
        .record_persistent_runtime_owner_generation(generation_id, now_unix_ms)
        .unwrap();
    let alias = RuntimeAlias::new("t157-recovery").unwrap();
    let truth = RuntimeTruth {
        ownership: OwnershipState::LiveOwned,
        process_liveness: ProcessLiveness::Running,
        endpoint_availability: EndpointAvailability::Available,
        continuity: ContinuityClass::RetainedLiveProcess,
    };
    store
        .persist_persistent_runtime_record(&PersistentRuntimeRecordInput {
            runtime_namespace_id,
            runtime_alias: &alias,
            workspace_id: None,
            session_id: None,
            terminal_execution_id: None,
            owner_generation_id: Some(generation_id),
            truth: &truth,
            last_lifecycle_event_kind: RuntimeLifecycleEventKind::OwnershipEstablished,
            created_unix_ms: now_unix_ms,
            updated_unix_ms: now_unix_ms,
            last_observed_unix_ms: Some(now_unix_ms),
            ownership_lost_unix_ms: None,
            recovery_reason: None,
        })
        .unwrap();
}

fn spawn_unrelated_process() -> Child {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        Command::new("/bin/sh")
            .args(["-c", "sleep 30"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }
    #[cfg(windows)]
    {
        Command::new(std::env::var("COMSPEC").expect("Windows CI must provide COMSPEC"))
            .args(["/D", "/S", "/C", "ping -n 30 127.0.0.1 >nul"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }
}

fn cleanup_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn t157_restart_marks_prior_generation_lost_and_ignores_stale_pid_material() {
    let home = test_root("stale-pid");
    let runtime_root = short_runtime_root();
    let old_generation = generation(0x51);
    let runtime_namespace_id = namespace(0x51);
    seed_live_runtime(&home, old_generation, runtime_namespace_id, 10);

    let mut unrelated = spawn_unrelated_process();
    fs::write(home.join("stale.pid"), unrelated.id().to_string()).unwrap();

    let owner = start_owner(&home, &runtime_root, 20);
    assert_eq!(owner.reconciled_runtime_count(), 1);
    let record = owner
        .store
        .load_persistent_runtime_record(runtime_namespace_id)
        .unwrap();
    assert_eq!(record.owner_generation_id, Some(old_generation));
    assert_eq!(record.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(record.truth.process_liveness, ProcessLiveness::Unknown);
    assert_eq!(
        record.truth.endpoint_availability,
        EndpointAvailability::Unknown
    );
    assert_eq!(record.truth.continuity, ContinuityClass::Unknown);
    assert_eq!(
        record.recovery_reason,
        Some(PersistentRuntimeRecoveryReason::OwnerGenerationChanged)
    );
    assert_eq!(record.ownership_lost_unix_ms, Some(20));
    assert_ne!(owner.generation_id(), old_generation);

    assert!(
        unrelated.try_wait().unwrap().is_none(),
        "restart reconciliation must never signal or adopt a process referenced only by stale PID material"
    );

    drop(owner);
    cleanup_child(&mut unrelated);
    let _ = fs::remove_dir_all(runtime_root);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t157_corrupt_live_owned_row_fails_before_reconciliation_rewrite() {
    let home = test_root("corrupt-row");
    let runtime_root = short_runtime_root();
    let old_generation = generation(0x52);
    let runtime_namespace_id = namespace(0x52);
    seed_live_runtime(&home, old_generation, runtime_namespace_id, 10);

    let connection = Connection::open(home.join("winds.db")).unwrap();
    connection
        .execute_batch("PRAGMA ignore_check_constraints = ON;")
        .unwrap();
    connection
        .execute(
            "UPDATE persistent_runtime_namespaces
             SET owner_generation_id = NULL
             WHERE runtime_namespace_id = ?1",
            [runtime_namespace_id.as_hex()],
        )
        .unwrap();
    drop(connection);

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let runtime_directory = runtime_root.join("r");
        crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
            .unwrap();
        let error = match PersistentOwner::start_for_test(&home, &runtime_directory, 20) {
            Ok(_) => panic!("corrupt LIVE_OWNED metadata must fail owner startup"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("corrupt LIVE_OWNED metadata"),
            "{error}"
        );
    }
    #[cfg(windows)]
    {
        let error = match PersistentOwner::start(&home, 20) {
            Ok(_) => panic!("corrupt LIVE_OWNED metadata must fail owner startup"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("corrupt LIVE_OWNED metadata"),
            "{error}"
        );
    }

    let connection = Connection::open(home.join("winds.db")).unwrap();
    let row: (String, Option<String>) = connection
        .query_row(
            "SELECT ownership_state, owner_generation_id
             FROM persistent_runtime_namespaces
             WHERE runtime_namespace_id = ?1",
            [runtime_namespace_id.as_hex()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(row.0, "LIVE_OWNED");
    assert_eq!(row.1, None);
    drop(connection);

    let _ = fs::remove_dir_all(runtime_root);
    fs::remove_dir_all(home).unwrap();
}

const CRASH_HELPER_ENV: &str = "WINDS_T157_CRASH_HELPER";
const CRASH_HOME_ENV: &str = "WINDS_T157_CRASH_HOME";
const CRASH_RUNTIME_ENV: &str = "WINDS_T157_CRASH_RUNTIME";
const CRASH_STATE_FILE: &str = "t157-crash-state.txt";

#[test]
#[ignore = "invoked only as a subprocess by the T157 abrupt-owner campaign"]
fn t157_abrupt_owner_helper() {
    if std::env::var_os(CRASH_HELPER_ENV).is_none() {
        return;
    }
    let home = PathBuf::from(std::env::var_os(CRASH_HOME_ENV).unwrap());
    let runtime_root = PathBuf::from(std::env::var_os(CRASH_RUNTIME_ENV).unwrap());
    let profile = native_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root, 30);
    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("abrupt-owner-child").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            31,
            1,
        )
        .unwrap();

    #[cfg(windows)]
    let long_running = b"ping -n 6 127.0.0.1 >nul\r\n";
    #[cfg(not(windows))]
    let long_running = b"sleep 5\n";
    owner
        .send_terminal_runtime_input(&attachment, long_running)
        .unwrap();

    fs::write(
        home.join(CRASH_STATE_FILE),
        format!(
            "{}\n{}\n",
            attachment.runtime_namespace_id().as_hex(),
            attachment.owner_generation_id().as_hex()
        ),
    )
    .unwrap();

    std::process::exit(97);
}

#[test]
fn t157_abrupt_owner_loss_reconciles_to_unknown_without_process_reconstruction() {
    let home = test_root("abrupt-owner");
    let runtime_root = short_runtime_root();
    let current_exe = std::env::current_exe().unwrap();
    let helper_name =
        "persistent_runtime::owner::t157_owner_recovery_tests::t157_abrupt_owner_helper";
    let status = Command::new(current_exe)
        .arg("--ignored")
        .arg("--exact")
        .arg(helper_name)
        .arg("--test-threads=1")
        .env(CRASH_HELPER_ENV, "1")
        .env(CRASH_HOME_ENV, &home)
        .env(CRASH_RUNTIME_ENV, &runtime_root)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(97));

    let state = fs::read_to_string(home.join(CRASH_STATE_FILE)).unwrap();
    let mut lines = state.lines();
    let runtime_namespace_id = RuntimeNamespaceId::parse(lines.next().unwrap()).unwrap();
    let prior_generation = OwnerGenerationId::parse(lines.next().unwrap()).unwrap();

    let mut replacement = start_owner(&home, &runtime_root, 50);
    assert_ne!(replacement.generation_id(), prior_generation);
    assert_eq!(replacement.reconciled_runtime_count(), 1);

    let record = replacement
        .store
        .load_persistent_runtime_record(runtime_namespace_id)
        .unwrap();
    assert_eq!(record.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(record.truth.process_liveness, ProcessLiveness::Unknown);
    assert_eq!(record.truth.continuity, ContinuityClass::Unknown);
    assert_eq!(
        record.recovery_reason,
        Some(PersistentRuntimeRecoveryReason::OwnerGenerationChanged)
    );

    let error = replacement
        .reattach_terminal_runtime(runtime_namespace_id, prior_generation, 51, 2)
        .unwrap_err();
    assert!(error.to_string().contains("owner generation"), "{error}");

    drop(replacement);
    let _ = fs::remove_dir_all(runtime_root);
    fs::remove_dir_all(home).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn t157_stale_same_user_socket_is_removed_only_after_identity_and_liveness_proof() {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    let home = test_root("stale-endpoint");
    let runtime_root = short_runtime_root();
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .unwrap();
    let endpoint =
        crate::persistent_runtime::transport::unix::endpoint_path(&runtime_directory).unwrap();

    let stale = UnixListener::bind(&endpoint).unwrap();
    fs::set_permissions(&endpoint, fs::Permissions::from_mode(0o600)).unwrap();
    drop(stale);

    let owner = PersistentOwner::start_for_test(&home, &runtime_directory, 10).unwrap();
    assert!(owner.is_ready());
    drop(owner);

    let _ = fs::remove_dir_all(runtime_root);
    fs::remove_dir_all(home).unwrap();
}

#[cfg(windows)]
#[test]
fn t157_closed_named_pipe_never_requires_stale_pid_or_endpoint_reconstruction() {
    use crate::persistent_runtime::transport::windows::WindowsNamedPipeServer;

    let generation = generation(0x53);
    {
        let _server = WindowsNamedPipeServer::bind(generation).unwrap();
    }
    let _rebound = WindowsNamedPipeServer::bind(generation).unwrap();
}

#[test]
fn t157_owner_shutdown_boundary_reports_each_runtime_and_clears_live_activity() {
    let home = test_root("owner-shutdown-boundary");
    let runtime_root = short_runtime_root();
    let profile = native_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root, 70);

    let first = owner
        .start_terminal_runtime(
            RuntimeAlias::new("shutdown-first").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            71,
            1,
        )
        .unwrap();
    let second = owner
        .start_terminal_runtime(
            RuntimeAlias::new("shutdown-second").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            72,
            2,
        )
        .unwrap();

    let reports = owner.shutdown_terminal_runtimes(80, 10);
    assert_eq!(reports.len(), 2);
    assert!(reports.iter().all(|report| report.error.is_none()));
    assert!(reports.iter().any(|report| {
        report.runtime_namespace_id == first.runtime_namespace_id()
            && report.snapshot.truth.process_liveness == ProcessLiveness::Exited
    }));
    assert!(reports.iter().any(|report| {
        report.runtime_namespace_id == second.runtime_namespace_id()
            && report.snapshot.truth.process_liveness == ProcessLiveness::Exited
    }));
    assert_eq!(owner.live_terminal_runtime_count(), 0);

    drop(owner);
    let _ = fs::remove_dir_all(runtime_root);
    fs::remove_dir_all(home).unwrap();
}
