use crate::multiplexer::domain::{
    MultiplexerErrorKind, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};
use crate::persistent_runtime::domain::{LocalControlErrorKind, OwnerGenerationId};
use crate::persistent_runtime::protocol::{
    ApplyTopologyOperationV2, MultiplexerEventSubscriptionAckV2, MultiplexerEventV2,
    MultiplexerSnapshotV2, MultiplexerSubscriptionBoundaryV2, MultiplexerSubscriptionStreamV2,
    ProtocolLayoutNodeV2, ProtocolPaneClosePolicy, ProtocolPanePlacement, ProtocolSplitAxis,
    ProtocolWorkspaceSnapshotV2, SubscribeMultiplexerEventsV2, TopologyOperationV2,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopologyProjectionFreshness {
    Unsubscribed,
    NeedsSnapshot,
    Current,
    NeedsResubscribe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TopologyProjectionError {
    Protocol(LocalControlErrorKind),
    Multiplexer(MultiplexerErrorKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TopologyProjection {
    owner_generation_id: OwnerGenerationId,
    subscription: Option<SubscribeMultiplexerEventsV2>,
    subscription_boundary: Option<TopologyGeneration>,
    observed_generation: Option<TopologyGeneration>,
    trusted_snapshot: Option<MultiplexerSnapshotV2>,
    freshness: TopologyProjectionFreshness,
}

impl TopologyProjection {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            subscription: None,
            subscription_boundary: None,
            observed_generation: None,
            trusted_snapshot: None,
            freshness: TopologyProjectionFreshness::Unsubscribed,
        }
    }

    pub(crate) fn reset_for_connection(&mut self, owner_generation_id: OwnerGenerationId) {
        *self = Self::new(owner_generation_id);
    }

    pub(crate) fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }

    pub(crate) fn freshness(&self) -> TopologyProjectionFreshness {
        self.freshness
    }

    pub(crate) fn subscription(&self) -> Option<&SubscribeMultiplexerEventsV2> {
        self.subscription.as_ref()
    }

    pub(crate) fn subscription_filter(&self) -> Option<Option<MultiplexerWorkspaceId>> {
        self.subscription
            .as_ref()
            .map(|subscription| subscription.multiplexer_workspace_id)
    }

    pub(crate) fn observed_generation(&self) -> Option<TopologyGeneration> {
        self.observed_generation
    }

    pub(crate) fn trusted_snapshot(&self) -> Option<&MultiplexerSnapshotV2> {
        (self.freshness == TopologyProjectionFreshness::Current)
            .then_some(self.trusted_snapshot.as_ref())
            .flatten()
    }

    pub(crate) fn accept_subscription_ack(
        &mut self,
        request: &SubscribeMultiplexerEventsV2,
        ack: &MultiplexerEventSubscriptionAckV2,
    ) -> Result<TopologyGeneration, TopologyProjectionError> {
        if request.stream != MultiplexerSubscriptionStreamV2::Topology
            || ack.stream != MultiplexerSubscriptionStreamV2::Topology
            || ack.multiplexer_workspace_id != request.multiplexer_workspace_id
        {
            return Err(TopologyProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ));
        }
        let MultiplexerSubscriptionBoundaryV2::Topology {
            topology_generation,
        } = ack.boundary
        else {
            return Err(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        };

        self.subscription = Some(request.clone());
        self.subscription_boundary = Some(topology_generation);
        self.observed_generation = Some(topology_generation);
        self.trusted_snapshot = None;
        self.freshness = TopologyProjectionFreshness::NeedsSnapshot;
        Ok(topology_generation)
    }

    pub(crate) fn accept_snapshot(
        &mut self,
        snapshot: &MultiplexerSnapshotV2,
    ) -> Result<TopologyGeneration, TopologyProjectionError> {
        if self.freshness == TopologyProjectionFreshness::NeedsResubscribe {
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }
        let subscription = self
            .subscription
            .as_ref()
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ))?;
        let generation = match (subscription.multiplexer_workspace_id, snapshot) {
            (
                None,
                MultiplexerSnapshotV2::WorkspaceList {
                    topology_generation,
                    ..
                },
            ) => *topology_generation,
            (Some(expected_workspace_id), MultiplexerSnapshotV2::Workspace { snapshot })
                if snapshot.multiplexer_workspace_id == expected_workspace_id =>
            {
                snapshot.topology_generation
            }
            _ => {
                return Err(TopologyProjectionError::Protocol(
                    LocalControlErrorKind::MalformedFrame,
                ));
            }
        };

        let boundary = self
            .subscription_boundary
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ))?;
        let observed = self.observed_generation.unwrap_or(boundary);
        if generation < boundary || generation < observed {
            self.trusted_snapshot = None;
            self.freshness = TopologyProjectionFreshness::NeedsResubscribe;
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }

        self.observed_generation = Some(generation);
        self.trusted_snapshot = Some(snapshot.clone());
        self.freshness = TopologyProjectionFreshness::Current;
        Ok(generation)
    }

    pub(crate) fn accept_event(
        &mut self,
        event: &MultiplexerEventV2,
    ) -> Result<(), TopologyProjectionError> {
        let subscription = self
            .subscription
            .as_ref()
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::UnsupportedOperation,
            ))?;
        if self.freshness == TopologyProjectionFreshness::NeedsResubscribe {
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }

        let (workspace_id, generation) = match event {
            MultiplexerEventV2::TopologyChanged {
                multiplexer_workspace_id,
                topology_generation,
            }
            | MultiplexerEventV2::PaneCleared {
                multiplexer_workspace_id,
                topology_generation,
                ..
            } => (Some(*multiplexer_workspace_id), Some(*topology_generation)),
            MultiplexerEventV2::HistoryGap {
                last_known_topology_generation,
            } => {
                self.observed_generation = Some(*last_known_topology_generation);
                self.trusted_snapshot = None;
                self.freshness = TopologyProjectionFreshness::NeedsResubscribe;
                return Ok(());
            }
        };

        if let (Some(expected), Some(actual)) =
            (subscription.multiplexer_workspace_id, workspace_id)
            && expected != actual
        {
            return Err(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ));
        }
        let generation = generation.ok_or(TopologyProjectionError::Protocol(
            LocalControlErrorKind::MalformedFrame,
        ))?;
        let previous = self
            .observed_generation
            .or(self.subscription_boundary)
            .ok_or(TopologyProjectionError::Protocol(
                LocalControlErrorKind::MalformedFrame,
            ))?;
        if generation <= previous {
            return Err(TopologyProjectionError::Multiplexer(
                MultiplexerErrorKind::StaleTopologyGeneration,
            ));
        }

        if subscription.multiplexer_workspace_id.is_none() {
            let expected = previous.checked_next().map_err(|_| {
                TopologyProjectionError::Protocol(LocalControlErrorKind::MalformedFrame)
            })?;
            if generation != expected {
                self.observed_generation = Some(generation);
                self.trusted_snapshot = None;
                self.freshness = TopologyProjectionFreshness::NeedsResubscribe;
                return Ok(());
            }
        }

        self.observed_generation = Some(generation);
        self.trusted_snapshot = None;
        self.freshness = TopologyProjectionFreshness::NeedsSnapshot;
        Ok(())
    }
}

const TUI_TOPOLOGY_AUTHORITY_LABEL: &str = "READ_ONLY_TOPOLOGY";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TuiTopologyTarget {
    Workspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
    },
    Tab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    },
    Pane {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
    },
}

impl TuiTopologyTarget {
    fn canonical_id(self) -> String {
        match self {
            Self::Workspace {
                multiplexer_workspace_id,
            } => multiplexer_workspace_id.to_string(),
            Self::Tab { tab_id, .. } => tab_id.to_string(),
            Self::Pane { pane_id, .. } => pane_id.to_string(),
        }
    }

    fn stable_key(self) -> String {
        match self {
            Self::Workspace {
                multiplexer_workspace_id,
            } => format!("0-workspace-{multiplexer_workspace_id}"),
            Self::Tab {
                multiplexer_workspace_id,
                tab_id,
            } => format!("1-tab-{multiplexer_workspace_id}-{tab_id}"),
            Self::Pane {
                multiplexer_workspace_id,
                tab_id,
                pane_id,
            } => format!("2-pane-{multiplexer_workspace_id}-{tab_id}-{pane_id}"),
        }
    }

    fn kind_label(self) -> &'static str {
        match self {
            Self::Workspace { .. } => "WORKSPACE",
            Self::Tab { .. } => "TAB",
            Self::Pane { .. } => "PANE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TuiTopologyRow {
    pub(crate) target: TuiTopologyTarget,
    pub(crate) display_label: String,
    pub(crate) depth: u8,
    pub(crate) focused: Option<bool>,
    pub(crate) zoomed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TuiTopologyProjectionError {
    UntrustedSnapshot,
    UnsupportedSnapshot,
    GenerationMismatch,
    IdentityCollision,
    UnknownTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TuiTopologyFindRank {
    ExactCanonicalId,
    ExactNormalizedLabel,
    NormalizedPrefix,
    NormalizedSubstring,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiTopologyFindResolution {
    NotFound,
    Unique(TuiTopologyTarget),
    Ambiguous(Vec<TuiTopologyTarget>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TuiTopologyHitRegion {
    pub(crate) target: TuiTopologyTarget,
    pub(crate) column: u16,
    pub(crate) row: u16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

impl TuiTopologyHitRegion {
    fn contains(self, column: u16, row: u16) -> bool {
        column >= self.column
            && row >= self.row
            && column < self.column.saturating_add(self.width)
            && row < self.row.saturating_add(self.height)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TuiTopologyAccessibility {
    pub(crate) reduced_motion: bool,
    pub(crate) high_contrast: bool,
    pub(crate) scaled_text: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiTopologyCommand {
    CreateWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        alias: String,
        first_tab_id: TabId,
        first_tab_alias: String,
        first_pane_id: PaneId,
    },
    CreateTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        alias: String,
        first_pane_id: PaneId,
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
    Focus(TuiTopologyTarget),
    MoveWorkspace {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        new_index: u16,
    },
    MoveTab {
        multiplexer_workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        new_index: u16,
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
    Close(TuiTopologyTarget),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TuiTopologyProjection {
    topology_generation: TopologyGeneration,
    rows: Vec<TuiTopologyRow>,
}

impl TuiTopologyProjection {
    pub(crate) fn from_trusted_projection(
        projection: &TopologyProjection,
    ) -> Result<Self, TuiTopologyProjectionError> {
        let snapshot = projection
            .trusted_snapshot()
            .ok_or(TuiTopologyProjectionError::UntrustedSnapshot)?;
        Self::from_snapshot(snapshot)
    }

    pub(crate) fn from_snapshot(
        snapshot: &MultiplexerSnapshotV2,
    ) -> Result<Self, TuiTopologyProjectionError> {
        match snapshot {
            MultiplexerSnapshotV2::WorkspaceList {
                topology_generation,
                workspaces,
            } => {
                let mut seen = BTreeSet::new();
                let mut rows = Vec::with_capacity(workspaces.len());
                for workspace in workspaces {
                    if workspace.topology_generation != *topology_generation {
                        return Err(TuiTopologyProjectionError::GenerationMismatch);
                    }
                    let target = TuiTopologyTarget::Workspace {
                        multiplexer_workspace_id: workspace.multiplexer_workspace_id,
                    };
                    if !seen.insert(target) {
                        return Err(TuiTopologyProjectionError::IdentityCollision);
                    }
                    rows.push(TuiTopologyRow {
                        target,
                        display_label: workspace.alias.clone(),
                        depth: 0,
                        focused: Some(workspace.is_focused),
                        zoomed: false,
                    });
                }
                Ok(Self {
                    topology_generation: *topology_generation,
                    rows,
                })
            }
            MultiplexerSnapshotV2::Workspace { snapshot } => {
                Self::from_workspace_snapshot(snapshot)
            }
            MultiplexerSnapshotV2::MutationResult { .. } => {
                Err(TuiTopologyProjectionError::UnsupportedSnapshot)
            }
        }
    }

    fn from_workspace_snapshot(
        snapshot: &ProtocolWorkspaceSnapshotV2,
    ) -> Result<Self, TuiTopologyProjectionError> {
        let workspace_target = TuiTopologyTarget::Workspace {
            multiplexer_workspace_id: snapshot.multiplexer_workspace_id,
        };
        let mut rows = vec![TuiTopologyRow {
            target: workspace_target,
            display_label: snapshot.alias.clone(),
            depth: 0,
            focused: None,
            zoomed: false,
        }];
        let mut seen_targets = BTreeSet::from([workspace_target]);
        let mut seen_tabs = BTreeSet::new();
        let mut seen_panes = BTreeSet::new();
        let mut focused_tab_present = false;

        for tab in &snapshot.tabs {
            if !seen_tabs.insert(tab.tab_id) {
                return Err(TuiTopologyProjectionError::IdentityCollision);
            }
            let tab_focused = tab.tab_id == snapshot.focused_tab_id;
            focused_tab_present |= tab_focused;
            let tab_target = TuiTopologyTarget::Tab {
                multiplexer_workspace_id: snapshot.multiplexer_workspace_id,
                tab_id: tab.tab_id,
            };
            if !seen_targets.insert(tab_target) {
                return Err(TuiTopologyProjectionError::IdentityCollision);
            }
            rows.push(TuiTopologyRow {
                target: tab_target,
                display_label: tab.alias.clone(),
                depth: 1,
                focused: Some(tab_focused),
                zoomed: false,
            });

            let mut pane_ids = Vec::new();
            collect_protocol_panes(&tab.root, &mut pane_ids);
            if pane_ids.is_empty()
                || !pane_ids.contains(&tab.focused_pane_id)
                || tab
                    .zoomed_pane_id
                    .is_some_and(|pane_id| !pane_ids.contains(&pane_id))
            {
                return Err(TuiTopologyProjectionError::UnknownTarget);
            }
            for pane_id in pane_ids {
                if !seen_panes.insert(pane_id) {
                    return Err(TuiTopologyProjectionError::IdentityCollision);
                }
                let pane_target = TuiTopologyTarget::Pane {
                    multiplexer_workspace_id: snapshot.multiplexer_workspace_id,
                    tab_id: tab.tab_id,
                    pane_id,
                };
                if !seen_targets.insert(pane_target) {
                    return Err(TuiTopologyProjectionError::IdentityCollision);
                }
                rows.push(TuiTopologyRow {
                    target: pane_target,
                    display_label: pane_id.to_string(),
                    depth: 2,
                    focused: Some(pane_id == tab.focused_pane_id),
                    zoomed: tab.zoomed_pane_id == Some(pane_id),
                });
            }
        }

        if snapshot.tabs.is_empty() || !focused_tab_present {
            return Err(TuiTopologyProjectionError::UnknownTarget);
        }

        Ok(Self {
            topology_generation: snapshot.topology_generation,
            rows,
        })
    }

    pub(crate) const fn topology_generation(&self) -> TopologyGeneration {
        self.topology_generation
    }

    pub(crate) fn rows(&self) -> &[TuiTopologyRow] {
        &self.rows
    }

    pub(crate) const fn authority_label(&self) -> &'static str {
        TUI_TOPOLOGY_AUTHORITY_LABEL
    }

    pub(crate) const fn changes_canonical_authority(&self) -> bool {
        false
    }

    pub(crate) fn find(&self, query: &str) -> TuiTopologyFindResolution {
        let query = normalize_topology_query(query);
        if query.is_empty() {
            return TuiTopologyFindResolution::NotFound;
        }
        let mut matches: Vec<(TuiTopologyFindRank, String, TuiTopologyTarget)> = self
            .rows
            .iter()
            .filter_map(|row| {
                topology_match_rank(&query, row)
                    .map(|rank| (rank, row.target.stable_key(), row.target))
            })
            .collect();
        matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        let Some(best_rank) = matches.first().map(|item| item.0) else {
            return TuiTopologyFindResolution::NotFound;
        };
        let best: Vec<TuiTopologyTarget> = matches
            .into_iter()
            .take_while(|item| item.0 == best_rank)
            .map(|item| item.2)
            .collect();
        if best.len() == 1 {
            TuiTopologyFindResolution::Unique(best[0])
        } else {
            TuiTopologyFindResolution::Ambiguous(best)
        }
    }

    pub(crate) fn pointer_target(
        &self,
        hit_regions: &[TuiTopologyHitRegion],
        column: u16,
        row: u16,
    ) -> Option<TuiTopologyTarget> {
        let matches: BTreeSet<TuiTopologyTarget> = hit_regions
            .iter()
            .filter(|region| region.contains(column, row))
            .map(|region| region.target)
            .filter(|target| self.contains_target(*target))
            .collect();
        (matches.len() == 1).then(|| {
            *matches
                .iter()
                .next()
                .expect("one exact topology hit target was just proven")
        })
    }

    pub(crate) fn pointer_focus_intent(
        &self,
        hit_regions: &[TuiTopologyHitRegion],
        column: u16,
        row: u16,
    ) -> Result<ApplyTopologyOperationV2, TuiTopologyProjectionError> {
        let target = self
            .pointer_target(hit_regions, column, row)
            .ok_or(TuiTopologyProjectionError::UnknownTarget)?;
        self.keyboard_intent(TuiTopologyCommand::Focus(target))
    }

    pub(crate) fn keyboard_intent(
        &self,
        command: TuiTopologyCommand,
    ) -> Result<ApplyTopologyOperationV2, TuiTopologyProjectionError> {
        let operation = match command {
            TuiTopologyCommand::CreateWorkspace {
                multiplexer_workspace_id,
                alias,
                first_tab_id,
                first_tab_alias,
                first_pane_id,
            } => {
                if self.contains_workspace(multiplexer_workspace_id)
                    || self.contains_pane(first_pane_id)
                {
                    return Err(TuiTopologyProjectionError::IdentityCollision);
                }
                TopologyOperationV2::CreateWorkspace {
                    multiplexer_workspace_id,
                    alias,
                    first_tab_id,
                    first_tab_alias,
                    first_pane_id,
                }
            }
            TuiTopologyCommand::CreateTab {
                multiplexer_workspace_id,
                tab_id,
                alias,
                first_pane_id,
            } => {
                self.require_target(TuiTopologyTarget::Workspace {
                    multiplexer_workspace_id,
                })?;
                if self.contains_tab(multiplexer_workspace_id, tab_id)
                    || self.contains_pane(first_pane_id)
                {
                    return Err(TuiTopologyProjectionError::IdentityCollision);
                }
                TopologyOperationV2::CreateTab {
                    multiplexer_workspace_id,
                    tab_id,
                    alias,
                    first_pane_id,
                }
            }
            TuiTopologyCommand::SplitPane {
                multiplexer_workspace_id,
                tab_id,
                target_pane_id,
                new_pane_id,
                axis,
                placement,
                ratio_basis_points,
            } => {
                self.require_target(TuiTopologyTarget::Pane {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id: target_pane_id,
                })?;
                if self.contains_pane(new_pane_id) {
                    return Err(TuiTopologyProjectionError::IdentityCollision);
                }
                TopologyOperationV2::SplitPane {
                    multiplexer_workspace_id,
                    tab_id,
                    target_pane_id,
                    new_pane_id,
                    axis,
                    placement,
                    ratio_basis_points,
                }
            }
            TuiTopologyCommand::Focus(target) => {
                self.require_target(target)?;
                match target {
                    TuiTopologyTarget::Workspace {
                        multiplexer_workspace_id,
                    } => TopologyOperationV2::FocusWorkspace {
                        multiplexer_workspace_id,
                    },
                    TuiTopologyTarget::Tab {
                        multiplexer_workspace_id,
                        tab_id,
                    } => TopologyOperationV2::FocusTab {
                        multiplexer_workspace_id,
                        tab_id,
                    },
                    TuiTopologyTarget::Pane {
                        multiplexer_workspace_id,
                        tab_id,
                        pane_id,
                    } => TopologyOperationV2::FocusPane {
                        multiplexer_workspace_id,
                        tab_id,
                        pane_id,
                    },
                }
            }
            TuiTopologyCommand::MoveWorkspace {
                multiplexer_workspace_id,
                new_index,
            } => {
                self.require_target(TuiTopologyTarget::Workspace {
                    multiplexer_workspace_id,
                })?;
                TopologyOperationV2::MoveWorkspace {
                    multiplexer_workspace_id,
                    new_index,
                }
            }
            TuiTopologyCommand::MoveTab {
                multiplexer_workspace_id,
                tab_id,
                new_index,
            } => {
                self.require_target(TuiTopologyTarget::Tab {
                    multiplexer_workspace_id,
                    tab_id,
                })?;
                TopologyOperationV2::MoveTab {
                    multiplexer_workspace_id,
                    tab_id,
                    new_index,
                }
            }
            TuiTopologyCommand::MovePane {
                multiplexer_workspace_id,
                source_tab_id,
                pane_id,
                destination_tab_id,
                destination_pane_id,
                axis,
                placement,
                ratio_basis_points,
            } => {
                self.require_target(TuiTopologyTarget::Pane {
                    multiplexer_workspace_id,
                    tab_id: source_tab_id,
                    pane_id,
                })?;
                self.require_target(TuiTopologyTarget::Pane {
                    multiplexer_workspace_id,
                    tab_id: destination_tab_id,
                    pane_id: destination_pane_id,
                })?;
                TopologyOperationV2::MovePane {
                    multiplexer_workspace_id,
                    source_tab_id,
                    pane_id,
                    destination_tab_id,
                    destination_pane_id,
                    axis,
                    placement,
                    ratio_basis_points,
                }
            }
            TuiTopologyCommand::ResizeSplit {
                multiplexer_workspace_id,
                tab_id,
                first_pane_id,
                second_pane_id,
                ratio_basis_points,
            } => {
                self.require_target(TuiTopologyTarget::Pane {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id: first_pane_id,
                })?;
                self.require_target(TuiTopologyTarget::Pane {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id: second_pane_id,
                })?;
                TopologyOperationV2::ResizeSplit {
                    multiplexer_workspace_id,
                    tab_id,
                    first_pane_id,
                    second_pane_id,
                    ratio_basis_points,
                }
            }
            TuiTopologyCommand::ToggleZoom {
                multiplexer_workspace_id,
                tab_id,
                pane_id,
            } => {
                self.require_target(TuiTopologyTarget::Pane {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id,
                })?;
                TopologyOperationV2::ToggleZoom {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id,
                }
            }
            TuiTopologyCommand::Close(target) => {
                self.require_target(target)?;
                match target {
                    TuiTopologyTarget::Workspace {
                        multiplexer_workspace_id,
                    } => TopologyOperationV2::CloseWorkspace {
                        multiplexer_workspace_id,
                    },
                    TuiTopologyTarget::Tab {
                        multiplexer_workspace_id,
                        tab_id,
                    } => TopologyOperationV2::CloseTab {
                        multiplexer_workspace_id,
                        tab_id,
                    },
                    TuiTopologyTarget::Pane {
                        multiplexer_workspace_id,
                        tab_id,
                        pane_id,
                    } => TopologyOperationV2::ClosePane {
                        multiplexer_workspace_id,
                        tab_id,
                        pane_id,
                        policy: ProtocolPaneClosePolicy::DetachView,
                    },
                }
            }
        };
        Ok(ApplyTopologyOperationV2 {
            expected_topology_generation: self.topology_generation,
            operation,
        })
    }

    pub(crate) fn render_lines(&self, accessibility: TuiTopologyAccessibility) -> Vec<String> {
        let mut lines = Vec::with_capacity(self.rows.len().saturating_add(1));
        lines.push(format!(
            "TOPOLOGY generation={} authority={} reduced_motion={} high_contrast={} scaled_text={}",
            self.topology_generation.get(),
            self.authority_label(),
            accessibility.reduced_motion,
            accessibility.high_contrast,
            accessibility.scaled_text,
        ));
        for row in &self.rows {
            let indent = "  ".repeat(usize::from(row.depth));
            let focused = row
                .focused
                .map_or("UNKNOWN", |value| if value { "YES" } else { "NO" });
            lines.push(format!(
                "{indent}{} label={} id={} focused={} zoomed={}",
                row.target.kind_label(),
                row.display_label,
                row.target.canonical_id(),
                focused,
                if row.zoomed { "YES" } else { "NO" },
            ));
        }
        lines
    }

    fn contains_target(&self, target: TuiTopologyTarget) -> bool {
        self.rows.iter().any(|row| row.target == target)
    }

    fn contains_workspace(&self, workspace_id: MultiplexerWorkspaceId) -> bool {
        self.rows.iter().any(|row| {
            matches!(
                row.target,
                TuiTopologyTarget::Workspace {
                    multiplexer_workspace_id
                } if multiplexer_workspace_id == workspace_id
            )
        })
    }

    fn contains_tab(&self, workspace_id: MultiplexerWorkspaceId, tab_id: TabId) -> bool {
        self.rows.iter().any(|row| {
            matches!(
                row.target,
                TuiTopologyTarget::Tab {
                    multiplexer_workspace_id,
                    tab_id: candidate_tab_id,
                } if multiplexer_workspace_id == workspace_id && candidate_tab_id == tab_id
            )
        })
    }

    fn contains_pane(&self, pane_id: PaneId) -> bool {
        self.rows.iter().any(|row| {
            matches!(row.target, TuiTopologyTarget::Pane { pane_id: candidate, .. } if candidate == pane_id)
        })
    }

    fn require_target(&self, target: TuiTopologyTarget) -> Result<(), TuiTopologyProjectionError> {
        self.contains_target(target)
            .then_some(())
            .ok_or(TuiTopologyProjectionError::UnknownTarget)
    }
}

fn collect_protocol_panes(node: &ProtocolLayoutNodeV2, pane_ids: &mut Vec<PaneId>) {
    match node {
        ProtocolLayoutNodeV2::Pane { pane_id } => pane_ids.push(*pane_id),
        ProtocolLayoutNodeV2::Split { first, second, .. } => {
            collect_protocol_panes(first, pane_ids);
            collect_protocol_panes(second, pane_ids);
        }
    }
}

fn normalize_topology_query(value: &str) -> String {
    value.trim().chars().flat_map(char::to_lowercase).collect()
}

fn topology_match_rank(query: &str, row: &TuiTopologyRow) -> Option<TuiTopologyFindRank> {
    let canonical_id = normalize_topology_query(&row.target.canonical_id());
    if canonical_id == query {
        return Some(TuiTopologyFindRank::ExactCanonicalId);
    }
    let label = normalize_topology_query(&row.display_label);
    if label == query {
        return Some(TuiTopologyFindRank::ExactNormalizedLabel);
    }
    if canonical_id.starts_with(query) || label.starts_with(query) {
        return Some(TuiTopologyFindRank::NormalizedPrefix);
    }
    (canonical_id.contains(query) || label.contains(query))
        .then_some(TuiTopologyFindRank::NormalizedSubstring)
}

#[cfg(test)]
#[path = "../t168_multiplexer_tui_tests.rs"]
mod t168_multiplexer_tui_tests;
