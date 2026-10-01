use super::*;
use crate::multiplexer::domain::{
    AgentObservationId, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeNamespaceId};
use crate::persistent_runtime::protocol::{
    AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationFreshnessV2,
    AgentObservationSourceV2, AgentObservationV2, ApplyTopologyOperationV2, TopologyOperationV2,
};

fn observation_id(byte: u8) -> AgentObservationId {
    AgentObservationId::from_entropy_bytes([byte; 16]).expect("valid observation id")
}

fn workspace_id(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).expect("valid workspace id")
}

fn tab_id(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).expect("valid tab id")
}

fn pane_id(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).expect("valid pane id")
}

fn owner_generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).expect("valid owner generation")
}

fn runtime_id(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).expect("valid runtime id")
}

fn observation(
    observation_byte: u8,
    workspace_byte: u8,
    tab_byte: u8,
    pane_byte: u8,
    runtime_byte: u8,
    observed_unix_ms: i64,
) -> AgentObservationV2 {
    AgentObservationV2 {
        observation_id: observation_id(observation_byte),
        family: AgentFamilyV2::Codex,
        source_class: AgentObservationSourceV2::ProviderStructuredMetadata,
        confidence_class: AgentObservationConfidenceV2::Strong,
        freshness: AgentObservationFreshnessV2::Current,
        multiplexer_workspace_id: workspace_id(workspace_byte),
        git_workspace_id: Some(format!("git-workspace-{workspace_byte}")),
        tab_id: tab_id(tab_byte),
        pane_id: pane_id(pane_byte),
        runtime_namespace_id: Some(runtime_id(runtime_byte)),
        provider_native_session_id: Some(format!("provider-session-{observation_byte}")),
        owner_generation_id: owner_generation(0x44),
        observed_unix_ms,
        structured_evidence_summary: format!("structured detector evidence {observation_byte}"),
    }
}

fn topology_for(observation: &AgentObservationV2, generation: u64) -> CanonicalTopologyPresentation {
    let topology_generation = TopologyGeneration::new(generation).expect("valid topology generation");
    CanonicalTopologyPresentation {
        topology_generation,
        text: "canonical topology".to_owned(),
        search_bindings: vec![CanonicalTopologySearchBinding {
            canonical_id: observation.pane_id.to_string(),
            display_label: "duplicate visible label".to_owned(),
            stable_key: format!("pane:{}", observation.pane_id),
            intent: ApplyTopologyOperationV2 {
                expected_topology_generation: topology_generation,
                operation: TopologyOperationV2::FocusPane {
                    multiplexer_workspace_id: observation.multiplexer_workspace_id,
                    tab_id: observation.tab_id,
                    pane_id: observation.pane_id,
                },
            },
        }],
    }
}

#[test]
fn t174_duplicate_labels_and_concurrent_observations_preserve_exact_targets() {
    let first = observation(0x11, 0x21, 0x31, 0x41, 0x51, 100);
    let second = observation(0x12, 0x21, 0x31, 0x42, 0x52, 101);
    let dock = CanonicalAgentDockPresentation::from_observations(vec![first.clone(), second.clone()])
        .expect("distinct observation identities should project");

    let rows = dock.list("codex", CanonicalAgentDockSort::Family, false);
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].observation_id, rows[1].observation_id);
    assert_ne!(rows[0].pane_id, rows[1].pane_id);
    assert_ne!(rows[0].runtime_namespace_id, rows[1].runtime_namespace_id);

    assert_eq!(dock.read(first.observation_id), Some(&first));
    assert_eq!(dock.read(second.observation_id), Some(&second));
}

#[test]
fn t174_rename_is_display_alias_only_and_never_changes_identity_or_truth() {
    let original = observation(0x13, 0x23, 0x33, 0x43, 0x53, 200);
    let mut dock = CanonicalAgentDockPresentation::from_observations(vec![original.clone()])
        .expect("observation should project");

    dock.rename(original.observation_id, Some("Pair Reviewer"))
        .expect("bounded display alias should be accepted");
    let item = dock.get(original.observation_id).expect("item should remain present");
    assert_eq!(item.display_alias.as_deref(), Some("Pair Reviewer"));
    assert_eq!(dock.read(original.observation_id), Some(&original));

    dock.rename(original.observation_id, None)
        .expect("clearing presentation alias should be accepted");
    assert_eq!(
        dock.get(original.observation_id)
            .expect("item should remain present")
            .display_alias,
        None
    );
    assert_eq!(dock.read(original.observation_id), Some(&original));
}

#[test]
fn t174_focus_returns_read_only_exact_pane_view_target_and_refuses_stale_or_missing_truth() {
    let current = observation(0x14, 0x24, 0x34, 0x44, 0x54, 300);
    let mut dock = CanonicalAgentDockPresentation::from_observations(vec![current.clone()])
        .expect("observation should project");
    let topology = topology_for(&current, 7);

    let target = dock
        .focus(current.observation_id, &topology)
        .expect("exact current pane should bind as a view target");
    assert_eq!(target.observation_id, current.observation_id);
    assert_eq!(target.multiplexer_workspace_id, current.multiplexer_workspace_id);
    assert_eq!(target.tab_id, current.tab_id);
    assert_eq!(target.pane_id, current.pane_id);
    assert_eq!(target.runtime_namespace_id, current.runtime_namespace_id);
    assert_eq!(target.owner_generation_id, current.owner_generation_id);
    assert_eq!(target.topology_generation, topology.topology_generation());
    assert_eq!(dock.focused_observation_id(), Some(current.observation_id));

    let mut stale = current.clone();
    stale.freshness = AgentObservationFreshnessV2::Stale;
    let mut stale_dock = CanonicalAgentDockPresentation::from_observations(vec![stale.clone()])
        .expect("stale observation remains explainable");
    assert!(stale_dock.focus(stale.observation_id, &topology).is_none());

    let unrelated = observation(0x15, 0x25, 0x35, 0x45, 0x55, 301);
    let unrelated_topology = topology_for(&unrelated, 8);
    assert!(dock.focus(current.observation_id, &unrelated_topology).is_none());
}

#[test]
fn t174_filter_sort_group_are_presentation_only_and_explain_detection_limits() {
    let first = observation(0x16, 0x26, 0x36, 0x46, 0x56, 400);
    let mut second = observation(0x17, 0x27, 0x37, 0x47, 0x57, 500);
    second.family = AgentFamilyV2::Claude;
    second.source_class = AgentObservationSourceV2::UserDeclaredPresentation;
    second.confidence_class = AgentObservationConfidenceV2::UserDeclared;
    let original = vec![first.clone(), second.clone()];
    let dock = CanonicalAgentDockPresentation::from_observations(original.clone())
        .expect("observations should project");

    let recent = dock.list("", CanonicalAgentDockSort::Recent, true);
    assert_eq!(recent[0].observation_id, second.observation_id);
    let filtered = dock.list("claude", CanonicalAgentDockSort::Source, true);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].observation_id, second.observation_id);

    assert_eq!(dock.read(first.observation_id), Some(&first));
    assert_eq!(dock.read(second.observation_id), Some(&second));
    let explanation = dock
        .explain(second.observation_id)
        .expect("current observation should be explainable");
    assert!(explanation.contains("source=USER_DECLARED_PRESENTATION"));
    assert!(explanation.contains("freshness=CURRENT"));
    assert!(explanation.contains("execution=DETECTION_ONLY_UNPROVEN"));
}

#[test]
fn t174_duplicate_observation_identity_is_rejected_instead_of_overwritten() {
    let first = observation(0x18, 0x28, 0x38, 0x48, 0x58, 600);
    let mut duplicate = observation(0x19, 0x29, 0x39, 0x49, 0x59, 601);
    duplicate.observation_id = first.observation_id;

    assert!(CanonicalAgentDockPresentation::from_observations(vec![first, duplicate]).is_err());
}
