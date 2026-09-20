use super::{NewWorkspace, Store, multiplexer_schema_objects};
use crate::multiplexer::domain::navigation::{
    LayoutTemplateNode, LayoutTemplateStructure, LayoutTemplateTab, MultiplexerTopology, SplitAxis,
    SplitRatioBps,
};
use crate::multiplexer::domain::persistence::{
    LayoutTemplateV1, RepositoryTrustRecord, TopologySnapshotV1, WorktreeMembershipRecord,
    WorktreeMembershipSource, WorktreeMembershipState,
};
use crate::multiplexer::domain::{
    LayoutTemplateId, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t164-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn cleanup(home: &Path) {
    let _ = fs::remove_dir_all(home);
}

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).unwrap()
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).unwrap()
}

fn template_id(byte: u8) -> LayoutTemplateId {
    LayoutTemplateId::from_entropy_bytes([byte; 16]).unwrap()
}

fn snapshot(alias: &str) -> TopologySnapshotV1 {
    let workspace_id = workspace(1);
    let mut topology = MultiplexerTopology::empty();
    let generation = topology
        .create_workspace(
            topology.generation(),
            workspace_id,
            alias.to_owned(),
            tab(1),
            "main".to_owned(),
            pane(1),
        )
        .unwrap();
    TopologySnapshotV1::from_workspace(
        topology.workspace(workspace_id).unwrap(),
        generation,
        topology.focused_workspace_id() == Some(workspace_id),
    )
    .unwrap()
}

fn template() -> LayoutTemplateV1 {
    LayoutTemplateV1::from_structure(&LayoutTemplateStructure {
        tabs: vec![LayoutTemplateTab {
            alias: "editor".to_owned(),
            root: LayoutTemplateNode::Split {
                axis: SplitAxis::Horizontal,
                ratio_bps: SplitRatioBps::new(5_000).unwrap(),
                first: Box::new(LayoutTemplateNode::Pane),
                second: Box::new(LayoutTemplateNode::Pane),
            },
        }],
    })
    .unwrap()
}

#[test]
fn t164_fresh_store_installs_and_validates_exact_schema_inventory() {
    let home = test_home("fresh");
    let store = Store::open(&home).unwrap();
    store.validate_multiplexer_schema().unwrap();

    let objects = multiplexer_schema_objects(&store.connection).unwrap();
    let names = objects.keys().map(String::as_str).collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "idx_multiplexer_worktree_membership_state",
            "multiplexer_layout_templates",
            "multiplexer_repository_trust",
            "multiplexer_workspaces",
            "multiplexer_worktree_memberships",
            "trg_multiplexer_layout_template_capacity",
            "trg_multiplexer_layout_template_time_regression",
            "trg_multiplexer_membership_confirmation_regression",
            "trg_multiplexer_repository_trust_revision_regression",
            "trg_multiplexer_repository_trust_revision_reuse",
            "trg_multiplexer_repository_trust_time_regression",
            "trg_multiplexer_workspace_capacity",
            "trg_multiplexer_workspace_generation_regression",
            "trg_multiplexer_workspace_generation_reuse",
            "trg_multiplexer_workspace_time_regression",
            "trg_multiplexer_worktree_membership_capacity",
        ]
    );

    cleanup(&home);
}

#[test]
fn t164_existing_database_without_0014_migrates_transactionally() {
    let home = test_home("migrated");
    {
        let store = Store::open(&home).unwrap();
        store
            .connection
            .execute_batch(
                "DROP TABLE multiplexer_worktree_memberships;
                 DROP TABLE multiplexer_repository_trust;
                 DROP TABLE multiplexer_layout_templates;
                 DROP TABLE multiplexer_workspaces;",
            )
            .unwrap();
        assert!(
            multiplexer_schema_objects(&store.connection)
                .unwrap()
                .is_empty()
        );
    }

    let reopened = Store::open(&home).unwrap();
    reopened.validate_multiplexer_schema().unwrap();
    assert_eq!(
        multiplexer_schema_objects(&reopened.connection)
            .unwrap()
            .len(),
        16
    );
    cleanup(&home);
}

#[test]
fn t164_partial_or_foreign_schema_fails_closed_without_silent_reinitialization() {
    let partial_home = test_home("partial");
    {
        let store = Store::open(&partial_home).unwrap();
        store
            .connection
            .execute_batch("DROP TABLE multiplexer_layout_templates;")
            .unwrap();
    }
    assert!(Store::open(&partial_home).is_err());
    cleanup(&partial_home);

    let foreign_home = test_home("foreign");
    {
        let store = Store::open(&foreign_home).unwrap();
        store
            .connection
            .execute_batch(
                "CREATE TRIGGER audit_multiplexer_workspace_insert
                 BEFORE INSERT ON multiplexer_workspaces
                 BEGIN
                     SELECT RAISE(ABORT, 'unexpected multiplexer trigger');
                 END;",
            )
            .unwrap();
    }
    assert!(Store::open(&foreign_home).is_err());
    cleanup(&foreign_home);
}

#[test]
fn t164_unrelated_foreign_key_to_multiplexer_table_fails_closed() {
    let home = test_home("foreign-fk");
    {
        let store = Store::open(&home).unwrap();
        store
            .connection
            .execute_batch(
                "CREATE TABLE external_metadata (
                    id TEXT PRIMARY KEY,
                    mux_id TEXT NOT NULL
                        REFERENCES multiplexer_workspaces(multiplexer_workspace_id)
                        ON DELETE CASCADE
                );",
            )
            .unwrap();
    }

    assert!(Store::open(&home).is_err());
    cleanup(&home);
}

#[test]
fn t164_database_capacity_triggers_enforce_global_limits() {
    let home = test_home("capacity-triggers");
    let store = Store::open(&home).unwrap();

    for index in 1_u128..=32 {
        let workspace_id = format!("{index:032x}");
        store
            .connection
            .execute(
                "INSERT INTO multiplexer_workspaces(
                    multiplexer_workspace_id, schema_version, workspace_alias,
                    topology_generation, snapshot_json, created_unix_ms, updated_unix_ms
                 ) VALUES (?1, 1, 'a', 1, '{}', 0, 0)",
                [workspace_id],
            )
            .unwrap();
    }
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO multiplexer_workspaces(
                    multiplexer_workspace_id, schema_version, workspace_alias,
                    topology_generation, snapshot_json, created_unix_ms, updated_unix_ms
                 ) VALUES ('00000000000000000000000000000021', 1, 'a', 1, '{}', 0, 0)",
                [],
            )
            .is_err()
    );

    for index in 1_u128..=128 {
        let template_id = format!("{:032x}", index + 1_000);
        store
            .connection
            .execute(
                "INSERT INTO multiplexer_layout_templates(
                    layout_template_id, schema_version, template_name, template_json,
                    created_unix_ms, updated_unix_ms
                 ) VALUES (?1, 1, 't', '{}', 0, 0)",
                [template_id],
            )
            .unwrap();
    }
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO multiplexer_layout_templates(
                    layout_template_id, schema_version, template_name, template_json,
                    created_unix_ms, updated_unix_ms
                 ) VALUES ('00000000000000000000000000001000', 1, 't', '{}', 0, 0)",
                [],
            )
            .is_err()
    );

    let workspace_id = "00000000000000000000000000000001";
    for index in 0..256 {
        let git_workspace_id = format!("workspace-{index:03}");
        store
            .connection
            .execute(
                "INSERT INTO multiplexer_worktree_memberships(
                    multiplexer_workspace_id, git_workspace_id, membership_source,
                    membership_state, created_unix_ms, last_confirmed_unix_ms
                 ) VALUES (?1, ?2, 'EXPLICIT_USER', 'PRESENT', 0, 0)",
                rusqlite::params![workspace_id, git_workspace_id],
            )
            .unwrap();
    }
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO multiplexer_worktree_memberships(
                    multiplexer_workspace_id, git_workspace_id, membership_source,
                    membership_state, created_unix_ms, last_confirmed_unix_ms
                 ) VALUES (?1, 'workspace-over-cap', 'EXPLICIT_USER', 'PRESENT', 0, 0)",
                [workspace_id],
            )
            .is_err()
    );

    cleanup(&home);
}

#[test]
fn t164_workspace_snapshot_round_trips_across_store_restart_without_live_authority() {
    let home = test_home("snapshot");
    let expected = snapshot("workspace");
    {
        let mut store = Store::open(&home).unwrap();
        store
            .persist_multiplexer_topology_snapshots(std::slice::from_ref(&expected), 10)
            .unwrap();
        let loaded = store
            .load_multiplexer_workspace_snapshot(expected.workspace_id())
            .unwrap();
        assert_eq!(loaded, expected);
        assert_eq!(loaded.to_workspace_state().unwrap().alias, "workspace");
    }

    let reopened = Store::open(&home).unwrap();
    let loaded = reopened
        .load_multiplexer_workspace_snapshot(expected.workspace_id())
        .unwrap();
    assert_eq!(loaded, expected);
    let json = loaded.to_canonical_json().unwrap();
    for forbidden in [
        "runtime_namespace_id",
        "controller",
        "provider_native",
        "process_handle",
        "pid",
        "credential",
        "bearer_token",
        "verification",
        "acceptance",
        "candidate_id",
        "evidence_id",
    ] {
        assert!(!json.contains(forbidden), "{forbidden}");
    }
    cleanup(&home);
}

#[test]
fn t164_topology_snapshot_set_is_atomic_and_rejects_mixed_generation_or_focus() {
    let home = test_home("snapshot-set");
    let first_workspace = workspace(10);
    let second_workspace = workspace(11);
    let mut topology = MultiplexerTopology::empty();
    topology
        .create_workspace(
            topology.generation(),
            first_workspace,
            "first".to_owned(),
            tab(10),
            "first-tab".to_owned(),
            pane(10),
        )
        .unwrap();
    let generation = topology
        .create_workspace(
            topology.generation(),
            second_workspace,
            "second".to_owned(),
            tab(11),
            "second-tab".to_owned(),
            pane(11),
        )
        .unwrap();

    let first = TopologySnapshotV1::from_workspace(
        topology.workspace(first_workspace).unwrap(),
        generation,
        false,
    )
    .unwrap();
    let second = TopologySnapshotV1::from_workspace(
        topology.workspace(second_workspace).unwrap(),
        generation,
        true,
    )
    .unwrap();

    let mut store = Store::open(&home).unwrap();
    store
        .persist_multiplexer_topology_snapshots(&[first.clone(), second.clone()], 10)
        .unwrap();
    assert_eq!(
        store.load_multiplexer_topology_snapshots().unwrap(),
        vec![first.clone(), second.clone()]
    );

    let mixed_generation = TopologySnapshotV1::from_workspace(
        topology.workspace(first_workspace).unwrap(),
        TopologyGeneration::new(generation.get() + 1).unwrap(),
        false,
    )
    .unwrap();
    assert!(
        store
            .persist_multiplexer_topology_snapshots(
                &[mixed_generation, second.clone()],
                11,
            )
            .is_err()
    );
    assert_eq!(
        store.load_multiplexer_topology_snapshots().unwrap(),
        vec![first.clone(), second.clone()]
    );

    let duplicate_focus = TopologySnapshotV1::from_workspace(
        topology.workspace(first_workspace).unwrap(),
        generation,
        true,
    )
    .unwrap();
    assert!(
        store
            .persist_multiplexer_topology_snapshots(
                &[duplicate_focus, second.clone()],
                11,
            )
            .is_err()
    );
    assert_eq!(
        store.load_multiplexer_topology_snapshots().unwrap(),
        vec![first, second]
    );

    cleanup(&home);
}

#[test]
fn t164_snapshot_validation_rejects_oversized_and_corrupt_state_without_truncation() {
    assert!(std::panic::catch_unwind(|| snapshot(&"x".repeat(129))).is_err());

    let home = test_home("corrupt-snapshot");
    let expected = snapshot("valid");
    let mut store = Store::open(&home).unwrap();
    store
        .persist_multiplexer_topology_snapshots(std::slice::from_ref(&expected), 10)
        .unwrap();

    store
        .connection
        .execute(
            "UPDATE multiplexer_workspaces
             SET topology_generation = topology_generation + 1,
                 snapshot_json = '{}',
                 updated_unix_ms = 11
             WHERE multiplexer_workspace_id = ?1",
            [expected.workspace_id().as_hex()],
        )
        .unwrap();
    let error = store
        .load_multiplexer_workspace_snapshot(expected.workspace_id())
        .unwrap_err()
        .to_string();
    assert!(error.contains("snapshot"));

    let oversized = "x".repeat(262_141);
    let direct = store.connection.execute(
        "INSERT INTO multiplexer_layout_templates(
            layout_template_id, schema_version, template_name, template_json,
            created_unix_ms, updated_unix_ms
         ) VALUES (?1, 1, 'oversized', ?2, 0, 0)",
        rusqlite::params![template_id(99).as_hex(), oversized],
    );
    assert!(direct.is_err());

    cleanup(&home);
}

#[test]
fn t164_layout_template_round_trip_is_structure_only_and_rejects_authority_fields() {
    let home = test_home("template");
    let store = Store::open(&home).unwrap();
    let id = template_id(2);
    let value = template();
    store
        .persist_multiplexer_layout_template(id, "two panes", &value, 10)
        .unwrap();
    let (name, loaded) = store.load_multiplexer_layout_template(id).unwrap();
    assert_eq!(name, "two panes");
    assert_eq!(loaded, value);
    assert_eq!(loaded.to_structure().unwrap().tabs.len(), 1);

    let json = value.to_canonical_json().unwrap();
    for forbidden in [
        "multiplexer_workspace_id",
        "tab_id",
        "pane_id",
        "runtime_namespace_id",
        "controller",
        "provider",
        "candidate",
        "evidence",
        "credential",
        "pid",
    ] {
        assert!(!json.contains(forbidden), "{forbidden}");
    }

    let injected = format!(
        "{},\"runtime_namespace_id\":\"{}\"}}",
        &json[..json.len() - 1],
        "1".repeat(32)
    );
    store
        .connection
        .execute(
            "UPDATE multiplexer_layout_templates
             SET template_json = ?2, updated_unix_ms = 11
             WHERE layout_template_id = ?1",
            rusqlite::params![id.as_hex(), injected],
        )
        .unwrap();
    assert!(store.load_multiplexer_layout_template(id).is_err());

    cleanup(&home);
}

#[test]
fn t164_explicit_worktree_membership_is_not_discovery_fk_and_metadata_delete_is_not_git_delete() {
    let home = test_home("membership");
    let mut store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-a",
                canonical_worktree_root: "/tmp/winds-t164-workspace-a",
                git_common_dir: "/tmp/winds-t164-workspace-a/.git",
            },
            1,
        )
        .unwrap();

    let snapshot = snapshot("mux");
    store
        .persist_multiplexer_topology_snapshots(std::slice::from_ref(&snapshot), 10)
        .unwrap();
    let membership = WorktreeMembershipRecord {
        multiplexer_workspace_id: snapshot.workspace_id(),
        git_workspace_id: "workspace-a".to_owned(),
        source: WorktreeMembershipSource::ExplicitUser,
        state: WorktreeMembershipState::Present,
        created_unix_ms: 10,
        last_confirmed_unix_ms: Some(10),
    };
    store
        .persist_multiplexer_worktree_membership(&membership)
        .unwrap();
    assert_eq!(
        store
            .load_multiplexer_worktree_membership(snapshot.workspace_id(), "workspace-a")
            .unwrap(),
        membership
    );

    store
        .connection
        .execute(
            "DELETE FROM workspaces WHERE workspace_id = 'workspace-a'",
            [],
        )
        .unwrap();
    assert_eq!(
        store
            .load_multiplexer_worktree_membership(snapshot.workspace_id(), "workspace-a")
            .unwrap(),
        membership
    );

    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-a",
                canonical_worktree_root: "/tmp/winds-t164-workspace-a",
                git_common_dir: "/tmp/winds-t164-workspace-a/.git",
            },
            20,
        )
        .unwrap();
    store
        .delete_multiplexer_workspace_metadata(snapshot.workspace_id())
        .unwrap();
    let count: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM workspaces WHERE workspace_id = 'workspace-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let membership_count: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM multiplexer_worktree_memberships",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(membership_count, 0);

    cleanup(&home);
}

#[test]
fn t164_repository_trust_revision_and_time_are_monotonic() {
    let home = test_home("trust");
    let store = Store::open(&home).unwrap();

    let first = RepositoryTrustRecord {
        repository_identity: "repo-identity".to_owned(),
        canonical_git_common_dir: "/tmp/repo/.git".to_owned(),
        revision: 1,
        created_unix_ms: 10,
        updated_unix_ms: 10,
    };
    store.persist_multiplexer_repository_trust(&first).unwrap();

    let second = RepositoryTrustRecord {
        revision: 2,
        updated_unix_ms: 20,
        ..first.clone()
    };
    store.persist_multiplexer_repository_trust(&second).unwrap();
    assert_eq!(
        store
            .load_multiplexer_repository_trust("repo-identity")
            .unwrap(),
        second
    );

    assert!(
        store
            .persist_multiplexer_repository_trust(&RepositoryTrustRecord {
                revision: 1,
                updated_unix_ms: 21,
                ..first.clone()
            })
            .is_err()
    );
    assert!(
        store
            .persist_multiplexer_repository_trust(&RepositoryTrustRecord {
                canonical_git_common_dir: "/tmp/other/.git".to_owned(),
                revision: 2,
                updated_unix_ms: 22,
                ..first
            })
            .is_err()
    );

    cleanup(&home);
}

#[test]
fn t164_schema_has_no_live_process_or_secret_authority_and_prior_migration_sizes_are_frozen() {
    assert_eq!(
        include_bytes!("../migrations/0011_model_mesh_continuity.sql").len(),
        20_238
    );
    assert_eq!(
        include_bytes!("../migrations/0012_desktop_presentation.sql").len(),
        4_660
    );
    assert_eq!(
        include_bytes!("../migrations/0013_persistent_runtime_owner.sql").len(),
        6_680
    );

    let migration =
        include_str!("../migrations/0014_workspace_multiplexer.sql").to_ascii_lowercase();
    let persistence = include_str!("multiplexer/persistence.rs").to_ascii_lowercase();
    for forbidden in [
        "runtime_namespace_id",
        "process_handle",
        "pty_handle",
        "controller_lease",
        "credential",
        "bearer_token",
        "terminal_transcript",
        "provider_native_session_id",
        "candidate_id",
        "evidence_id",
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
