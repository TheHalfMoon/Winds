use super::*;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::persistent_runtime::domain::{
    ContinuityClass, EndpointAvailability, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeAlias, RuntimeLifecycleEventKind, RuntimeNamespaceId, RuntimeTruth,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::persistent_runtime::persistence::{
    PersistentRuntimeRecordInput, PersistentRuntimeRecoveryReason,
};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::store::Store;
use std::fs;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

static NEXT_TEST_HOME: AtomicU64 = AtomicU64::new(1);

fn test_root(label: &str) -> PathBuf {
    let base = std::env::temp_dir()
        .canonicalize()
        .expect("temp directory must canonicalize");
    let id = NEXT_TEST_HOME.fetch_add(1, Ordering::Relaxed);
    let path = base.join(format!("winds-t151-{label}-{}-{id}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn short_runtime_root() -> PathBuf {
    let base = std::env::temp_dir()
        .canonicalize()
        .expect("temp directory must canonicalize");
    let id = NEXT_TEST_HOME.fetch_add(1, Ordering::Relaxed);
    let path = base.join(format!("w151r{id}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn namespace(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn seed_live_runtime(
    home: &Path,
    generation_id: OwnerGenerationId,
    runtime_id: RuntimeNamespaceId,
) {
    let store = Store::open(home).unwrap();
    store
        .record_persistent_runtime_owner_generation(generation_id, 10)
        .unwrap();
    let alias = RuntimeAlias::new("owner-shell-fixture").unwrap();
    let truth = RuntimeTruth {
        ownership: OwnershipState::LiveOwned,
        process_liveness: ProcessLiveness::Running,
        endpoint_availability: EndpointAvailability::Available,
        continuity: ContinuityClass::RetainedLiveProcess,
    };
    store
        .persist_persistent_runtime_record(&PersistentRuntimeRecordInput {
            runtime_namespace_id: runtime_id,
            runtime_alias: &alias,
            workspace_id: None,
            session_id: None,
            terminal_execution_id: None,
            owner_generation_id: Some(generation_id),
            truth: &truth,
            last_lifecycle_event_kind: RuntimeLifecycleEventKind::OwnershipEstablished,
            created_unix_ms: 10,
            updated_unix_ms: 10,
            last_observed_unix_ms: Some(10),
            ownership_lost_unix_ms: None,
            recovery_reason: None,
        })
        .unwrap();
}

#[test]
fn t151_singleton_concurrent_race_has_exactly_one_winner() {
    let home = test_root("singleton-race");
    let contenders = 12_usize;
    let barrier = Arc::new(Barrier::new(contenders));
    let attempted = Arc::new(AtomicUsize::new(0));
    let winners = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for _ in 0..contenders {
        let home = home.clone();
        let barrier = Arc::clone(&barrier);
        let attempted = Arc::clone(&attempted);
        let winners = Arc::clone(&winners);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            let acquired = OwnerSingleton::acquire(&home);
            attempted.fetch_add(1, Ordering::SeqCst);
            match acquired {
                Ok(_guard) => {
                    winners.fetch_add(1, Ordering::SeqCst);
                    while attempted.load(Ordering::SeqCst) != contenders {
                        std::thread::yield_now();
                    }
                }
                Err(OwnerError::SingletonAlreadyActive) => {}
                Err(error) => panic!("unexpected singleton race error: {error}"),
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(winners.load(Ordering::SeqCst), 1);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t151_stale_singleton_file_is_not_authority_after_kernel_lock_release() {
    let home = test_root("stale-singleton");
    let first = OwnerSingleton::acquire(&home).unwrap();
    drop(first);
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    assert!(home.join("persistent-runtime/owner.lock").exists());
    #[cfg(windows)]
    assert!(home.join("persistent-owner.lock").exists());
    let second = OwnerSingleton::acquire(&home).unwrap();
    drop(second);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn t151_idle_exit_is_exactly_300_seconds_and_non_idle_state_preserves_owner() {
    let mut activity = OwnerActivity::new(1_000);
    assert!(!activity.should_exit(300_999));
    assert!(activity.should_exit(301_000));

    activity.client_connected();
    assert!(!activity.should_exit(900_000));
    activity.client_disconnected(900_000).unwrap();
    assert!(!activity.should_exit(1_199_999));
    assert!(activity.should_exit(1_200_000));

    activity.set_live_runtime_count(1, 1_200_000);
    assert!(!activity.should_exit(9_000_000));
    activity.set_live_runtime_count(0, 9_000_000);
    assert!(!activity.should_exit(9_299_999));
    assert!(activity.should_exit(9_300_000));
    assert_eq!(OWNER_IDLE_GRACE_MS, 300_000);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn t151_stale_endpoint_and_prior_live_generation_reconcile_before_ready() {
    use crate::persistent_runtime::transport::unix::{endpoint_path, prepare_runtime_directory};
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    let home = test_root("restart-reconcile");
    let runtime_root = short_runtime_root();
    let runtime_directory = runtime_root.join("r");
    prepare_runtime_directory(&runtime_directory).unwrap();
    let stale_path = endpoint_path(&runtime_directory).unwrap();
    let stale = UnixListener::bind(&stale_path).unwrap();
    fs::set_permissions(&stale_path, fs::Permissions::from_mode(0o600)).unwrap();
    drop(stale);

    let old_generation = generation(4);
    let runtime_id = namespace(4);
    seed_live_runtime(&home, old_generation, runtime_id);

    let owner = PersistentOwner::start_for_test(&home, &runtime_directory, 20).unwrap();
    assert!(owner.is_ready());
    assert_eq!(owner.ready_unix_ms(), 20);
    assert_eq!(owner.reconciled_runtime_count(), 1);
    assert_ne!(owner.generation_id(), old_generation);

    let record = owner
        .store
        .load_persistent_runtime_record(runtime_id)
        .unwrap();
    assert_eq!(record.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(
        record.recovery_reason,
        Some(PersistentRuntimeRecoveryReason::OwnerGenerationChanged)
    );
    drop(owner);
    fs::remove_dir_all(home).unwrap();
    fs::remove_dir_all(runtime_root).unwrap();
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn t151_endpoint_startup_failure_occurs_after_reconciliation_and_never_returns_ready() {
    use crate::persistent_runtime::transport::unix::{endpoint_path, prepare_runtime_directory};

    let home = test_root("endpoint-failure");
    let runtime_root = short_runtime_root();
    let runtime_directory = runtime_root.join("r");
    prepare_runtime_directory(&runtime_directory).unwrap();
    let collision = endpoint_path(&runtime_directory).unwrap();
    fs::write(&collision, b"not-a-socket").unwrap();

    let old_generation = generation(5);
    let runtime_id = namespace(5);
    seed_live_runtime(&home, old_generation, runtime_id);

    let error = match PersistentOwner::start_for_test(&home, &runtime_directory, 20) {
        Ok(_) => panic!("owner must not become ready when endpoint bind fails"),
        Err(error) => error,
    };
    assert!(matches!(error, OwnerError::Endpoint(_)));

    let store = Store::open(&home).unwrap();
    let record = store.load_persistent_runtime_record(runtime_id).unwrap();
    assert_eq!(record.truth.ownership, OwnershipState::OwnershipLost);
    drop(store);
    fs::remove_dir_all(home).unwrap();
    fs::remove_dir_all(runtime_root).unwrap();
}

#[test]
fn t151_internal_owner_mode_is_hidden_and_has_zero_child_or_wire_shutdown_authority() {
    assert!(!crate::usage().contains(INTERNAL_OWNER_COMMAND));
    let source = include_str!("persistent_runtime/owner.rs");
    for prohibited in [
        "std::process::Command",
        "Command::new",
        "portable_pty",
        "NativePtySystem",
        "ConPty",
        "SHUTDOWN_IF_IDLE",
        "MessageKind::Stop",
        "RuntimeStopped",
    ] {
        assert!(!source.contains(prohibited), "{prohibited}");
    }
}
