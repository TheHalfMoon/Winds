use super::{
    NewWindsSession, NewWorkspace, NewWorkstream, Store, persistent_runtime_schema_objects,
};
use crate::persistent_runtime::domain::{
    ContinuityClass, EndpointAvailability, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeAlias, RuntimeLifecycleEventKind, RuntimeNamespaceId, RuntimeTruth,
};
use crate::persistent_runtime::persistence::{
    PersistentRuntimeRecordInput, PersistentRuntimeRecoveryReason,
};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t147-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn cleanup(home: &Path) {
    let _ = fs::remove_dir_all(home);
}

fn namespace(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn unowned_truth() -> RuntimeTruth {
    RuntimeTruth {
        ownership: OwnershipState::Unowned,
        process_liveness: ProcessLiveness::Unknown,
        endpoint_availability: EndpointAvailability::Unknown,
        continuity: ContinuityClass::Unknown,
    }
}

fn live_truth() -> RuntimeTruth {
    RuntimeTruth {
        ownership: OwnershipState::LiveOwned,
        process_liveness: ProcessLiveness::Running,
        endpoint_availability: EndpointAvailability::Available,
        continuity: ContinuityClass::RetainedLiveProcess,
    }
}

fn input<'a>(
    runtime_namespace_id: RuntimeNamespaceId,
    runtime_alias: &'a RuntimeAlias,
    owner_generation_id: Option<OwnerGenerationId>,
    truth: &'a RuntimeTruth,
    created_unix_ms: i64,
    updated_unix_ms: i64,
) -> PersistentRuntimeRecordInput<'a> {
    PersistentRuntimeRecordInput {
        runtime_namespace_id,
        runtime_alias,
        workspace_id: None,
        session_id: None,
        terminal_execution_id: None,
        owner_generation_id,
        truth,
        last_lifecycle_event_kind: if truth.ownership == OwnershipState::LiveOwned {
            RuntimeLifecycleEventKind::OwnershipEstablished
        } else {
            RuntimeLifecycleEventKind::NamespaceCreated
        },
        created_unix_ms,
        updated_unix_ms,
        last_observed_unix_ms: Some(updated_unix_ms),
        ownership_lost_unix_ms: None,
        recovery_reason: None,
    }
}

fn seed_canonical(store: &Store) {
    for (workspace_id, root) in [
        ("workspace-a", "/tmp/winds-t147-workspace-a"),
        ("workspace-b", "/tmp/winds-t147-workspace-b"),
    ] {
        store
            .create_workspace(
                NewWorkspace {
                    workspace_id,
                    canonical_worktree_root: root,
                    git_common_dir: &format!("{root}/.git"),
                },
                10,
            )
            .unwrap();
    }

    for (workstream_id, workspace_id) in [
        ("workstream-a", "workspace-a"),
        ("workstream-b", "workspace-b"),
    ] {
        store
            .create_workstream(
                NewWorkstream {
                    workstream_id,
                    workspace_id,
                    display_name: workstream_id,
                },
                20,
            )
            .unwrap();
    }

    for (session_id, workstream_id) in
        [("session-a", "workstream-a"), ("session-b", "workstream-b")]
    {
        store
            .create_winds_session(
                NewWindsSession {
                    session_id,
                    workstream_id,
                    display_name: session_id,
                },
                30,
            )
            .unwrap();
    }

    store
        .connection
        .execute(
            "INSERT INTO executions(
                execution_id, workspace_id, kind, request_source, execution_domain,
                status, status_source, requested_unix_ms
             ) VALUES (?1, ?2, 'TERMINAL', 'WINDS_OBSERVED', 'T147_TEST',
                       'REQUESTED', 'WINDS_OBSERVED', 40)",
            params!["terminal-a", "workspace-a"],
        )
        .unwrap();
    store
        .connection
        .execute(
            "INSERT INTO terminal_sessions(
                execution_id, profile_id, shell_executable, shell_arguments_json, requested_cwd
             ) VALUES (?1, 'test', '/bin/sh', '[]', '/tmp')",
            ["terminal-a"],
        )
        .unwrap();
}

fn digest_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn t147_fresh_store_installs_and_validates_exact_schema_inventory() {
    let home = test_home("fresh");
    let store = Store::open(&home).unwrap();
    store.validate_persistent_runtime_schema().unwrap();

    let objects = persistent_runtime_schema_objects(&store.connection).unwrap();
    let names = objects.keys().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "idx_persistent_runtime_generation",
            "persistent_runtime_namespaces",
            "persistent_runtime_owner_generations",
            "trg_persistent_runtime_insert_scope",
            "trg_persistent_runtime_reject_observation_regression",
            "trg_persistent_runtime_reject_time_regression",
            "trg_persistent_runtime_update_scope",
        ]
    );

    cleanup(&home);
}

#[test]
fn t147_existing_database_without_0013_migrates_transactionally() {
    let home = test_home("migrated");
    {
        let store = Store::open(&home).unwrap();
        store
            .connection
            .execute_batch(
                "DROP TABLE persistent_runtime_namespaces;
                 DROP TABLE persistent_runtime_owner_generations;",
            )
            .unwrap();
        assert!(
            persistent_runtime_schema_objects(&store.connection)
                .unwrap()
                .is_empty()
        );
    }

    let reopened = Store::open(&home).unwrap();
    reopened.validate_persistent_runtime_schema().unwrap();
    assert_eq!(
        persistent_runtime_schema_objects(&reopened.connection)
            .unwrap()
            .len(),
        7
    );
    cleanup(&home);
}

#[test]
fn t147_duplicate_aliases_remain_non_authoritative_and_alias_is_mutable() {
    let home = test_home("aliases");
    let store = Store::open(&home).unwrap();
    let alias = RuntimeAlias::new("worker").unwrap();
    let truth = unowned_truth();

    for id in [namespace(1), namespace(2)] {
        store
            .persist_persistent_runtime_record(&input(id, &alias, None, &truth, 10, 10))
            .unwrap();
    }

    let first = store.load_persistent_runtime_record(namespace(1)).unwrap();
    let second = store.load_persistent_runtime_record(namespace(2)).unwrap();
    assert_eq!(first.runtime_alias, second.runtime_alias);
    assert_ne!(first.runtime_namespace_id, second.runtime_namespace_id);

    let renamed = RuntimeAlias::new("worker-renamed").unwrap();
    store
        .persist_persistent_runtime_record(&input(namespace(1), &renamed, None, &truth, 10, 20))
        .unwrap();
    assert_eq!(
        store
            .load_persistent_runtime_record(namespace(1))
            .unwrap()
            .runtime_alias
            .as_str(),
        "worker-renamed"
    );

    let retrograde = store.persist_persistent_runtime_record(&input(
        namespace(1),
        &renamed,
        None,
        &truth,
        10,
        19,
    ));
    assert!(retrograde.is_err());

    let mut regressed_observation = input(namespace(1), &renamed, None, &truth, 10, 21);
    regressed_observation.last_observed_unix_ms = Some(19);
    assert!(
        store
            .persist_persistent_runtime_record(&regressed_observation)
            .is_err()
    );

    let preserved = store.load_persistent_runtime_record(namespace(1)).unwrap();
    assert_eq!(preserved.updated_unix_ms, 20);
    assert_eq!(preserved.last_observed_unix_ms, Some(20));

    cleanup(&home);
}

#[test]
fn t147_persisted_live_row_is_historical_metadata_not_live_ownership_proof() {
    let home = test_home("proof");
    let store = Store::open(&home).unwrap();
    let generation = generation(3);
    store
        .record_persistent_runtime_owner_generation(generation, 10)
        .unwrap();
    let alias = RuntimeAlias::new("owned").unwrap();
    let truth = live_truth();
    store
        .persist_persistent_runtime_record(&input(
            namespace(3),
            &alias,
            Some(generation),
            &truth,
            10,
            11,
        ))
        .unwrap();

    let error = store
        .load_persistent_runtime_record(namespace(3))
        .unwrap_err()
        .to_string();
    assert!(error.contains("historical metadata only"));
    assert!(error.contains("owner-held ownership primitive"));

    cleanup(&home);
}

#[test]
fn t147_restart_without_proven_generation_reconciles_live_owned_to_ownership_lost() {
    let home = test_home("restart");
    let old_generation = generation(4);
    let runtime_id = namespace(4);
    {
        let store = Store::open(&home).unwrap();
        store
            .record_persistent_runtime_owner_generation(old_generation, 10)
            .unwrap();
        let alias = RuntimeAlias::new("restart-runtime").unwrap();
        let truth = live_truth();
        store
            .persist_persistent_runtime_record(&input(
                runtime_id,
                &alias,
                Some(old_generation),
                &truth,
                10,
                12,
            ))
            .unwrap();
    }

    let reopened = Store::open(&home).unwrap();
    assert!(reopened.load_persistent_runtime_record(runtime_id).is_err());
    assert_eq!(
        reopened
            .reconcile_persistent_runtime_records(None, 20)
            .unwrap(),
        1
    );

    let reconciled = reopened.load_persistent_runtime_record(runtime_id).unwrap();
    assert_eq!(reconciled.truth.ownership, OwnershipState::OwnershipLost);
    assert_eq!(reconciled.truth.process_liveness, ProcessLiveness::Unknown);
    assert_eq!(
        reconciled.truth.endpoint_availability,
        EndpointAvailability::Unknown
    );
    assert_eq!(reconciled.truth.continuity, ContinuityClass::Unknown);
    assert_eq!(
        reconciled.recovery_reason,
        Some(PersistentRuntimeRecoveryReason::OwnerGenerationUnproven)
    );
    assert_eq!(reconciled.owner_generation_id, Some(old_generation));
    assert_eq!(reconciled.ownership_lost_unix_ms, Some(20));

    cleanup(&home);
}

#[test]
fn t147_stale_generation_is_lost_while_current_generation_can_remain_live() {
    let home = test_home("stale-generation");
    let store = Store::open(&home).unwrap();
    let stale = generation(5);
    let current = generation(6);
    store
        .record_persistent_runtime_owner_generation(stale, 10)
        .unwrap();
    store
        .record_persistent_runtime_owner_generation(current, 15)
        .unwrap();

    let truth = live_truth();
    let stale_alias = RuntimeAlias::new("stale").unwrap();
    let current_alias = RuntimeAlias::new("current").unwrap();
    store
        .persist_persistent_runtime_record(&input(
            namespace(5),
            &stale_alias,
            Some(stale),
            &truth,
            10,
            16,
        ))
        .unwrap();
    store
        .persist_persistent_runtime_record(&input(
            namespace(6),
            &current_alias,
            Some(current),
            &truth,
            15,
            16,
        ))
        .unwrap();

    assert_eq!(
        store
            .reconcile_persistent_runtime_records(Some(current), 20)
            .unwrap(),
        1
    );

    let stale_record = store.load_persistent_runtime_record(namespace(5)).unwrap();
    assert_eq!(
        stale_record.recovery_reason,
        Some(PersistentRuntimeRecoveryReason::OwnerGenerationChanged)
    );
    assert_eq!(stale_record.truth.ownership, OwnershipState::OwnershipLost);

    let current_error = store
        .load_persistent_runtime_record(namespace(6))
        .unwrap_err()
        .to_string();
    assert!(current_error.contains("historical metadata only"));
    let current_state: String = store
        .connection
        .query_row(
            "SELECT ownership_state
             FROM persistent_runtime_namespaces
             WHERE runtime_namespace_id = ?1",
            [namespace(6).as_hex()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(current_state, "LIVE_OWNED");

    cleanup(&home);
}

#[test]
fn t147_partial_or_corrupt_schema_fails_closed_without_silent_reinitialization() {
    let partial_home = test_home("partial-schema");
    {
        let store = Store::open(&partial_home).unwrap();
        store
            .connection
            .execute_batch("DROP TABLE persistent_runtime_namespaces;")
            .unwrap();
    }
    assert!(Store::open(&partial_home).is_err());
    let partial_connection = Connection::open(partial_home.join("winds.db")).unwrap();
    let generation_table_exists: i64 = partial_connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type = 'table' AND name = 'persistent_runtime_owner_generations'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(generation_table_exists, 1);
    drop(partial_connection);
    cleanup(&partial_home);

    let corrupt_home = test_home("corrupt-schema");
    {
        let store = Store::open(&corrupt_home).unwrap();
        store
            .connection
            .execute_batch(
                "CREATE TABLE persistent_runtime_intruder(
                    id INTEGER PRIMARY KEY
                 );",
            )
            .unwrap();
    }
    assert!(Store::open(&corrupt_home).is_err());
    cleanup(&corrupt_home);
}

#[test]
fn t147_partial_and_corrupt_rows_fail_closed() {
    let home = test_home("corrupt-row");
    let store = Store::open(&home).unwrap();

    let partial = store.connection.execute(
        "INSERT INTO persistent_runtime_namespaces(runtime_namespace_id)
         VALUES (?1)",
        [namespace(7).as_hex()],
    );
    assert!(partial.is_err());

    let generation = generation(7);
    store
        .record_persistent_runtime_owner_generation(generation, 10)
        .unwrap();
    let alias = RuntimeAlias::new("corrupt-me").unwrap();
    let truth = live_truth();
    store
        .persist_persistent_runtime_record(&input(
            namespace(7),
            &alias,
            Some(generation),
            &truth,
            10,
            10,
        ))
        .unwrap();

    store
        .connection
        .execute_batch("PRAGMA ignore_check_constraints = ON;")
        .unwrap();
    store
        .connection
        .execute(
            "UPDATE persistent_runtime_namespaces
             SET schema_version = 2
             WHERE runtime_namespace_id = ?1",
            [namespace(7).as_hex()],
        )
        .unwrap();
    store
        .connection
        .execute_batch("PRAGMA ignore_check_constraints = OFF;")
        .unwrap();

    let error = store
        .load_persistent_runtime_record(namespace(7))
        .unwrap_err()
        .to_string();
    assert!(error.contains("unsupported persistent runtime schema version"));

    cleanup(&home);
}

#[test]
fn t147_foreign_keys_and_canonical_scope_are_enforced_without_reverse_deletion() {
    let home = test_home("foreign-keys");
    let store = Store::open(&home).unwrap();
    seed_canonical(&store);
    let alias = RuntimeAlias::new("bound").unwrap();
    let truth = unowned_truth();

    let valid = PersistentRuntimeRecordInput {
        workspace_id: Some("workspace-a"),
        session_id: Some("session-a"),
        terminal_execution_id: Some("terminal-a"),
        ..input(namespace(8), &alias, None, &truth, 50, 50)
    };
    store.persist_persistent_runtime_record(&valid).unwrap();

    let missing_fk = PersistentRuntimeRecordInput {
        workspace_id: Some("workspace-missing"),
        ..input(namespace(9), &alias, None, &truth, 50, 50)
    };
    assert!(
        store
            .persist_persistent_runtime_record(&missing_fk)
            .is_err()
    );

    let mismatched_scope = PersistentRuntimeRecordInput {
        workspace_id: Some("workspace-b"),
        session_id: Some("session-a"),
        terminal_execution_id: Some("terminal-a"),
        ..input(namespace(10), &alias, None, &truth, 50, 50)
    };
    assert!(
        store
            .persist_persistent_runtime_record(&mismatched_scope)
            .is_err()
    );

    store
        .connection
        .execute(
            "DELETE FROM persistent_runtime_namespaces WHERE runtime_namespace_id = ?1",
            [namespace(8).as_hex()],
        )
        .unwrap();

    for (table, column, value) in [
        ("workspaces", "workspace_id", "workspace-a"),
        ("winds_sessions", "session_id", "session-a"),
        ("executions", "execution_id", "terminal-a"),
        ("terminal_sessions", "execution_id", "terminal-a"),
    ] {
        let query = format!("SELECT COUNT(*) FROM {table} WHERE {column} = ?1");
        let count: i64 = store
            .connection
            .query_row(&query, [value], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "{table}");
    }

    cleanup(&home);
}

#[test]
fn t147_schema_rejects_noncanonical_identity_version_and_loss_metadata() {
    let home = test_home("constraints");
    let store = Store::open(&home).unwrap();

    let invalid_generation = store.connection.execute(
        "INSERT INTO persistent_runtime_owner_generations(
            owner_generation_id, schema_version, started_unix_ms
         ) VALUES (?1, 1, 0)",
        ["00000000000000000000000000000000"],
    );
    assert!(invalid_generation.is_err());
    let invalid_version = store.connection.execute(
        "INSERT INTO persistent_runtime_owner_generations(
            owner_generation_id, schema_version, started_unix_ms
         ) VALUES (?1, 2, 0)",
        [generation(11).as_hex()],
    );
    assert!(invalid_version.is_err());

    let loss_without_reason = store.connection.execute(
        "INSERT INTO persistent_runtime_namespaces(
            runtime_namespace_id, schema_version, runtime_alias, ownership_state,
            process_liveness, endpoint_availability, continuity_class,
            last_lifecycle_event_kind, created_unix_ms, updated_unix_ms,
            ownership_lost_unix_ms
         ) VALUES (?1, 1, 'lost', 'OWNERSHIP_LOST', 'UNKNOWN', 'UNKNOWN', 'UNKNOWN',
                   'OWNERSHIP_LOST', 0, 1, 1)",
        [namespace(11).as_hex()],
    );
    assert!(loss_without_reason.is_err());

    cleanup(&home);
}

#[test]
fn t147_0011_0012_digests_remain_frozen_and_0013_has_no_pid_recovery_surface() {
    assert_eq!(
        digest_hex(include_bytes!(
            "../migrations/0011_model_mesh_continuity.sql"
        )),
        "9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d"
    );
    assert_eq!(
        digest_hex(include_bytes!(
            "../migrations/0012_desktop_presentation.sql"
        )),
        "b1972c19531fdc3d358abdaf1d948037cd7067e9eb49717ff3b5211ddc971f41"
    );

    let migration =
        include_str!("../migrations/0013_persistent_runtime_owner.sql").to_ascii_lowercase();
    let persistence = include_str!("persistent_runtime/persistence.rs").to_ascii_lowercase();
    for forbidden in [
        "pid",
        "process_handle",
        "pty_handle",
        "controller_lease",
        "credential",
        "bearer_token",
        "terminal_transcript",
    ] {
        assert!(
            !migration.contains(forbidden),
            "migration contains {forbidden}"
        );
        assert!(
            !persistence.contains(forbidden),
            "persistence contains {forbidden}"
        );
    }
}
