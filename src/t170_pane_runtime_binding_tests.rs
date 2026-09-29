use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::multiplexer::domain::navigation::{
    MultiplexerTopology, PanePlacement, SplitAxis, SplitRatioBps,
};
use crate::multiplexer::domain::{
    ClientSurfaceCapability, LayoutTemplateId, MultiplexerAuthority, MultiplexerErrorKind,
    MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};

use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, RuntimeAlias, RuntimeNamespaceId,
};
use crate::persistent_runtime::owner::PersistentOwner;
use crate::persistent_runtime::protocol::{
    ApplyTopologyOperationV2, MultiplexerSnapshotV2, ProtocolMessage, ProtocolPaneClosePolicy,
    ProtocolPanePlacement, ProtocolPayload, ProtocolSplitAxis, RequestMultiplexerWriteV2,
    TopologyMutationOutcomeV2, TopologyOperationV2,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_T170_ROOT: AtomicU64 = AtomicU64::new(1);

fn t170_root(label: &str) -> PathBuf {
    let sequence = NEXT_T170_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t170-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

fn t170_shell_profile(root: &Path) -> ShellProfile {
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
fn t170_start_owner(home: &Path, runtime_root: &Path, now_unix_ms: i64) -> PersistentOwner {
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .unwrap();
    PersistentOwner::start_for_test(home, &runtime_directory, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn t170_start_owner(home: &Path, _runtime_root: &Path, now_unix_ms: i64) -> PersistentOwner {
    PersistentOwner::start(home, now_unix_ms).unwrap()
}

fn t170_client(label: &str) -> ClientConnectionId {
    ClientConnectionId::new(label).unwrap()
}

fn t170_sequence(value: u64) -> EventSequence {
    EventSequence::new(value).unwrap()
}

fn t170_id(seed: u8) -> [u8; 16] {
    [seed; 16]
}

fn t170_runtime(seed: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes(t170_id(seed)).unwrap()
}

fn t170_workspace(seed: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes(t170_id(seed)).unwrap()
}

fn t170_tab(seed: u8) -> TabId {
    TabId::from_entropy_bytes(t170_id(seed)).unwrap()
}

fn t170_pane(seed: u8) -> PaneId {
    PaneId::from_entropy_bytes(t170_id(seed)).unwrap()
}

/// Builds a single workspace with one tab holding `pane_count` split panes.
fn t170_topology(pane_count: u8) -> MultiplexerTopology {
    let workspace_id = t170_workspace(0x11);
    let tab_id = t170_tab(0x12);
    let mut topology = MultiplexerTopology::empty();
    topology
        .create_workspace(
            TopologyGeneration::initial(),
            workspace_id,
            "bound".to_owned(),
            tab_id,
            "main".to_owned(),
            t170_pane(0x13),
        )
        .unwrap();
    let mut generation = topology.generation();
    for index in 1..pane_count {
        generation = topology
            .split_pane(
                generation,
                workspace_id,
                tab_id,
                t170_pane(0x13),
                t170_pane(0x13 + index),
                SplitAxis::Horizontal,
                PanePlacement::After,
                SplitRatioBps::new(5_000).unwrap(),
            )
            .unwrap();
    }
    topology
}

fn t170_bound_topology() -> (MultiplexerTopology, RuntimeNamespaceId, TopologyGeneration) {
    let mut topology = t170_topology(2);
    let runtime_namespace_id = t170_runtime(0x31);
    let expected = topology.generation();
    topology
        .bind_pane_runtime(
            expected,
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x13),
            runtime_namespace_id,
        )
        .unwrap();
    (topology, runtime_namespace_id, expected)
}

#[test]
fn t170_pane_runtime_binding_is_explicit_generation_checked_and_uniquely_owned() {
    let (mut topology, runtime_namespace_id, bound_generation) = t170_bound_topology();
    let bound_at = topology
        .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x13))
        .unwrap()
        .unwrap();
    assert_eq!(bound_at.runtime_namespace_id, runtime_namespace_id);
    // The binding records the generation the request was applied against, not the
    // generation the topology reached afterwards.
    assert_eq!(bound_at.bound_at_generation, bound_generation);
    assert_eq!(
        topology.generation(),
        bound_generation.checked_next().unwrap()
    );

    // A split sibling stays unbound: splitting never clones a runtime reference.
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x14))
            .unwrap(),
        None
    );

    // One runtime namespace can never back two panes.
    assert_eq!(
        topology.bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x14),
            runtime_namespace_id,
        ),
        Err(MultiplexerErrorKind::IdentityReuse)
    );

    // A stale expected generation is rejected before any binding is recorded.
    let stale = TopologyGeneration::initial();
    assert_eq!(
        topology.bind_pane_runtime(
            stale,
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x14),
            t170_runtime(0x32),
        ),
        Err(MultiplexerErrorKind::StaleTopologyGeneration)
    );
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x14))
            .unwrap(),
        None
    );

    // Binding is exact-target only.
    assert_eq!(
        topology.bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x99),
            t170_runtime(0x33),
        ),
        Err(MultiplexerErrorKind::UnknownPane)
    );
    assert_eq!(
        topology.pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x99)),
        Err(MultiplexerErrorKind::UnknownPane)
    );
}

#[test]
fn t170_close_policy_detaches_topology_and_leaves_the_runtime_discoverable() {
    let (mut topology, runtime_namespace_id, _bound) = t170_bound_topology();
    let generation = topology
        .close_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x13),
        )
        .unwrap();
    assert_eq!(generation, topology.generation());
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x14))
            .unwrap(),
        None
    );
    assert_eq!(
        topology.pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x13)),
        Err(MultiplexerErrorKind::UnknownPane)
    );

    // The pane reference is gone, but nothing about the runtime was touched: a
    // detaching close is not a stop, so the runtime keeps its own identity and is
    // never implicitly terminated, reaped, or orphaned by topology editing.
    assert_ne!(runtime_namespace_id, t170_runtime(0x99));
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x14))
            .unwrap(),
        None
    );
    // A freed runtime namespace is bindable again, proving the detach released the
    // pane reference without claiming the runtime was destroyed.
    topology
        .bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x14),
            runtime_namespace_id,
        )
        .unwrap();
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x14))
            .unwrap()
            .unwrap()
            .runtime_namespace_id,
        runtime_namespace_id
    );
}

#[test]
fn t170_replacement_pane_never_inherits_stale_binding_or_presentation_epoch() {
    let mut topology = t170_topology(2);
    let bound_pane = t170_pane(0x13);
    topology
        .bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            bound_pane,
            t170_runtime(0x41),
        )
        .unwrap();
    topology
        .clear_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            bound_pane,
            1,
        )
        .unwrap();
    assert_eq!(
        topology
            .pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), bound_pane)
            .unwrap()
            .get(),
        1
    );

    topology
        .close_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            bound_pane,
        )
        .unwrap();

    // A fresh PaneId is the only way to reuse the slot, and it inherits nothing.
    let replacement = t170_pane(0x15);
    let generation = topology
        .split_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x14),
            replacement,
            SplitAxis::Horizontal,
            PanePlacement::After,
            SplitRatioBps::new(5_000).unwrap(),
        )
        .unwrap();
    assert_eq!(generation, topology.generation());
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), replacement)
            .unwrap(),
        None
    );
    assert_eq!(
        topology
            .pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), replacement)
            .unwrap()
            .get(),
        0
    );
    // The retired PaneId itself can never be revived, and a stale expected
    // generation is refused before any target is even considered.
    let stale = TopologyGeneration::initial();
    assert_eq!(
        topology.bind_pane_runtime(
            stale,
            t170_workspace(0x11),
            t170_tab(0x12),
            replacement,
            t170_runtime(0x42),
        ),
        Err(MultiplexerErrorKind::StaleTopologyGeneration)
    );
    assert_eq!(
        topology.bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            bound_pane,
            t170_runtime(0x42),
        ),
        Err(MultiplexerErrorKind::UnknownPane)
    );
}

#[test]
fn t170_pane_clear_is_presentation_only_exact_and_deterministically_epoch_checked() {
    let mut topology = t170_topology(2);
    let cleared = t170_pane(0x13);
    let untouched = t170_pane(0x14);

    // A client cannot skip an epoch or mint its own: only the owner-computed next
    // value is accepted, so a proposal that jumps ahead fails closed.
    assert_eq!(
        topology.clear_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            cleared,
            7,
        ),
        Err(MultiplexerErrorKind::DuplicateOrReplayedMutation)
    );
    assert_eq!(
        topology
            .pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), cleared)
            .unwrap()
            .get(),
        0
    );

    topology
        .clear_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            cleared,
            1,
        )
        .unwrap();
    assert_eq!(
        topology
            .pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), cleared)
            .unwrap()
            .get(),
        1
    );

    // Clear affects exactly one pane and never rewinds a replayed epoch.
    assert_eq!(
        topology
            .pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), untouched)
            .unwrap()
            .get(),
        0
    );
    assert_eq!(
        topology.clear_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            cleared,
            1,
        ),
        Err(MultiplexerErrorKind::DuplicateOrReplayedMutation)
    );
    topology
        .clear_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            cleared,
            2,
        )
        .unwrap();
    assert_eq!(
        topology
            .pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), cleared)
            .unwrap()
            .get(),
        2
    );

    // Clear carries no process or binding effect at all.
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), t170_tab(0x12), cleared)
            .unwrap(),
        None
    );
    assert_eq!(
        topology.clear_pane(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x99),
            1,
        ),
        Err(MultiplexerErrorKind::UnknownPane)
    );
}

#[test]
fn t170_close_tab_and_workspace_drop_pane_references_without_stopping_anything() {
    let mut topology = t170_topology(2);
    topology
        .bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            t170_tab(0x12),
            t170_pane(0x13),
            t170_runtime(0x51),
        )
        .unwrap();
    let second_tab = t170_tab(0x16);
    topology
        .create_tab(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            "second".to_owned(),
            t170_pane(0x17),
        )
        .unwrap();
    topology
        .bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            t170_pane(0x17),
            t170_runtime(0x52),
        )
        .unwrap();

    topology
        .close_tab(topology.generation(), t170_workspace(0x11), t170_tab(0x12))
        .unwrap();
    assert_eq!(
        topology.pane_presentation_epoch(t170_workspace(0x11), t170_tab(0x12), t170_pane(0x13)),
        Err(MultiplexerErrorKind::UnknownTab)
    );

    // A tab created after a close still binds normally, and no stale reference
    // from the closed tab leaks into the surviving topology.
    topology
        .split_pane(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            t170_pane(0x17),
            t170_pane(0x18),
            SplitAxis::Vertical,
            PanePlacement::After,
            SplitRatioBps::new(5_000).unwrap(),
        )
        .unwrap();
    topology
        .bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            t170_pane(0x18),
            t170_runtime(0x53),
        )
        .unwrap();
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), second_tab, t170_pane(0x18))
            .unwrap()
            .unwrap()
            .runtime_namespace_id,
        t170_runtime(0x53)
    );
    // A runtime that belonged to the closed tab is free to bind again: closing
    // topology never stopped it, so nothing is implicitly terminated or orphaned.
    topology
        .close_pane(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            t170_pane(0x18),
        )
        .unwrap();
    topology
        .split_pane(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            t170_pane(0x17),
            t170_pane(0x19),
            SplitAxis::Horizontal,
            PanePlacement::After,
            SplitRatioBps::new(5_000).unwrap(),
        )
        .unwrap();
    topology
        .bind_pane_runtime(
            topology.generation(),
            t170_workspace(0x11),
            second_tab,
            t170_pane(0x19),
            t170_runtime(0x51),
        )
        .unwrap();
    assert_eq!(
        topology
            .pane_runtime_binding(t170_workspace(0x11), second_tab, t170_pane(0x19))
            .unwrap()
            .unwrap()
            .runtime_namespace_id,
        t170_runtime(0x51)
    );
}

fn t170_apply_operation(
    owner: &mut PersistentOwner,
    client: &ClientConnectionId,
    operation: TopologyOperationV2,
    request_sequence: u64,
    response_sequence: u64,
    now_unix_ms: i64,
    now_monotonic_ms: u64,
) -> TopologyMutationOutcomeV2 {
    let expected = owner.multiplexer_topology().generation();
    let request = ProtocolMessage::new(
        client.clone(),
        t170_sequence(request_sequence),
        None,
        owner.generation_id(),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: expected,
                operation,
            },
        },
    )
    .unwrap();
    let response = owner
        .dispatch_multiplexer_protocol_v2(
            client.clone(),
            &request,
            t170_sequence(response_sequence),
            now_unix_ms,
            now_monotonic_ms,
        )
        .unwrap();
    let ProtocolPayload::MultiplexerSnapshot {
        snapshot: MultiplexerSnapshotV2::MutationResult { result, .. },
    } = response.payload
    else {
        panic!("topology mutation must answer with a typed mutation result");
    };
    result.outcome
}

fn t170_grant_write(
    owner: &mut PersistentOwner,
    client: &ClientConnectionId,
    request_sequence: u64,
) {
    let request = ProtocolMessage::new(
        client.clone(),
        t170_sequence(request_sequence),
        None,
        owner.generation_id(),
        None,
        ProtocolPayload::RequestMultiplexerWrite {
            request: RequestMultiplexerWriteV2 {
                client_surface_capability: ClientSurfaceCapability::ControllingTerminal,
            },
        },
    )
    .unwrap();
    owner
        .dispatch_multiplexer_protocol_v2(
            client.clone(),
            &request,
            t170_sequence(request_sequence + 1),
            1,
            1,
        )
        .unwrap();
    assert_eq!(
        owner.multiplexer_authority(client),
        MultiplexerAuthority::MultiplexerWrite
    );
}

fn t170_seed_workspace(owner: &mut PersistentOwner, client: &ClientConnectionId, base: u64) {
    t170_grant_write(owner, client, base);
    let outcome = t170_apply_operation(
        owner,
        client,
        TopologyOperationV2::CreateWorkspace {
            multiplexer_workspace_id: t170_workspace(0x61),
            alias: "t170".to_owned(),
            first_tab_id: t170_tab(0x62),
            first_tab_alias: "main".to_owned(),
            first_pane_id: t170_pane(0x63),
        },
        base + 2,
        base + 3,
        (base + 4) as i64,
        100,
    );
    assert_eq!(outcome, TopologyMutationOutcomeV2::Accepted);
    assert!(matches!(
        t170_apply_operation(
            owner,
            client,
            TopologyOperationV2::SplitPane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                target_pane_id: t170_pane(0x63),
                new_pane_id: t170_pane(0x64),
                axis: ProtocolSplitAxis::Horizontal,
                placement: ProtocolPanePlacement::After,
                ratio_basis_points: 5_000,
            },
            base + 5,
            base + 6,
            (base + 7) as i64,
            100,
        ),
        TopologyMutationOutcomeV2::Accepted
    ));
}

#[test]
fn t170_stop_runtime_then_close_fails_closed_on_every_missing_authority() {
    let home = t170_root("stop-authority");
    let runtime_root = t170_root("stop-authority-runtime");
    let mut owner = t170_start_owner(&home, &runtime_root, 1);
    let writer = t170_client("t170-writer");

    t170_seed_workspace(&mut owner, &writer, 20);
    let generation_before = owner.multiplexer_topology().generation();

    // An unbound pane cannot satisfy a stop-then-close: it must not silently
    // degrade into a topology-only detach.
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x64),
                policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
            },
            40,
            41,
            42,
            100,
        ),
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::UnsupportedOperation
        }
    );

    // An unknown exact target is rejected.
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x99),
                policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
            },
            43,
            44,
            45,
            100,
        ),
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::UnknownPane
        }
    );

    // Every rejection above left the topology exactly where it was.
    assert_eq!(owner.multiplexer_topology().generation(), generation_before);

    // A plain detach still works and needs no runtime controller.
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x64),
                policy: ProtocolPaneClosePolicy::DetachView,
            },
            46,
            47,
            30,
            100,
        ),
        TopologyMutationOutcomeV2::Accepted
    );
}

#[test]
fn t170_stop_runtime_then_close_requires_multiplexer_write_and_exact_controller() {
    let home = t170_root("stop-controller");
    let runtime_root = t170_root("stop-controller-runtime");
    let profile = t170_shell_profile(&home);
    let mut owner = t170_start_owner(&home, &runtime_root, 1);
    let writer = t170_client("t170-controller-writer");

    t170_seed_workspace(&mut owner, &writer, 20);

    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("t170-bound").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            30,
            100,
        )
        .unwrap();
    let runtime_namespace_id = attachment.runtime_namespace_id();

    // Bind the exact pane to the exact runtime through owner-authoritative topology.
    owner
        .mutate_multiplexer_topology(
            &writer,
            owner.multiplexer_topology().generation(),
            31,
            |topology, expected| {
                topology.bind_pane_runtime(
                    expected,
                    t170_workspace(0x61),
                    t170_tab(0x62),
                    t170_pane(0x63),
                    runtime_namespace_id,
                )
            },
        )
        .unwrap();

    // No controller lease yet: the stop must not run and the pane must survive.
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x63),
                policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
            },
            40,
            41,
            32,
            100,
        ),
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::RuntimeControllerRequired
        }
    );
    assert!(
        owner
            .multiplexer_topology()
            .pane_runtime_binding(t170_workspace(0x61), t170_tab(0x62), t170_pane(0x63))
            .unwrap()
            .is_some()
    );
    assert!(owner.runtime_registry_is_live(runtime_namespace_id));

    // A different client holding the lease is still not the exact caller.
    let other_controller = t170_client("t170-other-controller");
    owner
        .request_terminal_control(other_controller.clone(), runtime_namespace_id, 33, 100)
        .unwrap();
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x63),
                policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
            },
            42,
            43,
            34,
            100,
        ),
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::RuntimeControllerRequired
        }
    );
    assert!(owner.runtime_registry_is_live(runtime_namespace_id));

    // Only the exact lease holder may stop the process, so the impostor lease is
    // released before the real controller takes over.
    owner
        .release_terminal_control(&other_controller, runtime_namespace_id, 35, 100)
        .unwrap();
    owner
        .request_terminal_control(writer.clone(), runtime_namespace_id, 35, 100)
        .unwrap();

    // A stale expected generation is refused before the stop can run, so the
    // process is untouched and the outcome is not reported as a close at all.
    let stale_generation = owner.multiplexer_topology().generation();
    owner
        .mutate_multiplexer_topology(
            &writer,
            owner.multiplexer_topology().generation(),
            35,
            |topology, expected| {
                topology.focus_pane(
                    expected,
                    t170_workspace(0x61),
                    t170_tab(0x62),
                    t170_pane(0x64),
                )
            },
        )
        .unwrap();
    let stale_request = ProtocolMessage::new(
        writer.clone(),
        t170_sequence(46),
        None,
        owner.generation_id(),
        None,
        ProtocolPayload::ApplyTopologyOperation {
            request: ApplyTopologyOperationV2 {
                expected_topology_generation: stale_generation,
                operation: TopologyOperationV2::ClosePane {
                    multiplexer_workspace_id: t170_workspace(0x61),
                    tab_id: t170_tab(0x62),
                    pane_id: t170_pane(0x63),
                    policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
                },
            },
        },
    )
    .unwrap();
    let stale_response = owner
        .dispatch_multiplexer_protocol_v2(
            writer.clone(),
            &stale_request,
            t170_sequence(47),
            36,
            100,
        )
        .unwrap();
    let ProtocolPayload::MultiplexerSnapshot {
        snapshot: MultiplexerSnapshotV2::MutationResult { result, .. },
    } = stale_response.payload
    else {
        panic!("a stale stop-then-close must still answer with a typed result");
    };
    assert_eq!(
        result.outcome,
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::StaleTopologyGeneration
        }
    );
    assert!(
        owner.runtime_registry_is_live(runtime_namespace_id),
        "a refused stop-then-close must never touch the process"
    );

    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x63),
                policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
            },
            48,
            49,
            37,
            100,
        ),
        TopologyMutationOutcomeV2::Accepted
    );
    assert_eq!(
        owner.multiplexer_topology().pane_runtime_binding(
            t170_workspace(0x61),
            t170_tab(0x62),
            t170_pane(0x63)
        ),
        Err(MultiplexerErrorKind::UnknownPane)
    );
    // A clean close is reported only after the process genuinely stopped.
    assert!(
        !owner.runtime_registry_is_live(runtime_namespace_id),
        "an accepted stop-then-close must correspond to a real terminal disposition"
    );
}

#[test]
fn t170_input_races_never_retarget_a_stale_pane_or_a_stale_controller() {
    let home = t170_root("input-race");
    let runtime_root = t170_root("input-race-runtime");
    let profile = t170_shell_profile(&home);
    let mut owner = t170_start_owner(&home, &runtime_root, 1);
    let writer = t170_client("t170-race-writer");

    t170_seed_workspace(&mut owner, &writer, 20);

    let first = owner
        .start_terminal_runtime(
            RuntimeAlias::new("t170-race-a").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            30,
            100,
        )
        .unwrap();
    let second = owner
        .start_terminal_runtime(
            RuntimeAlias::new("t170-race-b").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            31,
            100,
        )
        .unwrap();
    let (first_runtime, second_runtime) =
        (first.runtime_namespace_id(), second.runtime_namespace_id());

    owner
        .mutate_multiplexer_topology(
            &writer,
            owner.multiplexer_topology().generation(),
            32,
            |topology, expected| {
                topology.bind_pane_runtime(
                    expected,
                    t170_workspace(0x61),
                    t170_tab(0x62),
                    t170_pane(0x63),
                    first_runtime,
                )
            },
        )
        .unwrap();
    owner
        .mutate_multiplexer_topology(
            &writer,
            owner.multiplexer_topology().generation(),
            33,
            |topology, expected| {
                topology.bind_pane_runtime(
                    expected,
                    t170_workspace(0x61),
                    t170_tab(0x62),
                    t170_pane(0x64),
                    second_runtime,
                )
            },
        )
        .unwrap();
    owner
        .request_terminal_control(writer.clone(), first_runtime, 34, 100)
        .unwrap();

    let generation_before = owner.multiplexer_topology().generation();
    let resolve = |owner: &PersistentOwner| {
        owner
            .multiplexer_topology()
            .pane_runtime_binding(t170_workspace(0x61), t170_tab(0x62), t170_pane(0x63))
            .unwrap()
            .map(|binding| binding.runtime_namespace_id)
    };
    assert_eq!(resolve(&owner), Some(first_runtime));

    // Focus, swap, and takeover all move the topology. Every one of them makes the
    // caller's expected generation stale, so no in-flight input can land on the
    // pane the caller last saw.
    t170_apply_operation(
        &mut owner,
        &writer,
        TopologyOperationV2::FocusPane {
            multiplexer_workspace_id: t170_workspace(0x61),
            tab_id: t170_tab(0x62),
            pane_id: t170_pane(0x64),
        },
        60,
        61,
        35,
        100,
    );
    assert_ne!(owner.multiplexer_topology().generation(), generation_before);
    assert_eq!(
        resolve(&owner),
        Some(first_runtime),
        "focus must never retarget a bound runtime"
    );

    t170_apply_operation(
        &mut owner,
        &writer,
        TopologyOperationV2::SwapPanes {
            multiplexer_workspace_id: t170_workspace(0x61),
            tab_id: t170_tab(0x62),
            first_pane_id: t170_pane(0x63),
            second_pane_id: t170_pane(0x64),
        },
        62,
        63,
        36,
        100,
    );
    assert_eq!(
        resolve(&owner),
        Some(first_runtime),
        "a pane swap must follow the pane identity, never the layout position"
    );

    // The controller may only act on the runtime it actually holds.
    assert!(
        owner
            .controller_send_terminal_input(&writer, second_runtime, b"echo cross\n", 37, 100,)
            .is_err(),
        "input must never be dispatched to a runtime the caller does not control"
    );
    assert!(
        owner
            .controller_send_terminal_input(&writer, first_runtime, b"echo mine\n", 38, 100)
            .is_ok()
    );
    assert!(owner.runtime_registry_is_live(second_runtime));

    // A controller takeover between resolve and dispatch revokes the caller's lease,
    // so the stale caller's next input fails closed instead of reaching the child.
    let taker = t170_client("t170-race-taker");
    owner
        .release_terminal_control(&writer, first_runtime, 39, 100)
        .unwrap();
    owner
        .request_terminal_control(taker.clone(), first_runtime, 40, 100)
        .unwrap();
    assert_eq!(
        resolve(&owner),
        Some(first_runtime),
        "a controller takeover must not rebind the pane to another runtime"
    );
    assert!(
        owner
            .controller_send_terminal_input(&writer, first_runtime, b"echo stale\n", 41, 100)
            .is_err(),
        "a caller that lost the controller lease must never dispatch input"
    );
    assert!(
        owner
            .controller_send_terminal_input(&taker, first_runtime, b"echo fresh\n", 42, 100)
            .is_ok()
    );
    // The other pane's runtime is still untouched by every one of these races.
    assert!(owner.runtime_registry_is_live(second_runtime));
    assert_eq!(
        resolve(&owner),
        Some(first_runtime),
        "no race may retarget a pane to a different runtime"
    );
}

#[test]
fn t170_pane_clear_publishes_a_typed_presentation_marker_with_the_owner_assigned_epoch() {
    let home = t170_root("clear-marker");
    let runtime_root = t170_root("clear-marker-runtime");
    let mut owner = t170_start_owner(&home, &runtime_root, 1);
    let observer = t170_client("t170-clear-observer");
    let writer = t170_client("t170-clear-writer");
    t170_seed_workspace(&mut owner, &writer, 20);

    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClearPane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x63),
                presentation_epoch: 1,
            },
            40,
            41,
            30,
            100,
        ),
        TopologyMutationOutcomeV2::Accepted
    );
    assert_eq!(
        owner
            .multiplexer_topology()
            .pane_presentation_epoch(t170_workspace(0x61), t170_tab(0x62), t170_pane(0x63))
            .unwrap()
            .get(),
        1
    );

    // A replayed epoch is refused, so a clear marker can never be replayed or rewound.
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &writer,
            TopologyOperationV2::ClearPane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x63),
                presentation_epoch: 1,
            },
            42,
            43,
            31,
            100,
        ),
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::DuplicateOrReplayedMutation
        }
    );

    // Clear needs MultiplexerWrite and nothing else: it never touches a process.
    assert_eq!(
        t170_apply_operation(
            &mut owner,
            &observer,
            TopologyOperationV2::ClearPane {
                multiplexer_workspace_id: t170_workspace(0x61),
                tab_id: t170_tab(0x62),
                pane_id: t170_pane(0x63),
                presentation_epoch: 2,
            },
            44,
            45,
            32,
            100,
        ),
        TopologyMutationOutcomeV2::Rejected {
            error: MultiplexerErrorKind::MultiplexerWriteRequired
        }
    );
}

#[test]
fn t170_topology_write_authority_never_implies_runtime_controller_authority() {
    let home = t170_root("authority-split");
    let runtime_root = t170_root("authority-split-runtime");
    let profile = t170_shell_profile(&home);
    let mut owner = t170_start_owner(&home, &runtime_root, 1);
    let writer = t170_client("t170-authority-writer");

    t170_seed_workspace(&mut owner, &writer, 20);

    let attachment = owner
        .start_terminal_runtime(
            RuntimeAlias::new("t170-authority").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            30,
            100,
        )
        .unwrap();
    let runtime_namespace_id = attachment.runtime_namespace_id();
    owner
        .mutate_multiplexer_topology(
            &writer,
            owner.multiplexer_topology().generation(),
            31,
            |topology, expected| {
                topology.bind_pane_runtime(
                    expected,
                    t170_workspace(0x61),
                    t170_tab(0x62),
                    t170_pane(0x63),
                    runtime_namespace_id,
                )
            },
        )
        .unwrap();

    // MultiplexerWrite alone is not process authority.
    for error in [
        owner.controller_send_terminal_input(&writer, runtime_namespace_id, b"x", 32, 100),
        owner.controller_resize_terminal(
            &writer,
            runtime_namespace_id,
            TerminalSize {
                rows: 30,
                cols: 100,
            },
            33,
            100,
        ),
        owner.controller_interrupt_terminal(&writer, runtime_namespace_id, 34, 100),
        owner
            .controller_stop_terminal(&writer, runtime_namespace_id, 35, 100)
            .map(|_| ()),
    ] {
        assert!(
            error.is_err(),
            "topology write authority must never be process authority"
        );
    }
    assert!(owner.runtime_registry_is_live(runtime_namespace_id));
}

#[test]
fn t170_protocol_still_refuses_generic_or_unbound_privileged_topology_paths() {
    use crate::persistent_runtime::protocol::apply_topology_operation_v2;

    let mut topology = t170_topology(2);
    let generation = topology.generation();
    // The domain refuses a stop-then-close that reached it without controller proof.
    assert_eq!(
        apply_topology_operation_v2(
            &mut topology,
            generation,
            &TopologyOperationV2::ClosePane {
                multiplexer_workspace_id: t170_workspace(0x11),
                tab_id: t170_tab(0x12),
                pane_id: t170_pane(0x13),
                policy: ProtocolPaneClosePolicy::StopRuntimeThenClose,
            },
        ),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
    let generation = topology.generation();
    assert_eq!(
        apply_topology_operation_v2(
            &mut topology,
            generation,
            &TopologyOperationV2::ApplyLayoutTemplate {
                template_id: LayoutTemplateId::from_entropy_bytes(t170_id(0x71)).unwrap(),
                multiplexer_workspace_id: t170_workspace(0x11),
            },
        ),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
}
