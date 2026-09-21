use super::{
    MAX_CONTROL_FRAME_BYTES, MessageKind, ProtocolMessage, ProtocolPayload, ProtocolResult,
    encode_frame,
};
use crate::multiplexer::domain::navigation::{
    LayoutNode, MultiplexerTopology, SplitAxis, WorkspaceState,
};
use crate::multiplexer::domain::{
    AgentObservationId, LayoutTemplateId, MultiplexerAuthority, MultiplexerErrorKind,
    MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, LocalControlErrorKind, OwnerGenerationId, RuntimeNamespaceId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub(crate) const MAX_V2_WORKSPACES: usize = 32;
pub(crate) const MAX_V2_TABS_PER_WORKSPACE: usize = 32;
pub(crate) const MAX_V2_PANES_PER_TAB: usize = 64;
pub(crate) const MAX_V2_AGGREGATE_PANES: usize = 256;
pub(crate) const MAX_V2_AGENT_OBSERVATIONS_PER_PAGE: usize = 128;
pub(crate) const MAX_V2_WORKTREES_PER_PAGE: usize = 48;
pub(crate) const MAX_V2_ATTENTION_ITEMS_PER_PAGE: usize = 128;
pub(crate) const MAX_V2_ALIAS_BYTES: usize = 128;
pub(crate) const MAX_V2_GIT_WORKSPACE_ID_BYTES: usize = 256;
pub(crate) const MAX_V2_REPOSITORY_IDENTITY_BYTES: usize = 512;
pub(crate) const MAX_V2_PATH_BYTES: usize = 4096;
pub(crate) const MAX_V2_BRANCH_BYTES: usize = 256;
pub(crate) const MAX_V2_PROVIDER_SESSION_ID_BYTES: usize = 256;
pub(crate) const MAX_V2_DETAIL_BYTES: usize = 512;
pub(crate) const MAX_V2_EVIDENCE_SUMMARY_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ProtocolSplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ProtocolPanePlacement {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ProtocolPaneClosePolicy {
    DetachView,
    StopRuntimeThenClose,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum ProtocolLayoutNodeV2 {
    Pane {
        pane_id: PaneId,
    },
    Split {
        axis: ProtocolSplitAxis,
        ratio_basis_points: u16,
        first: Box<ProtocolLayoutNodeV2>,
        second: Box<ProtocolLayoutNodeV2>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProtocolTabSnapshotV2 {
    pub(crate) tab_id: TabId,
    pub(crate) alias: String,
    pub(crate) root: ProtocolLayoutNodeV2,
    pub(crate) focused_pane_id: PaneId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) zoomed_pane_id: Option<PaneId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProtocolWorkspaceSnapshotV2 {
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    pub(crate) alias: String,
    pub(crate) topology_generation: TopologyGeneration,
    pub(crate) focused_tab_id: TabId,
    pub(crate) tabs: Vec<ProtocolTabSnapshotV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProtocolWorkspaceSummaryV2 {
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    pub(crate) alias: String,
    pub(crate) topology_generation: TopologyGeneration,
    pub(crate) is_focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum TopologyMutationOutcomeV2 {
    Accepted,
    Rejected { error: MultiplexerErrorKind },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TopologyMutationResultV2 {
    pub(crate) outcome: TopologyMutationOutcomeV2,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) accepted_topology_generation: Option<TopologyGeneration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) multiplexer_workspace_id: Option<MultiplexerWorkspaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tab_id: Option<TabId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pane_id: Option<PaneId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum MultiplexerSnapshotV2 {
    WorkspaceList {
        topology_generation: TopologyGeneration,
        workspaces: Vec<ProtocolWorkspaceSummaryV2>,
    },
    Workspace {
        snapshot: ProtocolWorkspaceSnapshotV2,
    },
    MutationResult {
        result: TopologyMutationResultV2,
        #[serde(skip_serializing_if = "Option::is_none")]
        snapshot: Option<ProtocolWorkspaceSnapshotV2>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListMultiplexerWorkspacesV2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) multiplexer_workspace_id: Option<MultiplexerWorkspaceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MultiplexerWriteStateV2 {
    pub(crate) authority: MultiplexerAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum TopologyOperationV2 {
    CreateWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        alias: String,
        first_tab_id: TabId,
        first_tab_alias: String,
        first_pane_id: PaneId,
    },
    FocusWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
    },
    RenameWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        alias: String,
    },
    MoveWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        new_index: u16,
    },
    CloseWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
    },
    CreateTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        alias: String,
        first_pane_id: PaneId,
    },
    FocusTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    },
    RenameTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        alias: String,
    },
    MoveTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        new_index: u16,
    },
    CloseTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    },
    SplitPane {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        target_pane_id: PaneId,
        new_pane_id: PaneId,
        axis: ProtocolSplitAxis,
        placement: ProtocolPanePlacement,
        ratio_basis_points: u16,
    },
    SwapPanes {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        first_pane_id: PaneId,
        second_pane_id: PaneId,
    },
    MovePane {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        source_tab_id: TabId,
        pane_id: PaneId,
        destination_tab_id: TabId,
        destination_pane_id: PaneId,
        axis: ProtocolSplitAxis,
        placement: ProtocolPanePlacement,
        ratio_basis_points: u16,
    },
    ResizeSplit {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        first_pane_id: PaneId,
        second_pane_id: PaneId,
        ratio_basis_points: u16,
    },
    ToggleZoom {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
    },
    FocusPane {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
    },
    ClosePane {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
        policy: ProtocolPaneClosePolicy,
    },
    ClearPane {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
        presentation_epoch: u64,
    },
    ApplyLayoutTemplate {
        template_id: LayoutTemplateId,
        multiplexer_workspace_id: MultiplexerWorkspaceId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApplyTopologyOperationV2 {
    pub(crate) expected_topology_generation: TopologyGeneration,
    pub(crate) operation: TopologyOperationV2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum MultiplexerEventV2 {
    TopologyChanged {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        topology_generation: TopologyGeneration,
    },
    PaneCleared {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
        topology_generation: TopologyGeneration,
        presentation_epoch: u64,
    },
    HistoryGap {
        last_known_topology_generation: TopologyGeneration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AgentFamilyV2 {
    Pi,
    Claude,
    Codex,
    Gemini,
    Cursor,
    Devin,
    Antigravity,
    Cline,
    Omp,
    Mastracode,
    OpenCode,
    GithubCopilot,
    Kimi,
    Kiro,
    Droid,
    Amp,
    Grok,
    Hermes,
    Kilo,
    Qodercli,
    Qwen,
    Letta,
    Maki,
    Muse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AgentObservationSourceV2 {
    WindsLaunchMetadata,
    OwnedProcessMetadata,
    ProviderStructuredMetadata,
    UserDeclaredPresentation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AgentObservationConfidenceV2 {
    Exact,
    Strong,
    UserDeclared,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AgentObservationFreshnessV2 {
    Current,
    Ambiguous,
    Stale,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentObservationV2 {
    pub(crate) observation_id: AgentObservationId,
    pub(crate) family: AgentFamilyV2,
    pub(crate) source_class: AgentObservationSourceV2,
    pub(crate) confidence_class: AgentObservationConfidenceV2,
    pub(crate) freshness: AgentObservationFreshnessV2,
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) git_workspace_id: Option<String>,
    pub(crate) tab_id: TabId,
    pub(crate) pane_id: PaneId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime_namespace_id: Option<RuntimeNamespaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) provider_native_session_id: Option<String>,
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) observed_unix_ms: i64,
    pub(crate) structured_evidence_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentObservationCursorV2 {
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) snapshot_revision: u64,
    pub(crate) offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListAgentObservationsV2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) multiplexer_workspace_id: Option<MultiplexerWorkspaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cursor: Option<AgentObservationCursorV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentObservationSnapshotV2 {
    pub(crate) snapshot_revision: u64,
    pub(crate) page_offset: u16,
    pub(crate) observations: Vec<AgentObservationV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) next_cursor: Option<AgentObservationCursorV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum AgentObservationEventV2 {
    Upsert {
        snapshot_revision: u64,
        observation: AgentObservationV2,
    },
    Removed {
        snapshot_revision: u64,
        observation_id: AgentObservationId,
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        pane_id: PaneId,
    },
    HistoryGap {
        last_known_snapshot_revision: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum WorktreeMembershipV2 {
    DiscoveredWorktree,
    ExplicitMember,
    CreatedByWindsMember,
    StaleExplicitMember,
    RemovedExplicitMember,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorktreeObservationV2 {
    pub(crate) git_workspace_id: String,
    pub(crate) repository_identity: String,
    pub(crate) canonical_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) branch: Option<String>,
    pub(crate) head_oid: String,
    pub(crate) dirty: bool,
    pub(crate) membership: WorktreeMembershipV2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorktreeCursorV2 {
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) snapshot_revision: u64,
    pub(crate) offset: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListWorktreesV2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) multiplexer_workspace_id: Option<MultiplexerWorkspaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) cursor: Option<WorktreeCursorV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub(crate) enum WorktreeOperationV2 {
    Create {
        repository_identity: String,
        destination_path: String,
        base_commit_oid: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_branch_name: Option<String>,
    },
    Open {
        canonical_path: String,
    },
    Remove {
        git_workspace_id: String,
        canonical_path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApplyWorktreeOperationV2 {
    pub(crate) operation: WorktreeOperationV2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum WorktreeOperationOutcomeV2 {
    Accepted,
    UnsupportedOperation,
    TrustRequired,
    UnsafeState,
    OutcomeUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorktreeOperationResultV2 {
    pub(crate) outcome: WorktreeOperationOutcomeV2,
    pub(crate) snapshot_revision: u64,
    pub(crate) page_offset: u16,
    pub(crate) worktrees: Vec<WorktreeObservationV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) next_cursor: Option<WorktreeCursorV2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AttentionKindV2 {
    ControllerRequired,
    ControllerConflict,
    OutcomeUnknown,
    RuntimeOwnershipLost,
    ExplicitAgentAttention,
    WorktreeTrustRequired,
    WorktreeDestructiveConfirmation,
    WorktreeStaleMembership,
    ProtocolUpgradeRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttentionItemV2 {
    pub(crate) source_domain: String,
    pub(crate) source_event_id: String,
    pub(crate) kind: AttentionKindV2,
    pub(crate) multiplexer_workspace_id: MultiplexerWorkspaceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tab_id: Option<TabId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) pane_id: Option<PaneId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime_namespace_id: Option<RuntimeNamespaceId>,
    pub(crate) owner_generation_id: OwnerGenerationId,
    pub(crate) detail: String,
    pub(crate) stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttentionSnapshotV2 {
    pub(crate) snapshot_revision: u64,
    pub(crate) items: Vec<AttentionItemV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "event_kind",
    rename_all = "SCREAMING_SNAKE_CASE",
    deny_unknown_fields
)]
pub(crate) enum AttentionEventV2 {
    Upsert {
        snapshot_revision: u64,
        item: AttentionItemV2,
    },
    Removed {
        snapshot_revision: u64,
        source_domain: String,
        source_event_id: String,
        kind: AttentionKindV2,
        multiplexer_workspace_id: MultiplexerWorkspaceId,
    },
    HistoryGap {
        last_known_snapshot_revision: u64,
    },
}

fn protocol_layout_node(node: &LayoutNode) -> ProtocolLayoutNodeV2 {
    match node {
        LayoutNode::Pane(pane_id) => ProtocolLayoutNodeV2::Pane { pane_id: *pane_id },
        LayoutNode::Split {
            axis,
            ratio_bps,
            first,
            second,
        } => ProtocolLayoutNodeV2::Split {
            axis: match axis {
                SplitAxis::Horizontal => ProtocolSplitAxis::Horizontal,
                SplitAxis::Vertical => ProtocolSplitAxis::Vertical,
            },
            ratio_basis_points: ratio_bps.get(),
            first: Box::new(protocol_layout_node(first)),
            second: Box::new(protocol_layout_node(second)),
        },
    }
}

fn protocol_workspace_snapshot(
    workspace: &WorkspaceState,
    topology_generation: TopologyGeneration,
) -> ProtocolWorkspaceSnapshotV2 {
    ProtocolWorkspaceSnapshotV2 {
        multiplexer_workspace_id: workspace.id,
        alias: workspace.alias.clone(),
        topology_generation,
        focused_tab_id: workspace.focused_tab_id,
        tabs: workspace
            .tabs
            .iter()
            .map(|tab| ProtocolTabSnapshotV2 {
                tab_id: tab.id,
                alias: tab.alias.clone(),
                root: protocol_layout_node(&tab.root),
                focused_pane_id: tab.focused_pane_id,
                zoomed_pane_id: tab.zoomed_pane_id,
            })
            .collect(),
    }
}

fn layout_pane_count(node: &LayoutNode) -> usize {
    match node {
        LayoutNode::Pane(_) => 1,
        LayoutNode::Split { first, second, .. } => {
            layout_pane_count(first).saturating_add(layout_pane_count(second))
        }
    }
}

fn exact_snapshot_frame_with_worst_case_envelope(
    owner_generation_id: OwnerGenerationId,
    snapshot: MultiplexerSnapshotV2,
) -> Result<Vec<u8>, MultiplexerErrorKind> {
    const MAX_CONNECTION_ID_BYTES: usize = 128;
    let connection_id = ClientConnectionId::new(&"c".repeat(MAX_CONNECTION_ID_BYTES))
        .map_err(|_| MultiplexerErrorKind::OutcomeUnknown)?;
    let sequence =
        EventSequence::new(u64::MAX).map_err(|_| MultiplexerErrorKind::OutcomeUnknown)?;
    let message = ProtocolMessage::new(
        connection_id,
        sequence,
        None,
        owner_generation_id,
        Some(sequence),
        ProtocolPayload::MultiplexerSnapshot { snapshot },
    )
    .map_err(|error| match error {
        LocalControlErrorKind::OversizedFrame => MultiplexerErrorKind::SnapshotLimitExceeded,
        _ => MultiplexerErrorKind::OutcomeUnknown,
    })?;
    let frame = encode_frame(&message).map_err(|error| match error {
        LocalControlErrorKind::OversizedFrame => MultiplexerErrorKind::SnapshotLimitExceeded,
        _ => MultiplexerErrorKind::OutcomeUnknown,
    })?;
    if frame.len() > MAX_CONTROL_FRAME_BYTES {
        return Err(MultiplexerErrorKind::SnapshotLimitExceeded);
    }
    Ok(frame)
}

pub(crate) fn validate_candidate_topology_v2(
    topology: &MultiplexerTopology,
    owner_generation_id: OwnerGenerationId,
) -> Result<(), MultiplexerErrorKind> {
    if topology.workspaces().len() > MAX_V2_WORKSPACES {
        return Err(MultiplexerErrorKind::SnapshotLimitExceeded);
    }

    let mut aggregate_panes = 0_usize;
    let mut summaries = Vec::with_capacity(topology.workspaces().len());
    for workspace in topology.workspaces() {
        if workspace.alias.len() > MAX_V2_ALIAS_BYTES
            || workspace.alias.contains('\0')
            || workspace.tabs.is_empty()
            || workspace.tabs.len() > MAX_V2_TABS_PER_WORKSPACE
        {
            return Err(MultiplexerErrorKind::SnapshotLimitExceeded);
        }

        let mut workspace_panes = 0_usize;
        for tab in &workspace.tabs {
            if tab.alias.len() > MAX_V2_ALIAS_BYTES || tab.alias.contains('\0') {
                return Err(MultiplexerErrorKind::SnapshotLimitExceeded);
            }
            let tab_panes = layout_pane_count(&tab.root);
            if tab_panes == 0 || tab_panes > MAX_V2_PANES_PER_TAB {
                return Err(MultiplexerErrorKind::SnapshotLimitExceeded);
            }
            workspace_panes = workspace_panes
                .checked_add(tab_panes)
                .ok_or(MultiplexerErrorKind::SnapshotLimitExceeded)?;
        }

        aggregate_panes = aggregate_panes
            .checked_add(workspace_panes)
            .ok_or(MultiplexerErrorKind::SnapshotLimitExceeded)?;
        if aggregate_panes > MAX_V2_AGGREGATE_PANES {
            return Err(MultiplexerErrorKind::SnapshotLimitExceeded);
        }

        summaries.push(ProtocolWorkspaceSummaryV2 {
            multiplexer_workspace_id: workspace.id,
            alias: workspace.alias.clone(),
            topology_generation: topology.generation(),
            is_focused: topology.focused_workspace_id() == Some(workspace.id),
        });

        exact_snapshot_frame_with_worst_case_envelope(
            owner_generation_id,
            MultiplexerSnapshotV2::Workspace {
                snapshot: protocol_workspace_snapshot(workspace, topology.generation()),
            },
        )?;
    }

    exact_snapshot_frame_with_worst_case_envelope(
        owner_generation_id,
        MultiplexerSnapshotV2::WorkspaceList {
            topology_generation: topology.generation(),
            workspaces: summaries,
        },
    )?;
    Ok(())
}

pub(super) fn encode_v2_body(payload: &ProtocolPayload) -> ProtocolResult<Value> {
    match payload {
        ProtocolPayload::ListMultiplexerWorkspaces { request } => to_value(request),
        ProtocolPayload::MultiplexerSnapshot { snapshot } => to_value(snapshot),
        ProtocolPayload::RequestMultiplexerWrite | ProtocolPayload::ReleaseMultiplexerWrite => {
            to_value(&EmptyV2 {})
        }
        ProtocolPayload::MultiplexerWriteState { state } => to_value(state),
        ProtocolPayload::ApplyTopologyOperation { request } => to_value(request),
        ProtocolPayload::MultiplexerEvent { event } => to_value(event),
        ProtocolPayload::ListAgentObservations { request } => to_value(request),
        ProtocolPayload::AgentObservationSnapshot { snapshot } => to_value(snapshot),
        ProtocolPayload::AgentObservationEvent { event } => to_value(event),
        ProtocolPayload::ListWorktrees { request } => to_value(request),
        ProtocolPayload::ApplyWorktreeOperation { request } => to_value(request),
        ProtocolPayload::WorktreeOperationResult { result } => to_value(result),
        ProtocolPayload::AttentionSnapshot { snapshot } => to_value(snapshot),
        ProtocolPayload::AttentionEvent { event } => to_value(event),
        _ => Err(LocalControlErrorKind::MalformedFrame),
    }
}

pub(super) fn decode_v2_body(kind: MessageKind, body: Value) -> ProtocolResult<ProtocolPayload> {
    match kind {
        MessageKind::ListMultiplexerWorkspaces => Ok(ProtocolPayload::ListMultiplexerWorkspaces {
            request: from_value(body)?,
        }),
        MessageKind::MultiplexerSnapshot => Ok(ProtocolPayload::MultiplexerSnapshot {
            snapshot: from_value(body)?,
        }),
        MessageKind::RequestMultiplexerWrite => {
            from_value::<EmptyV2>(body)?;
            Ok(ProtocolPayload::RequestMultiplexerWrite)
        }
        MessageKind::ReleaseMultiplexerWrite => {
            from_value::<EmptyV2>(body)?;
            Ok(ProtocolPayload::ReleaseMultiplexerWrite)
        }
        MessageKind::MultiplexerWriteState => Ok(ProtocolPayload::MultiplexerWriteState {
            state: from_value(body)?,
        }),
        MessageKind::ApplyTopologyOperation => Ok(ProtocolPayload::ApplyTopologyOperation {
            request: from_value(body)?,
        }),
        MessageKind::MultiplexerEvent => Ok(ProtocolPayload::MultiplexerEvent {
            event: from_value(body)?,
        }),
        MessageKind::ListAgentObservations => Ok(ProtocolPayload::ListAgentObservations {
            request: from_value(body)?,
        }),
        MessageKind::AgentObservationSnapshot => Ok(ProtocolPayload::AgentObservationSnapshot {
            snapshot: from_value(body)?,
        }),
        MessageKind::AgentObservationEvent => Ok(ProtocolPayload::AgentObservationEvent {
            event: from_value(body)?,
        }),
        MessageKind::ListWorktrees => Ok(ProtocolPayload::ListWorktrees {
            request: from_value(body)?,
        }),
        MessageKind::ApplyWorktreeOperation => Ok(ProtocolPayload::ApplyWorktreeOperation {
            request: from_value(body)?,
        }),
        MessageKind::WorktreeOperationResult => Ok(ProtocolPayload::WorktreeOperationResult {
            result: from_value(body)?,
        }),
        MessageKind::AttentionSnapshot => Ok(ProtocolPayload::AttentionSnapshot {
            snapshot: from_value(body)?,
        }),
        MessageKind::AttentionEvent => Ok(ProtocolPayload::AttentionEvent {
            event: from_value(body)?,
        }),
        _ => Err(LocalControlErrorKind::MalformedFrame),
    }
}

pub(super) fn validate_v2_payload(payload: &ProtocolPayload) -> ProtocolResult<()> {
    match payload {
        ProtocolPayload::ListMultiplexerWorkspaces { .. }
        | ProtocolPayload::RequestMultiplexerWrite
        | ProtocolPayload::ReleaseMultiplexerWrite
        | ProtocolPayload::MultiplexerWriteState { .. } => Ok(()),
        ProtocolPayload::MultiplexerSnapshot { snapshot } => validate_snapshot(snapshot),
        ProtocolPayload::ApplyTopologyOperation { request } => {
            validate_topology_operation(&request.operation)
        }
        ProtocolPayload::MultiplexerEvent { event } => validate_multiplexer_event(event),
        ProtocolPayload::ListAgentObservations { request } => {
            if let Some(cursor) = &request.cursor {
                validate_cursor(cursor.snapshot_revision)?;
            }
            Ok(())
        }
        ProtocolPayload::AgentObservationSnapshot { snapshot } => {
            validate_revision(snapshot.snapshot_revision)?;
            if snapshot.observations.len() > MAX_V2_AGENT_OBSERVATIONS_PER_PAGE {
                return Err(LocalControlErrorKind::OversizedFrame);
            }
            for observation in &snapshot.observations {
                validate_agent_observation(observation)?;
            }
            validate_next_offset(
                snapshot.page_offset,
                snapshot.observations.len(),
                snapshot
                    .next_cursor
                    .as_ref()
                    .map(|cursor| (cursor.snapshot_revision, cursor.offset)),
                snapshot.snapshot_revision,
            )
        }
        ProtocolPayload::AgentObservationEvent { event } => validate_agent_event(event),
        ProtocolPayload::ListWorktrees { request } => {
            if let Some(cursor) = &request.cursor {
                validate_cursor(cursor.snapshot_revision)?;
            }
            Ok(())
        }
        ProtocolPayload::ApplyWorktreeOperation { request } => {
            validate_worktree_operation(&request.operation)
        }
        ProtocolPayload::WorktreeOperationResult { result } => validate_worktree_result(result),
        ProtocolPayload::AttentionSnapshot { snapshot } => {
            validate_revision(snapshot.snapshot_revision)?;
            if snapshot.items.len() > MAX_V2_ATTENTION_ITEMS_PER_PAGE {
                return Err(LocalControlErrorKind::OversizedFrame);
            }
            for item in &snapshot.items {
                validate_attention_item(item)?;
            }
            Ok(())
        }
        ProtocolPayload::AttentionEvent { event } => validate_attention_event(event),
        _ => Ok(()),
    }
}

fn to_value<T: Serialize>(value: &T) -> ProtocolResult<Value> {
    serde_json::to_value(value).map_err(|_| LocalControlErrorKind::MalformedFrame)
}

fn from_value<T: for<'de> Deserialize<'de>>(value: Value) -> ProtocolResult<T> {
    serde_json::from_value(value).map_err(|_| LocalControlErrorKind::MalformedFrame)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyV2 {}

fn validate_bounded_text(value: &str, max_bytes: usize) -> ProtocolResult<()> {
    if value.len() > max_bytes || value.contains('\0') {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(())
}

fn validate_nonempty_text(value: &str, max_bytes: usize) -> ProtocolResult<()> {
    validate_bounded_text(value, max_bytes)?;
    if value.is_empty() {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(())
}

fn validate_revision(value: u64) -> ProtocolResult<()> {
    if value == 0 {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(())
}

fn validate_cursor(revision: u64) -> ProtocolResult<()> {
    validate_revision(revision)
}

fn validate_next_offset(
    page_offset: u16,
    item_count: usize,
    next_cursor: Option<(u64, u16)>,
    snapshot_revision: u64,
) -> ProtocolResult<()> {
    if let Some((cursor_revision, cursor_offset)) = next_cursor {
        validate_cursor(cursor_revision)?;
        if cursor_revision != snapshot_revision {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        let item_count =
            u16::try_from(item_count).map_err(|_| LocalControlErrorKind::OversizedFrame)?;
        let expected_offset = page_offset
            .checked_add(item_count)
            .ok_or(LocalControlErrorKind::OversizedFrame)?;
        if cursor_offset != expected_offset {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
    }
    Ok(())
}

fn validate_ratio(value: u16) -> ProtocolResult<()> {
    if !(1_000..=9_000).contains(&value) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(())
}

fn validate_layout_node(
    node: &ProtocolLayoutNodeV2,
    pane_ids: &mut BTreeSet<PaneId>,
) -> ProtocolResult<()> {
    match node {
        ProtocolLayoutNodeV2::Pane { pane_id } => {
            if !pane_ids.insert(*pane_id) {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
        }
        ProtocolLayoutNodeV2::Split {
            ratio_basis_points,
            first,
            second,
            ..
        } => {
            validate_ratio(*ratio_basis_points)?;
            validate_layout_node(first, pane_ids)?;
            validate_layout_node(second, pane_ids)?;
        }
    }
    Ok(())
}

fn validate_workspace_snapshot(snapshot: &ProtocolWorkspaceSnapshotV2) -> ProtocolResult<()> {
    validate_bounded_text(&snapshot.alias, MAX_V2_ALIAS_BYTES)?;
    if snapshot.tabs.is_empty() || snapshot.tabs.len() > MAX_V2_TABS_PER_WORKSPACE {
        return Err(LocalControlErrorKind::MalformedFrame);
    }

    let mut tab_ids = BTreeSet::new();
    let mut pane_ids = BTreeSet::new();
    for tab in &snapshot.tabs {
        if !tab_ids.insert(tab.tab_id) {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        validate_bounded_text(&tab.alias, MAX_V2_ALIAS_BYTES)?;
        let before = pane_ids.len();
        validate_layout_node(&tab.root, &mut pane_ids)?;
        let tab_pane_count = pane_ids.len().saturating_sub(before);
        if tab_pane_count == 0
            || tab_pane_count > MAX_V2_PANES_PER_TAB
            || pane_ids.len() > MAX_V2_AGGREGATE_PANES
            || !pane_ids.contains(&tab.focused_pane_id)
        {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        if let Some(zoomed) = tab.zoomed_pane_id
            && !pane_ids.contains(&zoomed)
        {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
    }
    if !tab_ids.contains(&snapshot.focused_tab_id) {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(())
}

fn validate_snapshot(snapshot: &MultiplexerSnapshotV2) -> ProtocolResult<()> {
    match snapshot {
        MultiplexerSnapshotV2::WorkspaceList { workspaces, .. } => {
            if workspaces.len() > MAX_V2_WORKSPACES {
                return Err(LocalControlErrorKind::OversizedFrame);
            }
            let mut ids = BTreeSet::new();
            for workspace in workspaces {
                if !ids.insert(workspace.multiplexer_workspace_id) {
                    return Err(LocalControlErrorKind::MalformedFrame);
                }
                validate_bounded_text(&workspace.alias, MAX_V2_ALIAS_BYTES)?;
            }
            Ok(())
        }
        MultiplexerSnapshotV2::Workspace { snapshot } => validate_workspace_snapshot(snapshot),
        MultiplexerSnapshotV2::MutationResult { result, snapshot } => {
            if matches!(
                result.outcome,
                TopologyMutationOutcomeV2::Rejected {
                    error: MultiplexerErrorKind::SnapshotLimitExceeded
                }
            ) && (result.accepted_topology_generation.is_some() || snapshot.is_some())
            {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            match result.outcome {
                TopologyMutationOutcomeV2::Accepted
                    if result.accepted_topology_generation.is_none() =>
                {
                    return Err(LocalControlErrorKind::MalformedFrame);
                }
                TopologyMutationOutcomeV2::Rejected { .. }
                    if result.accepted_topology_generation.is_some() =>
                {
                    return Err(LocalControlErrorKind::MalformedFrame);
                }
                _ => {}
            }
            if let Some(snapshot) = snapshot {
                validate_workspace_snapshot(snapshot)?;
                if let Some(accepted) = result.accepted_topology_generation
                    && accepted != snapshot.topology_generation
                {
                    return Err(LocalControlErrorKind::MalformedFrame);
                }
            }
            Ok(())
        }
    }
}

fn validate_topology_operation(operation: &TopologyOperationV2) -> ProtocolResult<()> {
    match operation {
        TopologyOperationV2::CreateWorkspace {
            alias,
            first_tab_alias,
            ..
        } => {
            validate_bounded_text(alias, MAX_V2_ALIAS_BYTES)?;
            validate_bounded_text(first_tab_alias, MAX_V2_ALIAS_BYTES)
        }
        TopologyOperationV2::RenameWorkspace { alias, .. }
        | TopologyOperationV2::CreateTab { alias, .. }
        | TopologyOperationV2::RenameTab { alias, .. } => {
            validate_bounded_text(alias, MAX_V2_ALIAS_BYTES)
        }
        TopologyOperationV2::MoveWorkspace { new_index, .. } => {
            if usize::from(*new_index) >= MAX_V2_WORKSPACES {
                Err(LocalControlErrorKind::MalformedFrame)
            } else {
                Ok(())
            }
        }
        TopologyOperationV2::MoveTab { new_index, .. } => {
            if usize::from(*new_index) >= MAX_V2_TABS_PER_WORKSPACE {
                Err(LocalControlErrorKind::MalformedFrame)
            } else {
                Ok(())
            }
        }
        TopologyOperationV2::SplitPane {
            ratio_basis_points, ..
        }
        | TopologyOperationV2::MovePane {
            ratio_basis_points, ..
        }
        | TopologyOperationV2::ResizeSplit {
            ratio_basis_points, ..
        } => validate_ratio(*ratio_basis_points),
        TopologyOperationV2::ClearPane {
            presentation_epoch, ..
        } if *presentation_epoch == 0 => Err(LocalControlErrorKind::MalformedFrame),
        _ => Ok(()),
    }
}

fn validate_multiplexer_event(event: &MultiplexerEventV2) -> ProtocolResult<()> {
    match event {
        MultiplexerEventV2::PaneCleared {
            presentation_epoch, ..
        } if *presentation_epoch == 0 => Err(LocalControlErrorKind::MalformedFrame),
        _ => Ok(()),
    }
}

fn validate_agent_observation(observation: &AgentObservationV2) -> ProtocolResult<()> {
    if let Some(git_workspace_id) = &observation.git_workspace_id {
        validate_nonempty_text(git_workspace_id, MAX_V2_GIT_WORKSPACE_ID_BYTES)?;
    }
    if let Some(provider_native_session_id) = &observation.provider_native_session_id {
        validate_nonempty_text(provider_native_session_id, MAX_V2_PROVIDER_SESSION_ID_BYTES)?;
    }
    validate_bounded_text(
        &observation.structured_evidence_summary,
        MAX_V2_EVIDENCE_SUMMARY_BYTES,
    )
}

fn validate_agent_event(event: &AgentObservationEventV2) -> ProtocolResult<()> {
    match event {
        AgentObservationEventV2::Upsert {
            snapshot_revision,
            observation,
        } => {
            validate_revision(*snapshot_revision)?;
            validate_agent_observation(observation)
        }
        AgentObservationEventV2::Removed {
            snapshot_revision, ..
        } => validate_revision(*snapshot_revision),
        AgentObservationEventV2::HistoryGap {
            last_known_snapshot_revision,
        } => validate_revision(*last_known_snapshot_revision),
    }
}

fn validate_oid(value: &str) -> ProtocolResult<()> {
    if !(7..=64).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(LocalControlErrorKind::MalformedFrame);
    }
    Ok(())
}

fn validate_worktree(worktree: &WorktreeObservationV2) -> ProtocolResult<()> {
    validate_nonempty_text(&worktree.git_workspace_id, MAX_V2_GIT_WORKSPACE_ID_BYTES)?;
    validate_nonempty_text(
        &worktree.repository_identity,
        MAX_V2_REPOSITORY_IDENTITY_BYTES,
    )?;
    validate_nonempty_text(&worktree.canonical_path, MAX_V2_PATH_BYTES)?;
    if let Some(branch) = &worktree.branch {
        validate_nonempty_text(branch, MAX_V2_BRANCH_BYTES)?;
    }
    validate_oid(&worktree.head_oid)
}

fn validate_worktree_operation(operation: &WorktreeOperationV2) -> ProtocolResult<()> {
    match operation {
        WorktreeOperationV2::Create {
            repository_identity,
            destination_path,
            base_commit_oid,
            new_branch_name,
        } => {
            validate_nonempty_text(repository_identity, MAX_V2_REPOSITORY_IDENTITY_BYTES)?;
            validate_nonempty_text(destination_path, MAX_V2_PATH_BYTES)?;
            validate_oid(base_commit_oid)?;
            if let Some(branch) = new_branch_name {
                validate_bounded_text(branch, MAX_V2_BRANCH_BYTES)?;
            }
            Ok(())
        }
        WorktreeOperationV2::Open { canonical_path } => {
            validate_nonempty_text(canonical_path, MAX_V2_PATH_BYTES)
        }
        WorktreeOperationV2::Remove {
            git_workspace_id,
            canonical_path,
        } => {
            validate_nonempty_text(git_workspace_id, MAX_V2_GIT_WORKSPACE_ID_BYTES)?;
            validate_nonempty_text(canonical_path, MAX_V2_PATH_BYTES)
        }
    }
}

fn validate_worktree_result(result: &WorktreeOperationResultV2) -> ProtocolResult<()> {
    validate_revision(result.snapshot_revision)?;
    if result.worktrees.len() > MAX_V2_WORKTREES_PER_PAGE {
        return Err(LocalControlErrorKind::OversizedFrame);
    }
    for worktree in &result.worktrees {
        validate_worktree(worktree)?;
    }
    validate_next_offset(
        result.page_offset,
        result.worktrees.len(),
        result
            .next_cursor
            .as_ref()
            .map(|cursor| (cursor.snapshot_revision, cursor.offset)),
        result.snapshot_revision,
    )
}

pub(super) fn validate_v2_owner_generation_binding(
    payload: &ProtocolPayload,
    owner_generation_id: OwnerGenerationId,
) -> ProtocolResult<()> {
    match payload {
        ProtocolPayload::ListAgentObservations { request } => {
            if let Some(cursor) = &request.cursor
                && cursor.owner_generation_id != owner_generation_id
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::AgentObservationSnapshot { snapshot } => {
            if snapshot
                .observations
                .iter()
                .any(|observation| observation.owner_generation_id != owner_generation_id)
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
            if let Some(cursor) = &snapshot.next_cursor
                && cursor.owner_generation_id != owner_generation_id
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::AgentObservationEvent { event } => {
            if let AgentObservationEventV2::Upsert { observation, .. } = event
                && observation.owner_generation_id != owner_generation_id
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::ListWorktrees { request } => {
            if let Some(cursor) = &request.cursor
                && cursor.owner_generation_id != owner_generation_id
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::WorktreeOperationResult { result } => {
            if let Some(cursor) = &result.next_cursor
                && cursor.owner_generation_id != owner_generation_id
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::AttentionSnapshot { snapshot } => {
            if snapshot
                .items
                .iter()
                .any(|item| item.owner_generation_id != owner_generation_id)
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        ProtocolPayload::AttentionEvent { event } => {
            if let AttentionEventV2::Upsert { item, .. } = event
                && item.owner_generation_id != owner_generation_id
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn validate_v2_response_binding(
    request: &ProtocolPayload,
    response: &ProtocolPayload,
) -> ProtocolResult<()> {
    match (request, response) {
        (
            ProtocolPayload::ListAgentObservations { request },
            ProtocolPayload::AgentObservationSnapshot { snapshot },
        ) => {
            let expected_offset = request.cursor.as_ref().map_or(0, |cursor| cursor.offset);
            if snapshot.page_offset != expected_offset {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            if let Some(cursor) = &request.cursor
                && cursor.snapshot_revision != snapshot.snapshot_revision
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        (
            ProtocolPayload::ListWorktrees { request },
            ProtocolPayload::WorktreeOperationResult { result },
        ) => {
            if result.outcome != WorktreeOperationOutcomeV2::Accepted {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            let expected_offset = request.cursor.as_ref().map_or(0, |cursor| cursor.offset);
            if result.page_offset != expected_offset {
                return Err(LocalControlErrorKind::MalformedFrame);
            }
            if let Some(cursor) = &request.cursor
                && cursor.snapshot_revision != result.snapshot_revision
            {
                return Err(LocalControlErrorKind::StaleOwnerGeneration);
            }
        }
        (
            ProtocolPayload::ApplyWorktreeOperation { .. },
            ProtocolPayload::WorktreeOperationResult { result },
        ) if result.page_offset != 0 || result.next_cursor.is_some() => {
            return Err(LocalControlErrorKind::MalformedFrame);
        }
        _ => {}
    }
    Ok(())
}

fn validate_attention_item(item: &AttentionItemV2) -> ProtocolResult<()> {
    validate_nonempty_text(&item.source_domain, MAX_V2_DETAIL_BYTES)?;
    validate_nonempty_text(&item.source_event_id, MAX_V2_DETAIL_BYTES)?;
    validate_bounded_text(&item.detail, MAX_V2_DETAIL_BYTES)
}

fn validate_attention_event(event: &AttentionEventV2) -> ProtocolResult<()> {
    match event {
        AttentionEventV2::Upsert {
            snapshot_revision,
            item,
        } => {
            validate_revision(*snapshot_revision)?;
            validate_attention_item(item)
        }
        AttentionEventV2::Removed {
            snapshot_revision,
            source_domain,
            source_event_id,
            ..
        } => {
            validate_revision(*snapshot_revision)?;
            validate_nonempty_text(source_domain, MAX_V2_DETAIL_BYTES)?;
            validate_nonempty_text(source_event_id, MAX_V2_DETAIL_BYTES)
        }
        AttentionEventV2::HistoryGap {
            last_known_snapshot_revision,
        } => validate_revision(*last_known_snapshot_revision),
    }
}
