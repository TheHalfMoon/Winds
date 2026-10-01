//! Focused T173 evidence for bounded agent observations.
//!
//! The acceptance fixtures are the four the task names, plus the identity,
//! bound, and budget properties that make them true rather than incidental.

use super::*;
use crate::multiplexer::domain::TopologyGeneration;
use crate::multiplexer::domain::navigation::{
    LayoutNode, MultiplexerTopology, SplitAxis, SplitRatioBps, TabState, WorkspaceState,
};
use crate::persistent_runtime::domain::RuntimeNamespaceId;
use crate::persistent_runtime::domain::{ClientConnectionId, EventSequence};
use crate::persistent_runtime::protocol::{
    MAX_CONTROL_FRAME_BYTES, MAX_V2_AGENT_OBSERVATIONS_PER_PAGE, MAX_V2_EVIDENCE_SUMMARY_BYTES,
    MAX_V2_PROVIDER_SESSION_ID_BYTES, MessageKind, ProtocolMessage, ProtocolPayload, encode_frame,
};

/// An owner(0xa1) generation for the store under test, and one that is not.
/// An owner generation for the store under test. The same constructor also
/// produces the foreign generation used to prove owner-generation refusal.
fn owner(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte.max(1); 16]).expect("non-zero entropy is accepted")
}

fn id(byte: u8) -> AgentObservationId {
    AgentObservationId::from_entropy_bytes([byte.max(1); 16]).unwrap_or_else(|_| {
        AgentObservationId::from_entropy_bytes([1; 16]).expect("non-zero entropy is accepted")
    })
}

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte.max(1); 16])
        .expect("non-zero entropy is accepted")
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte.max(1); 16]).expect("non-zero entropy is accepted")
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte.max(1); 16]).expect("non-zero entropy is accepted")
}

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte.max(1); 16]).expect("non-zero entropy is accepted")
}

/// One workspace, one tab, split into two live panes.
///
/// Two panes are needed because the topology refuses to close the last pane in a
/// tab, and a close is the only owner-accepted way to retire a pane identity.
fn topology_with(pane_id: PaneId) -> (MultiplexerTopology, MultiplexerWorkspaceId, TabId) {
    topology_with_sibling(pane_id, pane(0xee))
}

fn topology_with_sibling(
    pane_id: PaneId,
    sibling: PaneId,
) -> (MultiplexerTopology, MultiplexerWorkspaceId, TabId) {
    let workspace_id = workspace(0x10);
    let tab_id = tab(0x11);
    let root = LayoutNode::Split {
        axis: SplitAxis::Horizontal,
        ratio_bps: SplitRatioBps::new(5_000).expect("a half split is valid"),
        first: Box::new(LayoutNode::Pane(pane_id)),
        second: Box::new(LayoutNode::Pane(sibling)),
    };
    let state = WorkspaceState {
        id: workspace_id,
        alias: "dev".to_owned(),
        tabs: vec![TabState {
            id: tab_id,
            alias: "main".to_owned(),
            root,
            focused_pane_id: pane_id,
            zoomed_pane_id: None,
        }],
        focused_tab_id: tab_id,
    };
    let topology = MultiplexerTopology::restore_presentation(
        TopologyGeneration::initial(),
        vec![state],
        Some(workspace_id),
    )
    .expect("a single live workspace is restorable");
    (topology, workspace_id, tab_id)
}

fn candidate(
    workspace_id: MultiplexerWorkspaceId,
    tab_id: TabId,
    pane_id: PaneId,
    observation_id: AgentObservationId,
    detection: AgentDetection,
) -> ObservationCandidate {
    candidate_with_ids(
        workspace_id,
        tab_id,
        pane_id,
        vec![observation_id],
        detection,
    )
}

/// A candidate carrying one observation identity per candidate family.
///
/// An ambiguous or unavailable classification names several families, and the
/// frozen page validator rejects a page that repeats an observation identity, so
/// a multi-family outcome must be offered one identity per family.
fn candidate_with_ids(
    workspace_id: MultiplexerWorkspaceId,
    tab_id: TabId,
    pane_id: PaneId,
    observation_ids: Vec<AgentObservationId>,
    detection: AgentDetection,
) -> ObservationCandidate {
    ObservationCandidate {
        observation_ids,
        multiplexer_workspace_id: workspace_id,
        tab_id,
        pane_id,
        detection,
        source_class: AgentObservationSourceV2::OwnedProcessMetadata,
        provider_native_session_id: None,
        git_workspace_id: None,
        observed_unix_ms: 1_700_000_000_000,
    }
}

fn observed(family: AgentFamily) -> AgentDetection {
    AgentDetection::Observed {
        family,
        source: crate::multiplexer::agent_catalog::DetectionSource::StructuredMetadataNamespace,
        support: crate::multiplexer::agent_catalog::AgentSupport::DetectionOnly,
    }
}

fn all_families() -> Vec<AgentFamily> {
    vec![
        AgentFamily::Pi,
        AgentFamily::Claude,
        AgentFamily::Codex,
        AgentFamily::Gemini,
        AgentFamily::Cursor,
        AgentFamily::Devin,
        AgentFamily::Antigravity,
        AgentFamily::Cline,
        AgentFamily::Omp,
        AgentFamily::Mastracode,
        AgentFamily::OpenCode,
        AgentFamily::GithubCopilot,
        AgentFamily::Kimi,
        AgentFamily::Kiro,
        AgentFamily::Droid,
        AgentFamily::Amp,
        AgentFamily::Grok,
        AgentFamily::Hermes,
        AgentFamily::Kilo,
        AgentFamily::Qodercli,
        AgentFamily::Qwen,
        AgentFamily::Letta,
        AgentFamily::Maki,
        AgentFamily::Muse,
    ]
}

fn new_store() -> AgentObservationStore {
    AgentObservationStore::new(owner(0xa1))
}

fn full_snapshot(
    store: &mut AgentObservationStore,
    topology: &MultiplexerTopology,
) -> AgentObservationSnapshotV2 {
    store
        .snapshot(
            topology,
            &ListAgentObservationsV2 {
                multiplexer_workspace_id: None,
                cursor: None,
            },
        )
        .expect("a cursorless snapshot is always answerable")
}

#[test]
fn t173_observed_detection_binds_to_exact_pane_and_runtime() {
    let pane_id = pane(0x20);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    let events = store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x30),
                observed(AgentFamily::Claude),
            ),
        )
        .expect("a live pane is recordable");
    assert_eq!(events.len(), 1);

    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(snapshot.observations.len(), 1);
    let observation = &snapshot.observations[0];
    assert_eq!(observation.family, AgentFamilyV2::Claude);
    assert_eq!(observation.multiplexer_workspace_id, workspace_id);
    assert_eq!(observation.tab_id, tab_id);
    assert_eq!(observation.pane_id, pane_id);
    assert_eq!(observation.owner_generation_id, owner(0xa1));
    assert_eq!(observation.freshness, AgentObservationFreshnessV2::Current);
    // An unbound pane records no runtime reference rather than inventing one.
    assert_eq!(observation.runtime_namespace_id, None);
    // A Git workspace id is optional and independent of the multiplexer workspace.
    assert_eq!(observation.git_workspace_id, None);
}

#[test]
fn t173_bound_runtime_reference_comes_from_topology_not_the_candidate() {
    let pane_id = pane(0x21);
    let (mut topology, workspace_id, tab_id) = topology_with(pane_id);
    let bound = bind(&mut topology, workspace_id, tab_id, pane_id, runtime(0x22));
    assert!(bound, "the pane must accept a runtime binding");

    let mut store = new_store();
    let mut with_runtime = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x31),
        observed(AgentFamily::Codex),
    );
    // The candidate carries no runtime field at all, and still cannot misattribute.
    assert_eq!(
        store
            .record(&topology, owner(0xa1), with_runtime.clone())
            .expect("recordable")
            .len(),
        1
    );
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(
        snapshot.observations[0].runtime_namespace_id,
        Some(runtime(0x22)),
        "the runtime reference must be the one topology bound to the pane"
    );

    // Recording again under a different family keeps the same bound runtime.
    with_runtime.detection = observed(AgentFamily::Gemini);
    store
        .record(&topology, owner(0xa1), with_runtime)
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(snapshot.observations[0].family, AgentFamilyV2::Gemini);
    assert_eq!(
        snapshot.observations[0].runtime_namespace_id,
        Some(runtime(0x22))
    );
}

fn bind(
    topology: &mut MultiplexerTopology,
    workspace_id: MultiplexerWorkspaceId,
    tab_id: TabId,
    pane_id: PaneId,
    runtime_namespace_id: RuntimeNamespaceId,
) -> bool {
    let generation = topology.generation();
    topology
        .bind_pane_runtime(
            generation,
            workspace_id,
            tab_id,
            pane_id,
            runtime_namespace_id,
        )
        .is_ok()
}

#[test]
fn t173_pane_replacement_invalidates_a_stale_observation() {
    let pane_id = pane(0x23);
    let (mut topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x32),
                observed(AgentFamily::Kilo),
            ),
        )
        .expect("recordable");
    assert_eq!(store.observation_count(), 1);

    // Close the pane. The observation must not survive as current.
    let generation = topology.generation();
    let closed = topology.close_pane(generation, workspace_id, tab_id, pane_id);
    assert!(closed.is_ok(), "the pane must close");
    let removed = store
        .invalidate_against(&topology)
        .expect("invalidation cannot be refused");
    assert_eq!(
        removed.len(),
        1,
        "a closed pane invalidates its observation"
    );
    assert_eq!(store.observation_count(), 0);

    // And the snapshot cannot resurrect it.
    assert_eq!(
        full_snapshot(&mut store, &topology).observations.len(),
        0,
        "an invalidated observation is never projected again"
    );
}

#[test]
fn t173_runtime_replacement_invalidates_a_stale_observation() {
    let pane_id = pane(0x24);
    let (mut topology, workspace_id, tab_id) = topology_with(pane_id);
    assert!(bind(
        &mut topology,
        workspace_id,
        tab_id,
        pane_id,
        runtime(0x25)
    ));
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x33),
                observed(AgentFamily::Cline),
            ),
        )
        .expect("recordable");
    assert_eq!(store.observation_count(), 1);
    assert_eq!(
        full_snapshot(&mut store, &topology).observations[0].runtime_namespace_id,
        Some(runtime(0x25))
    );

    // Prove the binding itself, rather than pane liveness, is part of freshness:
    // the same workspace/tab/pane identities resolving to a different runtime
    // must invalidate the observation recorded against runtime 0x25.
    let (mut replacement_topology, replacement_workspace, replacement_tab) = topology_with(pane_id);
    assert_eq!(replacement_workspace, workspace_id);
    assert_eq!(replacement_tab, tab_id);
    assert!(bind(
        &mut replacement_topology,
        workspace_id,
        tab_id,
        pane_id,
        runtime(0x26)
    ));
    let removed = store
        .invalidate_against(&replacement_topology)
        .expect("runtime replacement invalidation cannot be refused");
    assert_eq!(removed.len(), 1);
    assert_eq!(store.observation_count(), 0);
}

#[test]
fn t173_unbound_runtime_invalidates_a_bound_observation() {
    // An observation bound to a runtime must not survive the pane losing it.
    let pane_id = pane(0x26);
    let (mut topology, workspace_id, tab_id) = topology_with(pane_id);
    assert!(bind(
        &mut topology,
        workspace_id,
        tab_id,
        pane_id,
        runtime(0x27)
    ));
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x34),
                observed(AgentFamily::Qwen),
            ),
        )
        .expect("recordable");

    // The same exact pane resolving to another runtime must not inherit the old
    // observation merely because its workspace/tab/pane IDs match.
    let (mut other_topology, other_workspace, other_tab) = topology_with(pane_id);
    assert!(bind(
        &mut other_topology,
        other_workspace,
        other_tab,
        pane_id,
        runtime(0x28)
    ));
    let removed = store
        .invalidate_against(&other_topology)
        .expect("not refusable");
    assert_eq!(
        removed.len(),
        1,
        "an observation bound to runtime 0x27 must not hold where the pane resolves to runtime 0x28"
    );
}

#[test]
fn t173_duplicate_labels_cannot_change_association() {
    // Two panes in different workspaces carrying the same alias.
    let left_pane = pane(0x30);
    let right_pane = pane(0x31);
    let left_workspace = workspace(0x40);
    let right_workspace = workspace(0x41);
    let left_tab = tab(0x42);
    let right_tab = tab(0x43);

    let tab_of = |tab_id: TabId, pane_id: PaneId| TabState {
        id: tab_id,
        alias: "same-label".to_owned(),
        root: LayoutNode::Pane(pane_id),
        focused_pane_id: pane_id,
        zoomed_pane_id: None,
    };
    let topology = MultiplexerTopology::restore_presentation(
        TopologyGeneration::initial(),
        vec![
            WorkspaceState {
                id: left_workspace,
                alias: "duplicate".to_owned(),
                tabs: vec![tab_of(left_tab, left_pane)],
                focused_tab_id: left_tab,
            },
            WorkspaceState {
                id: right_workspace,
                alias: "duplicate".to_owned(),
                tabs: vec![tab_of(right_tab, right_pane)],
                focused_tab_id: right_tab,
            },
        ],
        Some(left_workspace),
    )
    .expect("two identically labelled workspaces are restorable");

    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                left_workspace,
                left_tab,
                left_pane,
                id(0x50),
                observed(AgentFamily::Pi),
            ),
        )
        .expect("recordable");
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                right_workspace,
                right_tab,
                right_pane,
                id(0x51),
                observed(AgentFamily::Muse),
            ),
        )
        .expect("recordable");
    assert_eq!(store.observation_count(), 2);

    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(snapshot.observations.len(), 2);
    // Association follows the immutable pane identity, never the alias.
    let mut by_pane: Vec<(PaneId, AgentFamilyV2, MultiplexerWorkspaceId)> = snapshot
        .observations
        .iter()
        .map(|observation| {
            (
                observation.pane_id,
                observation.family,
                observation.multiplexer_workspace_id,
            )
        })
        .collect();
    by_pane.sort_by_key(|(pane_id, _, _)| *pane_id);
    assert_eq!(
        by_pane,
        vec![
            (left_pane, AgentFamilyV2::Pi, left_workspace),
            (right_pane, AgentFamilyV2::Muse, right_workspace),
        ]
    );

    // A candidate claiming the left pane under the right workspace is refused,
    // so a label can never move an observation across panes.
    assert_eq!(
        store.record(
            &topology,
            owner(0xa1),
            candidate(
                right_workspace,
                right_tab,
                left_pane,
                id(0x52),
                observed(AgentFamily::Amp),
            ),
        ),
        Err(ObservationError::Topology(
            MultiplexerErrorKind::UnknownPane
        )),
        "a pane identity is bound to its own workspace and tab"
    );
    assert_eq!(store.observation_count(), 2, "a refusal mutates nothing");
}

#[test]
fn t173_one_observation_identity_cannot_denote_two_panes() {
    let first = pane(0x33);
    let second = pane(0x34);
    let (first_topology, first_workspace, first_tab) = topology_with(first);
    let (second_topology, second_workspace, second_tab) = topology_with(second);
    let mut store = new_store();
    store
        .record(
            &first_topology,
            owner(0xa1),
            candidate(
                first_workspace,
                first_tab,
                first,
                id(0x60),
                observed(AgentFamily::Cline),
            ),
        )
        .expect("recordable");
    assert_eq!(
        store.record(
            &second_topology,
            owner(0xa1),
            candidate(
                second_workspace,
                second_tab,
                second,
                id(0x60),
                observed(AgentFamily::Cline),
            ),
        ),
        Err(ObservationError::ObservationIdentityReuse),
        "one observation identity may denote only one pane"
    );
    assert_eq!(store.observation_count(), 1);
}

#[test]
fn t173_event_gap_forces_an_explicit_resnapshot() {
    let pane_id = pane(0x35);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x61),
                observed(AgentFamily::Droid),
            ),
        )
        .expect("recordable");

    let before = store.snapshot_revision();
    let stale_cursor = AgentObservationCursorV2 {
        owner_generation_id: owner(0xa1),
        snapshot_revision: before,
        offset: 0,
    };
    // Advance the revision so the cursor is genuinely behind.
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x62),
                observed(AgentFamily::Grok),
            ),
        )
        .expect("recordable");
    assert!(store.snapshot_revision() > before);

    // Events since the old cursor are an explicit gap, never a partial list.
    let events = store
        .events_since(&topology, Some(&stale_cursor))
        .expect("not refusable");
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        AgentObservationEventV2::HistoryGap {
            last_known_snapshot_revision
        } if last_known_snapshot_revision == before
    ));

    // The recovery is a full resnapshot from offset zero.
    let resnapshot = store
        .snapshot(
            &topology,
            &ListAgentObservationsV2 {
                multiplexer_workspace_id: None,
                cursor: Some(stale_cursor),
            },
        )
        .expect("a stale cursor still resnapshots");
    assert_eq!(resnapshot.page_offset, 0);
    assert_eq!(resnapshot.observations.len(), 1);
    assert_eq!(resnapshot.snapshot_revision, store.snapshot_revision());
}

#[test]
fn t173_snapshot_paging_is_bounded_and_complete() {
    let families = all_families();
    let mut identities = Vec::new();
    let mut workspaces = Vec::new();
    for index in 0_u8..6 {
        let workspace_id = workspace(0x20 + index);
        let tab_id = tab(0x40 + index);
        let pane_id = pane(0x60 + index);
        identities.push((workspace_id, tab_id, pane_id));
        workspaces.push(WorkspaceState {
            id: workspace_id,
            alias: format!("page-{index}"),
            tabs: vec![TabState {
                id: tab_id,
                alias: "main".to_owned(),
                root: LayoutNode::Pane(pane_id),
                focused_pane_id: pane_id,
                zoomed_pane_id: None,
            }],
            focused_tab_id: tab_id,
        });
    }
    let topology = MultiplexerTopology::restore_presentation(
        TopologyGeneration::initial(),
        workspaces,
        Some(identities[0].0),
    )
    .expect("the paging topology is valid");
    let mut store = new_store();

    for (pane_index, (workspace_id, tab_id, pane_id)) in identities.iter().copied().enumerate() {
        let observation_ids = (0..families.len())
            .map(|family_index| {
                id(u8::try_from(pane_index * families.len() + family_index + 1)
                    .expect("the fixture stays below 255 identities"))
            })
            .collect();
        store
            .record(
                &topology,
                owner(0xa1),
                candidate_with_ids(
                    workspace_id,
                    tab_id,
                    pane_id,
                    observation_ids,
                    AgentDetection::Ambiguous {
                        families: families.clone(),
                    },
                ),
            )
            .expect("each paging pane is recordable");
    }
    let expected_total = identities.len() * families.len();
    assert_eq!(expected_total, 144);
    assert_eq!(store.observation_count(), expected_total);

    let first = full_snapshot(&mut store, &topology);
    assert_eq!(first.observations.len(), MAX_V2_AGENT_OBSERVATIONS_PER_PAGE);
    let stale_cursor = first
        .next_cursor
        .clone()
        .expect("144 observations require a continuation page");
    assert_eq!(
        usize::from(stale_cursor.offset),
        MAX_V2_AGENT_OBSERVATIONS_PER_PAGE
    );

    // Move the revision after a client has received page one. The old cursor must
    // restart from offset zero, but the restarted snapshot must remain pageable.
    let (workspace_id, tab_id, pane_id) = identities[0];
    store
        .record(
            &topology,
            owner(0xa1),
            candidate_with_ids(
                workspace_id,
                tab_id,
                pane_id,
                (0..families.len())
                    .map(|index| id(0xa0 + u8::try_from(index).expect("24 families fit in u8")))
                    .collect(),
                AgentDetection::Ambiguous {
                    families: families.clone(),
                },
            ),
        )
        .expect("revision-moving replacement is recordable");
    assert_eq!(store.observation_count(), expected_total);

    let restarted = store
        .snapshot(
            &topology,
            &ListAgentObservationsV2 {
                multiplexer_workspace_id: None,
                cursor: Some(stale_cursor),
            },
        )
        .expect("a stale cursor restarts authoritatively");
    assert_eq!(restarted.page_offset, 0);
    assert_eq!(
        restarted.observations.len(),
        MAX_V2_AGENT_OBSERVATIONS_PER_PAGE
    );
    assert_eq!(restarted.snapshot_revision, store.snapshot_revision());
    let mut cursor = restarted
        .next_cursor
        .clone()
        .expect("the restarted first page must preserve continuation");
    assert_eq!(cursor.snapshot_revision, store.snapshot_revision());

    let mut seen = restarted
        .observations
        .iter()
        .map(|observation| observation.observation_id)
        .collect::<Vec<_>>();
    let mut page_count = 1;
    loop {
        let page = store
            .snapshot(
                &topology,
                &ListAgentObservationsV2 {
                    multiplexer_workspace_id: None,
                    cursor: Some(cursor),
                },
            )
            .expect("continuation cursor is answerable");
        page_count += 1;
        assert!(page.observations.len() <= MAX_V2_AGENT_OBSERVATIONS_PER_PAGE);
        assert_eq!(page.snapshot_revision, store.snapshot_revision());
        seen.extend(
            page.observations
                .iter()
                .map(|observation| observation.observation_id),
        );
        let Some(next) = page.next_cursor else {
            break;
        };
        cursor = next;
    }
    assert_eq!(page_count, 2);
    assert_eq!(seen.len(), expected_total);
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        expected_total,
        "every observation appears exactly once"
    );
}

#[test]
fn t173_a_cursor_from_another_owner_generation_is_refused() {
    let pane_id = pane(0x37);
    let (topology, _workspace_id, _tab_id) = topology_with(pane_id);
    let mut store = new_store();
    let foreign = AgentObservationCursorV2 {
        owner_generation_id: owner(0xb2),
        snapshot_revision: store.snapshot_revision(),
        offset: 0,
    };
    assert_eq!(
        store.snapshot(
            &topology,
            &ListAgentObservationsV2 {
                multiplexer_workspace_id: None,
                cursor: Some(foreign.clone()),
            },
        ),
        Err(ObservationError::StaleOwnerGeneration)
    );
    assert_eq!(
        store.events_since(&topology, Some(&foreign)),
        Err(ObservationError::StaleOwnerGeneration)
    );
}

#[test]
fn t173_a_candidate_from_another_owner_generation_is_refused() {
    let pane_id = pane(0x38);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    let before = store.snapshot_revision();
    assert_eq!(
        store.record(
            &topology,
            owner(0xb2),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x64),
                observed(AgentFamily::Kiro),
            ),
        ),
        Err(ObservationError::StaleOwnerGeneration)
    );
    assert_eq!(store.observation_count(), 0);
    assert_eq!(
        store.snapshot_revision(),
        before,
        "a refusal advances nothing"
    );
}

#[test]
fn t173_ambiguity_never_collapses_to_one_family() {
    let pane_id = pane(0x39);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate_with_ids(
                workspace_id,
                tab_id,
                pane_id,
                vec![id(0x65), id(0x75)],
                AgentDetection::Ambiguous {
                    families: vec![AgentFamily::Pi, AgentFamily::Claude],
                },
            ),
        )
        .expect("recordable");

    let snapshot = full_snapshot(&mut store, &topology);
    // Both candidates are retained as candidates. Dropping one would silently
    // turn an ambiguity into a claim about the family that happened to be kept.
    assert_eq!(
        snapshot.observations.len(),
        2,
        "both candidates must survive"
    );
    assert!(snapshot.observations.iter().all(|observation| {
        observation.freshness == AgentObservationFreshnessV2::Ambiguous
            && observation.confidence_class == AgentObservationConfidenceV2::Unknown
    }));
    let families: Vec<AgentFamilyV2> = snapshot
        .observations
        .iter()
        .map(|item| item.family)
        .collect();
    assert_eq!(families, vec![AgentFamilyV2::Pi, AgentFamilyV2::Claude]);
    // Distinct observation identities, because the frozen page validator
    // rejects a page that repeats one.
    let identities: Vec<AgentObservationId> = snapshot
        .observations
        .iter()
        .map(|item| item.observation_id)
        .collect();
    assert_ne!(identities[0], identities[1]);

    // And the multi-family page is itself a valid frozen message.
    let sequence = EventSequence::new(9).expect("a nonzero sequence is accepted");
    ProtocolMessage::new(
        ClientConnectionId::new("t173-ambiguous").expect("a bounded label is accepted"),
        sequence,
        None,
        owner(0xa1),
        Some(sequence),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: snapshot.snapshot_revision,
                page_offset: snapshot.page_offset,
                observations: snapshot.observations.clone(),
                next_cursor: None,
            },
        },
    )
    .expect("an ambiguous page must satisfy the frozen contract");
}

#[test]
fn t173_a_multi_family_outcome_needs_one_identity_per_family() {
    let pane_id = pane(0x5a);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    let ambiguous = AgentDetection::Ambiguous {
        families: vec![AgentFamily::Pi, AgentFamily::Claude],
    };
    // Too few identities would make the page violate the frozen contract.
    assert_eq!(
        store.record(
            &topology,
            owner(0xa1),
            candidate_with_ids(
                workspace_id,
                tab_id,
                pane_id,
                vec![id(0x65)],
                ambiguous.clone()
            ),
        ),
        Err(ObservationError::Unrepresentable)
    );
    assert_eq!(
        store.record(
            &topology,
            owner(0xa1),
            candidate_with_ids(workspace_id, tab_id, pane_id, vec![], ambiguous),
        ),
        Err(ObservationError::Unrepresentable)
    );
    assert_eq!(store.observation_count(), 0, "a refusal mutates nothing");
}

#[test]
fn t173_unavailable_and_unknown_are_explicit_states() {
    let pane_id = pane(0x3a);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x66),
                AgentDetection::Unavailable {
                    reason:
                        crate::multiplexer::agent_catalog::UnavailableReason::UnsupportedPlatform,
                    families: vec![AgentFamily::Amp],
                },
            ),
        )
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(
        snapshot.observations[0].freshness,
        AgentObservationFreshnessV2::Unavailable
    );

    // An outcome that names no family withdraws the observation entirely rather
    // than inventing one.
    for unnameable in [
        AgentDetection::NearMatch,
        AgentDetection::Unknown {
            reason: crate::multiplexer::agent_catalog::UnknownReason::NoMatch,
        },
        AgentDetection::UntrustedText {
            source: crate::multiplexer::agent_catalog::UntrustedTextSource::TerminalProse,
        },
    ] {
        let mut scenario = new_store();
        scenario
            .record(
                &topology,
                owner(0xa1),
                candidate(
                    workspace_id,
                    tab_id,
                    pane_id,
                    id(0x67),
                    observed(AgentFamily::Claude),
                ),
            )
            .expect("recordable");
        assert_eq!(scenario.observation_count(), 1);
        let removed = scenario
            .record(
                &topology,
                owner(0xa1),
                candidate(workspace_id, tab_id, pane_id, id(0x67), unnameable.clone()),
            )
            .expect("recordable");
        assert_eq!(removed.len(), 1, "{unnameable:?} must withdraw");
        assert_eq!(scenario.observation_count(), 0);
        assert_eq!(
            full_snapshot(&mut scenario, &topology).observations.len(),
            0,
            "{unnameable:?} must never appear as an observation"
        );
    }
}

#[test]
fn t173_source_classes_are_ordered_and_a_user_claim_never_raises_confidence() {
    let pane_id = pane(0x3b);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();

    // A user-declared presentation is capped at UserDeclared confidence.
    let mut declared = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x70),
        observed(AgentFamily::Letta),
    );
    declared.source_class = AgentObservationSourceV2::UserDeclaredPresentation;
    store
        .record(&topology, owner(0xa1), declared)
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(
        snapshot.observations[0].confidence_class,
        AgentObservationConfidenceV2::UserDeclared,
        "a user claim never raises confidence above what the user asserted"
    );

    // A stronger source does replace a weaker accepted classification.
    let stronger = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x72),
        observed(AgentFamily::Letta),
    );
    store
        .record(&topology, owner(0xa1), stronger)
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(
        snapshot.observations[0].source_class,
        AgentObservationSourceV2::OwnedProcessMetadata
    );
    assert_eq!(
        snapshot.observations[0].confidence_class,
        AgentObservationConfidenceV2::Strong
    );

    // A weaker source then cannot replace it again for the same family.
    let mut weaker = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x71),
        observed(AgentFamily::Letta),
    );
    weaker.source_class = AgentObservationSourceV2::UserDeclaredPresentation;
    let events = store
        .record(&topology, owner(0xa1), weaker)
        .expect("recordable");
    assert!(
        events.is_empty(),
        "a weaker source must not replace a stronger classification"
    );
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(snapshot.observations[0].family, AgentFamilyV2::Letta);
    assert_eq!(
        snapshot.observations[0].source_class,
        AgentObservationSourceV2::OwnedProcessMetadata,
        "the stronger source class must be the one retained"
    );
}

#[test]
fn t173_a_stronger_source_replaces_a_different_family_weakly_observed() {
    let pane_id = pane(0x3f);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();

    // A weak source qualifies one family; a stronger source qualifies another.
    // The stronger classification is the one that holds, because source order
    // decides which accepted description Winds stands behind.
    let mut weak = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x73),
        observed(AgentFamily::Maki),
    );
    weak.source_class = AgentObservationSourceV2::ProviderStructuredMetadata;
    store
        .record(&topology, owner(0xa1), weak)
        .expect("recordable");
    assert_eq!(
        full_snapshot(&mut store, &topology).observations[0].family,
        AgentFamilyV2::Maki
    );

    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x74),
                observed(AgentFamily::Amp),
            ),
        )
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(snapshot.observations[0].family, AgentFamilyV2::Amp);
    assert_eq!(snapshot.observations.len(), 1, "one pane, one observation");
}

#[test]
fn t173_provider_session_identity_is_separate_and_optional() {
    let pane_id = pane(0x3c);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();

    let mut with_session = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x80),
        observed(AgentFamily::Omp),
    );
    with_session.provider_native_session_id = Some("sess-abc".to_owned());
    with_session.git_workspace_id = Some("git-ws-1".to_owned());
    store
        .record(&topology, owner(0xa1), with_session)
        .expect("recordable");

    let snapshot = full_snapshot(&mut store, &topology);
    let observation = &snapshot.observations[0];
    assert_eq!(
        observation.provider_native_session_id.as_deref(),
        Some("sess-abc")
    );
    assert_eq!(observation.git_workspace_id.as_deref(), Some("git-ws-1"));
    // The provider session is its own identity, carried alongside the pane
    // binding rather than in place of it.
    assert_ne!(
        observation.provider_native_session_id.as_deref(),
        Some(observation.pane_id.as_hex().as_str()),
        "a provider-native session is never the pane identity"
    );
    assert_ne!(
        observation.git_workspace_id.as_deref(),
        Some(observation.multiplexer_workspace_id.as_hex().as_str()),
        "a Git workspace is never the multiplexer workspace identity"
    );

    // Changing the provider session cannot move the association: the observation
    // stays bound to the same pane and family.
    let mut moved = candidate(
        workspace_id,
        tab_id,
        pane_id,
        id(0x82),
        observed(AgentFamily::Omp),
    );
    moved.provider_native_session_id = Some("sess-xyz".to_owned());
    store
        .record(&topology, owner(0xa1), moved)
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    assert_eq!(snapshot.observations[0].pane_id, pane_id);
    assert_eq!(snapshot.observations[0].family, AgentFamilyV2::Omp);
    assert_eq!(
        snapshot.observations[0]
            .provider_native_session_id
            .as_deref(),
        Some("sess-xyz")
    );

    // An empty or oversized optional field is refused rather than truncated.
    for bad in [
        Some(String::new()),
        Some("x".repeat(MAX_V2_PROVIDER_SESSION_ID_BYTES + 1)),
    ] {
        let mut invalid = candidate(
            workspace_id,
            tab_id,
            pane_id,
            id(0x81),
            observed(AgentFamily::Omp),
        );
        invalid.provider_native_session_id = bad;
        assert_eq!(
            store.record(&topology, owner(0xa1), invalid),
            Err(ObservationError::Unrepresentable)
        );
    }
    assert_eq!(store.observation_count(), 1, "a refusal mutates nothing");
}

#[test]
fn t173_detection_never_claims_execution_or_verification() {
    let pane_id = pane(0x3d);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0x90),
                observed(AgentFamily::Muse),
            ),
        )
        .expect("recordable");
    let snapshot = full_snapshot(&mut store, &topology);
    let observation = &snapshot.observations[0];

    // Detection support is not execution authority, in the catalog and here.
    assert!(!observation_is_execution(observed(AgentFamily::Muse)));
    // The strongest confidence a structured detection reaches is Strong, never a
    // verification or acceptance state, and there is no such field to set.
    assert_eq!(
        observation.confidence_class,
        AgentObservationConfidenceV2::Strong
    );
    let summary = observation.structured_evidence_summary.as_str();
    assert!(summary.contains("OBSERVED"));
    assert!(!summary.contains("VERIFIED"));
    assert!(!summary.contains("ACCEPTED"));
    assert!(!summary.contains("NEEDS_YOU"));
}

fn observation_is_execution(detection: AgentDetection) -> bool {
    detection.confers_execution_authority()
}

#[test]
fn t173_full_page_fits_one_already_frozen_frame() {
    // The T166 contract permits at most one typed page per snapshot and fixes the
    // item budget. The byte budget is whatever the frozen frame limit already is;
    // T173 introduces no framing of its own, so the proof is that the largest
    // permitted page encodes inside one frame.
    let pane_id = pane(0x3e);
    let (topology, workspace_id, tab_id) = topology_with(pane_id);
    let mut store = new_store();
    store
        .record(
            &topology,
            owner(0xa1),
            candidate(
                workspace_id,
                tab_id,
                pane_id,
                id(0xa0),
                observed(AgentFamily::Claude),
            ),
        )
        .expect("recordable");

    let observation = full_snapshot(&mut store, &topology).observations[0].clone();
    let widest_evidence = "e".repeat(MAX_V2_EVIDENCE_SUMMARY_BYTES);
    let mut page = Vec::with_capacity(MAX_V2_AGENT_OBSERVATIONS_PER_PAGE);
    for index in 0..MAX_V2_AGENT_OBSERVATIONS_PER_PAGE {
        let mut item = observation.clone();
        // Every item needs its own observation identity: the frozen validator
        // rejects a page that repeats one, so a budget proof that reused an
        // identity would fail the contract rather than measure it.
        item.observation_id = AgentObservationId::from_entropy_bytes([index as u8 + 1; 16])
            .expect("non-zero entropy is accepted");
        item.structured_evidence_summary = widest_evidence.clone();
        item.git_workspace_id = Some("g".repeat(256));
        item.provider_native_session_id = Some("p".repeat(256));
        item.observed_unix_ms = index as i64;
        page.push(item);
    }
    assert_eq!(page.len(), MAX_V2_AGENT_OBSERVATIONS_PER_PAGE);

    let sequence = EventSequence::new(1).expect("a nonzero sequence is accepted");
    let message = ProtocolMessage::new(
        ClientConnectionId::new("t173-client").expect("a bounded label is accepted"),
        sequence,
        None,
        owner(0xa1),
        // The frozen contract marks an agent-observation snapshot as requiring a
        // correlation sequence, and forbids binding it to a runtime namespace.
        Some(sequence),
        ProtocolPayload::AgentObservationSnapshot {
            snapshot: AgentObservationSnapshotV2 {
                filter_multiplexer_workspace_id: None,
                snapshot_revision: 2,
                page_offset: 0,
                observations: page,
                next_cursor: None,
            },
        },
    )
    .expect("a full permitted page must be a valid frozen message");
    assert_eq!(message.kind(), MessageKind::AgentObservationSnapshot);
    let encoded = encode_frame(&message).expect("the page must encode");
    assert!(
        encoded.len() <= MAX_CONTROL_FRAME_BYTES,
        "a full permitted page must fit one frame: {} > {}",
        encoded.len(),
        MAX_CONTROL_FRAME_BYTES
    );
    // And T173 must not raise the item budget beyond the frozen limit.
    assert_eq!(MAX_V2_AGENT_OBSERVATIONS_PER_PAGE, 128);
}
