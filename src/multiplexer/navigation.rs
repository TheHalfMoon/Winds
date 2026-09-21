use super::{MultiplexerErrorKind, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};
use std::collections::BTreeSet;

const GEOMETRY_EXTENT: u32 = 1_000_000;
const MIN_SPLIT_RATIO_BPS: u16 = 1_000;
const MAX_SPLIT_RATIO_BPS: u16 = 9_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanePlacement {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SplitRatioBps(u16);

impl SplitRatioBps {
    pub(crate) fn new(value: u16) -> Result<Self, MultiplexerErrorKind> {
        if !(MIN_SPLIT_RATIO_BPS..=MAX_SPLIT_RATIO_BPS).contains(&value) {
            return Err(MultiplexerErrorKind::UnsupportedOperation);
        }
        Ok(Self(value))
    }

    pub(crate) const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LayoutNode {
    Pane(PaneId),
    Split {
        axis: SplitAxis,
        ratio_bps: SplitRatioBps,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LayoutTemplateNode {
    Pane,
    Split {
        axis: SplitAxis,
        ratio_bps: SplitRatioBps,
        first: Box<LayoutTemplateNode>,
        second: Box<LayoutTemplateNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayoutTemplateTab {
    pub(crate) alias: String,
    pub(crate) root: LayoutTemplateNode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayoutTemplateStructure {
    pub(crate) tabs: Vec<LayoutTemplateTab>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TabState {
    pub(crate) id: TabId,
    pub(crate) alias: String,
    pub(crate) root: LayoutNode,
    pub(crate) focused_pane_id: PaneId,
    pub(crate) zoomed_pane_id: Option<PaneId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkspaceState {
    pub(crate) id: MultiplexerWorkspaceId,
    pub(crate) alias: String,
    pub(crate) tabs: Vec<TabState>,
    pub(crate) focused_tab_id: TabId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MultiplexerTopology {
    generation: TopologyGeneration,
    workspaces: Vec<WorkspaceState>,
    focused_workspace_id: Option<MultiplexerWorkspaceId>,
    retired_workspace_ids: BTreeSet<MultiplexerWorkspaceId>,
    retired_tab_ids: BTreeSet<(MultiplexerWorkspaceId, TabId)>,
    retired_pane_ids: BTreeSet<PaneId>,
}

impl MultiplexerTopology {
    pub(crate) fn empty() -> Self {
        Self {
            generation: TopologyGeneration::initial(),
            workspaces: Vec::new(),
            focused_workspace_id: None,
            retired_workspace_ids: BTreeSet::new(),
            retired_tab_ids: BTreeSet::new(),
            retired_pane_ids: BTreeSet::new(),
        }
    }

    pub(crate) fn restore_presentation(
        generation: TopologyGeneration,
        workspaces: Vec<WorkspaceState>,
        focused_workspace_id: Option<MultiplexerWorkspaceId>,
    ) -> Result<Self, MultiplexerErrorKind> {
        if workspaces.is_empty() {
            if focused_workspace_id.is_some() {
                return Err(MultiplexerErrorKind::UnknownWorkspace);
            }
            return Ok(Self {
                generation,
                workspaces,
                focused_workspace_id: None,
                retired_workspace_ids: BTreeSet::new(),
                retired_tab_ids: BTreeSet::new(),
                retired_pane_ids: BTreeSet::new(),
            });
        }

        let focused_workspace_id =
            focused_workspace_id.ok_or(MultiplexerErrorKind::UnknownWorkspace)?;
        let mut workspace_ids = BTreeSet::new();
        let mut active_pane_ids = BTreeSet::new();
        for workspace in &workspaces {
            if !workspace_ids.insert(workspace.id) {
                return Err(MultiplexerErrorKind::IdentityReuse);
            }
            let mut tab_ids = BTreeSet::new();
            for tab in &workspace.tabs {
                if !tab_ids.insert(tab.id) {
                    return Err(MultiplexerErrorKind::IdentityReuse);
                }
                let mut pane_ids = BTreeSet::new();
                collect_pane_ids(&tab.root, &mut pane_ids);
                if pane_ids.is_empty() || !pane_ids.contains(&tab.focused_pane_id) {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }
                if tab
                    .zoomed_pane_id
                    .is_some_and(|pane_id| !pane_ids.contains(&pane_id))
                {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }
                for pane_id in pane_ids {
                    if !active_pane_ids.insert(pane_id) {
                        return Err(MultiplexerErrorKind::IdentityReuse);
                    }
                }
            }
            if !tab_ids.contains(&workspace.focused_tab_id) {
                return Err(MultiplexerErrorKind::UnknownTab);
            }
        }
        if !workspace_ids.contains(&focused_workspace_id) {
            return Err(MultiplexerErrorKind::UnknownWorkspace);
        }

        Ok(Self {
            generation,
            workspaces,
            focused_workspace_id: Some(focused_workspace_id),
            retired_workspace_ids: BTreeSet::new(),
            retired_tab_ids: BTreeSet::new(),
            retired_pane_ids: BTreeSet::new(),
        })
    }

    pub(crate) const fn generation(&self) -> TopologyGeneration {
        self.generation
    }

    pub(crate) fn workspaces(&self) -> &[WorkspaceState] {
        &self.workspaces
    }

    pub(crate) const fn focused_workspace_id(&self) -> Option<MultiplexerWorkspaceId> {
        self.focused_workspace_id
    }

    pub(crate) fn workspace(
        &self,
        workspace_id: MultiplexerWorkspaceId,
    ) -> Result<&WorkspaceState, MultiplexerErrorKind> {
        self.workspaces
            .iter()
            .find(|workspace| workspace.id == workspace_id)
            .ok_or(MultiplexerErrorKind::UnknownWorkspace)
    }

    pub(crate) fn tab(
        &self,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    ) -> Result<&TabState, MultiplexerErrorKind> {
        self.workspace(workspace_id)?
            .tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .ok_or(MultiplexerErrorKind::UnknownTab)
    }

    pub(crate) fn create_workspace(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        alias: String,
        first_tab_id: TabId,
        first_tab_alias: String,
        first_pane_id: PaneId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            if candidate.retired_workspace_ids.contains(&workspace_id)
                || candidate
                    .workspaces
                    .iter()
                    .any(|workspace| workspace.id == workspace_id)
            {
                return Err(MultiplexerErrorKind::IdentityReuse);
            }
            candidate.require_fresh_tab_id(workspace_id, first_tab_id)?;
            candidate.require_fresh_pane_id(first_pane_id)?;
            candidate.workspaces.push(WorkspaceState {
                id: workspace_id,
                alias,
                tabs: vec![TabState {
                    id: first_tab_id,
                    alias: first_tab_alias,
                    root: LayoutNode::Pane(first_pane_id),
                    focused_pane_id: first_pane_id,
                    zoomed_pane_id: None,
                }],
                focused_tab_id: first_tab_id,
            });
            candidate.focused_workspace_id = Some(workspace_id);
            Ok(())
        })
    }

    pub(crate) fn focus_workspace(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            candidate.workspace(workspace_id)?;
            candidate.focused_workspace_id = Some(workspace_id);
            Ok(())
        })
    }

    pub(crate) fn rename_workspace(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        alias: String,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            candidate.workspace_mut(workspace_id)?.alias = alias;
            Ok(())
        })
    }

    pub(crate) fn move_workspace(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        new_index: usize,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            if new_index >= candidate.workspaces.len() {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            let index = candidate
                .workspaces
                .iter()
                .position(|workspace| workspace.id == workspace_id)
                .ok_or(MultiplexerErrorKind::UnknownWorkspace)?;
            let workspace = candidate.workspaces.remove(index);
            candidate.workspaces.insert(new_index, workspace);
            Ok(())
        })
    }

    pub(crate) fn close_workspace(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            let index = candidate
                .workspaces
                .iter()
                .position(|workspace| workspace.id == workspace_id)
                .ok_or(MultiplexerErrorKind::UnknownWorkspace)?;
            let workspace = candidate.workspaces.remove(index);
            candidate.retired_workspace_ids.insert(workspace_id);
            for tab in &workspace.tabs {
                candidate.retired_tab_ids.insert((workspace_id, tab.id));
                collect_pane_ids(&tab.root, &mut candidate.retired_pane_ids);
            }
            if candidate.focused_workspace_id == Some(workspace_id) {
                candidate.focused_workspace_id = candidate.workspaces.first().map(|item| item.id);
            }
            Ok(())
        })
    }

    pub(crate) fn create_tab(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        alias: String,
        first_pane_id: PaneId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            candidate.require_fresh_tab_id(workspace_id, tab_id)?;
            candidate.require_fresh_pane_id(first_pane_id)?;
            let workspace = candidate.workspace_mut(workspace_id)?;
            workspace.tabs.push(TabState {
                id: tab_id,
                alias,
                root: LayoutNode::Pane(first_pane_id),
                focused_pane_id: first_pane_id,
                zoomed_pane_id: None,
            });
            workspace.focused_tab_id = tab_id;
            candidate.focused_workspace_id = Some(workspace_id);
            Ok(())
        })
    }

    pub(crate) fn focus_tab(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            let workspace = candidate.workspace_mut(workspace_id)?;
            require_tab(workspace, tab_id)?;
            workspace.focused_tab_id = tab_id;
            candidate.focused_workspace_id = Some(workspace_id);
            Ok(())
        })
    }

    pub(crate) fn rename_tab(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        alias: String,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            tab_mut(candidate.workspace_mut(workspace_id)?, tab_id)?.alias = alias;
            Ok(())
        })
    }

    pub(crate) fn move_tab(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        new_index: usize,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            let workspace = candidate.workspace_mut(workspace_id)?;
            if new_index >= workspace.tabs.len() {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            let index = workspace
                .tabs
                .iter()
                .position(|tab| tab.id == tab_id)
                .ok_or(MultiplexerErrorKind::UnknownTab)?;
            let tab = workspace.tabs.remove(index);
            workspace.tabs.insert(new_index, tab);
            Ok(())
        })
    }

    pub(crate) fn close_tab(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            let closed_root = {
                let workspace = candidate.workspace_mut(workspace_id)?;
                if workspace.tabs.len() == 1 {
                    return Err(MultiplexerErrorKind::UnsupportedOperation);
                }
                let index = workspace
                    .tabs
                    .iter()
                    .position(|tab| tab.id == tab_id)
                    .ok_or(MultiplexerErrorKind::UnknownTab)?;
                let tab = workspace.tabs.remove(index);
                if workspace.focused_tab_id == tab_id {
                    workspace.focused_tab_id = workspace.tabs[0].id;
                }
                tab.root
            };
            candidate.retired_tab_ids.insert((workspace_id, tab_id));
            collect_pane_ids(&closed_root, &mut candidate.retired_pane_ids);
            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn split_pane(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        target_pane_id: PaneId,
        new_pane_id: PaneId,
        axis: SplitAxis,
        placement: PanePlacement,
        ratio_bps: SplitRatioBps,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            candidate.require_fresh_pane_id(new_pane_id)?;
            let tab = tab_mut(candidate.workspace_mut(workspace_id)?, tab_id)?;
            if !replace_leaf_with_split(
                &mut tab.root,
                target_pane_id,
                new_pane_id,
                axis,
                placement,
                ratio_bps,
            ) {
                return Err(MultiplexerErrorKind::UnknownPane);
            }
            Ok(())
        })
    }

    pub(crate) fn swap_panes(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        first_pane_id: PaneId,
        second_pane_id: PaneId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            if first_pane_id == second_pane_id {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            let tab = tab_mut(candidate.workspace_mut(workspace_id)?, tab_id)?;
            if !contains_pane(&tab.root, first_pane_id) || !contains_pane(&tab.root, second_pane_id)
            {
                return Err(MultiplexerErrorKind::UnknownPane);
            }
            swap_leaf_ids(&mut tab.root, first_pane_id, second_pane_id);
            Ok(())
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn move_pane(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        source_tab_id: TabId,
        pane_id: PaneId,
        destination_tab_id: TabId,
        destination_pane_id: PaneId,
        axis: SplitAxis,
        placement: PanePlacement,
        ratio_bps: SplitRatioBps,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            if pane_id == destination_pane_id {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            let workspace = candidate.workspace_mut(workspace_id)?;
            let source_index = workspace
                .tabs
                .iter()
                .position(|tab| tab.id == source_tab_id)
                .ok_or(MultiplexerErrorKind::UnknownTab)?;
            let destination_index = workspace
                .tabs
                .iter()
                .position(|tab| tab.id == destination_tab_id)
                .ok_or(MultiplexerErrorKind::UnknownTab)?;

            if leaf_count(&workspace.tabs[source_index].root) <= 1 {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            if !contains_pane(&workspace.tabs[source_index].root, pane_id)
                || !contains_pane(&workspace.tabs[destination_index].root, destination_pane_id)
            {
                return Err(MultiplexerErrorKind::UnknownPane);
            }

            if source_index == destination_index {
                let root = workspace.tabs[source_index].root.clone();
                let (root, removed) = remove_pane(root, pane_id);
                if !removed {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }
                let mut root = root.ok_or(MultiplexerErrorKind::UnsupportedOperation)?;
                if !insert_existing_pane(
                    &mut root,
                    destination_pane_id,
                    pane_id,
                    axis,
                    placement,
                    ratio_bps,
                ) {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }
                let tab = &mut workspace.tabs[source_index];
                tab.root = root;
                if tab.zoomed_pane_id == Some(pane_id) {
                    tab.zoomed_pane_id = None;
                }
            } else {
                let source_root = workspace.tabs[source_index].root.clone();
                let (source_root, removed) = remove_pane(source_root, pane_id);
                if !removed {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }
                let source_root = source_root.ok_or(MultiplexerErrorKind::UnsupportedOperation)?;
                let mut destination_root = workspace.tabs[destination_index].root.clone();
                if !insert_existing_pane(
                    &mut destination_root,
                    destination_pane_id,
                    pane_id,
                    axis,
                    placement,
                    ratio_bps,
                ) {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }

                {
                    let source_tab = &mut workspace.tabs[source_index];
                    source_tab.root = source_root;
                    if source_tab.focused_pane_id == pane_id {
                        source_tab.focused_pane_id = first_pane(&source_tab.root)
                            .ok_or(MultiplexerErrorKind::UnknownPane)?;
                    }
                    if source_tab.zoomed_pane_id == Some(pane_id) {
                        source_tab.zoomed_pane_id = None;
                    }
                }
                workspace.tabs[destination_index].root = destination_root;
            }
            Ok(())
        })
    }

    pub(crate) fn resize_split_between(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        first_pane_id: PaneId,
        second_pane_id: PaneId,
        ratio_bps: SplitRatioBps,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            if first_pane_id == second_pane_id {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            let tab = tab_mut(candidate.workspace_mut(workspace_id)?, tab_id)?;
            if !contains_pane(&tab.root, first_pane_id) || !contains_pane(&tab.root, second_pane_id)
            {
                return Err(MultiplexerErrorKind::UnknownPane);
            }
            if !resize_lowest_separating_split(
                &mut tab.root,
                first_pane_id,
                second_pane_id,
                ratio_bps,
            ) {
                return Err(MultiplexerErrorKind::UnsupportedOperation);
            }
            Ok(())
        })
    }

    pub(crate) fn toggle_zoom(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            let tab = tab_mut(candidate.workspace_mut(workspace_id)?, tab_id)?;
            if !contains_pane(&tab.root, pane_id) {
                return Err(MultiplexerErrorKind::UnknownPane);
            }
            tab.zoomed_pane_id = if tab.zoomed_pane_id == Some(pane_id) {
                None
            } else {
                Some(pane_id)
            };
            Ok(())
        })
    }

    pub(crate) fn focus_pane(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            let workspace = candidate.workspace_mut(workspace_id)?;
            let tab = tab_mut(workspace, tab_id)?;
            if !contains_pane(&tab.root, pane_id) {
                return Err(MultiplexerErrorKind::UnknownPane);
            }
            tab.focused_pane_id = pane_id;
            workspace.focused_tab_id = tab_id;
            candidate.focused_workspace_id = Some(workspace_id);
            Ok(())
        })
    }

    pub(crate) fn close_pane(
        &mut self,
        expected: TopologyGeneration,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        self.transact(expected, |candidate| {
            {
                let workspace = candidate.workspace_mut(workspace_id)?;
                let tab = tab_mut(workspace, tab_id)?;
                if leaf_count(&tab.root) <= 1 {
                    return Err(MultiplexerErrorKind::UnsupportedOperation);
                }
                let root = tab.root.clone();
                let (root, removed) = remove_pane(root, pane_id);
                if !removed {
                    return Err(MultiplexerErrorKind::UnknownPane);
                }
                tab.root = root.ok_or(MultiplexerErrorKind::UnsupportedOperation)?;
                if tab.focused_pane_id == pane_id {
                    tab.focused_pane_id =
                        first_pane(&tab.root).ok_or(MultiplexerErrorKind::UnknownPane)?;
                }
                if tab.zoomed_pane_id == Some(pane_id) {
                    tab.zoomed_pane_id = None;
                }
            }
            candidate.retired_pane_ids.insert(pane_id);
            Ok(())
        })
    }

    pub(crate) fn directional_neighbor(
        &self,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        pane_id: PaneId,
        direction: Direction,
    ) -> Result<Option<PaneId>, MultiplexerErrorKind> {
        let tab = self.tab(workspace_id, tab_id)?;
        let rects = pane_rects(&tab.root);
        let source = rects
            .iter()
            .find(|(candidate, _)| *candidate == pane_id)
            .ok_or(MultiplexerErrorKind::UnknownPane)?;
        Ok(select_directional_neighbor(source, &rects, direction))
    }

    pub(crate) fn edge_pane(
        &self,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
        edge: Edge,
    ) -> Result<PaneId, MultiplexerErrorKind> {
        let tab = self.tab(workspace_id, tab_id)?;
        select_edge(&pane_rects(&tab.root), edge).ok_or(MultiplexerErrorKind::UnknownPane)
    }

    fn transact(
        &mut self,
        expected: TopologyGeneration,
        mutation: impl FnOnce(&mut Self) -> Result<(), MultiplexerErrorKind>,
    ) -> Result<TopologyGeneration, MultiplexerErrorKind> {
        if self.generation != expected {
            return Err(MultiplexerErrorKind::StaleTopologyGeneration);
        }
        let next = self.generation.checked_next()?;
        let mut candidate = self.clone();
        mutation(&mut candidate)?;
        candidate.generation = next;
        *self = candidate;
        Ok(next)
    }

    fn workspace_mut(
        &mut self,
        workspace_id: MultiplexerWorkspaceId,
    ) -> Result<&mut WorkspaceState, MultiplexerErrorKind> {
        self.workspaces
            .iter_mut()
            .find(|workspace| workspace.id == workspace_id)
            .ok_or(MultiplexerErrorKind::UnknownWorkspace)
    }

    fn require_fresh_tab_id(
        &self,
        workspace_id: MultiplexerWorkspaceId,
        tab_id: TabId,
    ) -> Result<(), MultiplexerErrorKind> {
        if self.retired_tab_ids.contains(&(workspace_id, tab_id))
            || self
                .workspaces
                .iter()
                .find(|workspace| workspace.id == workspace_id)
                .is_some_and(|workspace| workspace.tabs.iter().any(|tab| tab.id == tab_id))
        {
            return Err(MultiplexerErrorKind::IdentityReuse);
        }
        Ok(())
    }

    fn require_fresh_pane_id(&self, pane_id: PaneId) -> Result<(), MultiplexerErrorKind> {
        if self.retired_pane_ids.contains(&pane_id)
            || self
                .workspaces
                .iter()
                .flat_map(|workspace| &workspace.tabs)
                .any(|tab| contains_pane(&tab.root, pane_id))
        {
            return Err(MultiplexerErrorKind::IdentityReuse);
        }
        Ok(())
    }
}

fn require_tab(
    workspace: &WorkspaceState,
    tab_id: TabId,
) -> Result<&TabState, MultiplexerErrorKind> {
    workspace
        .tabs
        .iter()
        .find(|tab| tab.id == tab_id)
        .ok_or(MultiplexerErrorKind::UnknownTab)
}

fn tab_mut(
    workspace: &mut WorkspaceState,
    tab_id: TabId,
) -> Result<&mut TabState, MultiplexerErrorKind> {
    workspace
        .tabs
        .iter_mut()
        .find(|tab| tab.id == tab_id)
        .ok_or(MultiplexerErrorKind::UnknownTab)
}

fn contains_pane(node: &LayoutNode, pane_id: PaneId) -> bool {
    match node {
        LayoutNode::Pane(candidate) => *candidate == pane_id,
        LayoutNode::Split { first, second, .. } => {
            contains_pane(first, pane_id) || contains_pane(second, pane_id)
        }
    }
}

fn first_pane(node: &LayoutNode) -> Option<PaneId> {
    match node {
        LayoutNode::Pane(pane_id) => Some(*pane_id),
        LayoutNode::Split { first, .. } => first_pane(first),
    }
}

fn leaf_count(node: &LayoutNode) -> usize {
    match node {
        LayoutNode::Pane(_) => 1,
        LayoutNode::Split { first, second, .. } => leaf_count(first) + leaf_count(second),
    }
}

fn collect_pane_ids(node: &LayoutNode, output: &mut BTreeSet<PaneId>) {
    match node {
        LayoutNode::Pane(pane_id) => {
            output.insert(*pane_id);
        }
        LayoutNode::Split { first, second, .. } => {
            collect_pane_ids(first, output);
            collect_pane_ids(second, output);
        }
    }
}

fn replace_leaf_with_split(
    node: &mut LayoutNode,
    target: PaneId,
    new_pane: PaneId,
    axis: SplitAxis,
    placement: PanePlacement,
    ratio_bps: SplitRatioBps,
) -> bool {
    match node {
        LayoutNode::Pane(pane_id) if *pane_id == target => {
            let existing = LayoutNode::Pane(target);
            let new = LayoutNode::Pane(new_pane);
            let (first, second) = match placement {
                PanePlacement::Before => (new, existing),
                PanePlacement::After => (existing, new),
            };
            *node = LayoutNode::Split {
                axis,
                ratio_bps,
                first: Box::new(first),
                second: Box::new(second),
            };
            true
        }
        LayoutNode::Pane(_) => false,
        LayoutNode::Split { first, second, .. } => {
            replace_leaf_with_split(first, target, new_pane, axis, placement, ratio_bps)
                || replace_leaf_with_split(second, target, new_pane, axis, placement, ratio_bps)
        }
    }
}

fn insert_existing_pane(
    node: &mut LayoutNode,
    target: PaneId,
    moving: PaneId,
    axis: SplitAxis,
    placement: PanePlacement,
    ratio_bps: SplitRatioBps,
) -> bool {
    replace_leaf_with_split(node, target, moving, axis, placement, ratio_bps)
}

fn swap_leaf_ids(node: &mut LayoutNode, first: PaneId, second: PaneId) {
    match node {
        LayoutNode::Pane(pane_id) => {
            if *pane_id == first {
                *pane_id = second;
            } else if *pane_id == second {
                *pane_id = first;
            }
        }
        LayoutNode::Split {
            first: left,
            second: right,
            ..
        } => {
            swap_leaf_ids(left, first, second);
            swap_leaf_ids(right, first, second);
        }
    }
}

fn remove_pane(node: LayoutNode, target: PaneId) -> (Option<LayoutNode>, bool) {
    match node {
        LayoutNode::Pane(pane_id) if pane_id == target => (None, true),
        LayoutNode::Pane(pane_id) => (Some(LayoutNode::Pane(pane_id)), false),
        LayoutNode::Split {
            axis,
            ratio_bps,
            first,
            second,
        } => {
            let first_node = *first;
            let second_node = *second;
            let (first_result, removed_first) = remove_pane(first_node, target);
            if removed_first {
                return match first_result {
                    Some(first_result) => (
                        Some(LayoutNode::Split {
                            axis,
                            ratio_bps,
                            first: Box::new(first_result),
                            second: Box::new(second_node),
                        }),
                        true,
                    ),
                    None => (Some(second_node), true),
                };
            }
            let (second_result, removed_second) = remove_pane(second_node, target);
            if removed_second {
                return match second_result {
                    Some(second_result) => (
                        Some(LayoutNode::Split {
                            axis,
                            ratio_bps,
                            first: Box::new(first_result.expect("unchanged first subtree")),
                            second: Box::new(second_result),
                        }),
                        true,
                    ),
                    None => (first_result, true),
                };
            }
            (
                Some(LayoutNode::Split {
                    axis,
                    ratio_bps,
                    first: Box::new(first_result.expect("unchanged first subtree")),
                    second: Box::new(second_result.expect("unchanged second subtree")),
                }),
                false,
            )
        }
    }
}

fn resize_lowest_separating_split(
    node: &mut LayoutNode,
    first_pane: PaneId,
    second_pane: PaneId,
    ratio_bps: SplitRatioBps,
) -> bool {
    let LayoutNode::Split {
        ratio_bps: current,
        first,
        second,
        ..
    } = node
    else {
        return false;
    };

    let first_in_left = contains_pane(first, first_pane);
    let second_in_left = contains_pane(first, second_pane);
    let first_in_right = contains_pane(second, first_pane);
    let second_in_right = contains_pane(second, second_pane);

    if (first_in_left && second_in_right) || (second_in_left && first_in_right) {
        *current = ratio_bps;
        return true;
    }
    if first_in_left && second_in_left {
        return resize_lowest_separating_split(first, first_pane, second_pane, ratio_bps);
    }
    if first_in_right && second_in_right {
        return resize_lowest_separating_split(second, first_pane, second_pane, ratio_bps);
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PaneRect {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
}

impl PaneRect {
    fn center_x(self) -> i64 {
        (i64::from(self.left) + i64::from(self.right)) / 2
    }

    fn center_y(self) -> i64 {
        (i64::from(self.top) + i64::from(self.bottom)) / 2
    }
}

fn pane_rects(root: &LayoutNode) -> Vec<(PaneId, PaneRect)> {
    let mut output = Vec::new();
    collect_rects(
        root,
        PaneRect {
            left: 0,
            top: 0,
            right: GEOMETRY_EXTENT,
            bottom: GEOMETRY_EXTENT,
        },
        &mut output,
    );
    output
}

fn collect_rects(node: &LayoutNode, rect: PaneRect, output: &mut Vec<(PaneId, PaneRect)>) {
    match node {
        LayoutNode::Pane(pane_id) => output.push((*pane_id, rect)),
        LayoutNode::Split {
            axis,
            ratio_bps,
            first,
            second,
        } => {
            let ratio = u32::from(ratio_bps.get());
            match axis {
                SplitAxis::Horizontal => {
                    let width = rect.right - rect.left;
                    let cut_offset = (u64::from(width) * u64::from(ratio) / 10_000) as u32;
                    let cut = rect.left + cut_offset;
                    collect_rects(
                        first,
                        PaneRect {
                            left: rect.left,
                            top: rect.top,
                            right: cut,
                            bottom: rect.bottom,
                        },
                        output,
                    );
                    collect_rects(
                        second,
                        PaneRect {
                            left: cut,
                            top: rect.top,
                            right: rect.right,
                            bottom: rect.bottom,
                        },
                        output,
                    );
                }
                SplitAxis::Vertical => {
                    let height = rect.bottom - rect.top;
                    let cut_offset = (u64::from(height) * u64::from(ratio) / 10_000) as u32;
                    let cut = rect.top + cut_offset;
                    collect_rects(
                        first,
                        PaneRect {
                            left: rect.left,
                            top: rect.top,
                            right: rect.right,
                            bottom: cut,
                        },
                        output,
                    );
                    collect_rects(
                        second,
                        PaneRect {
                            left: rect.left,
                            top: cut,
                            right: rect.right,
                            bottom: rect.bottom,
                        },
                        output,
                    );
                }
            }
        }
    }
}

fn select_directional_neighbor(
    source: &(PaneId, PaneRect),
    rects: &[(PaneId, PaneRect)],
    direction: Direction,
) -> Option<PaneId> {
    let (_, source_rect) = *source;
    rects
        .iter()
        .filter(|(pane_id, _)| *pane_id != source.0)
        .filter_map(|(pane_id, rect)| {
            directional_score(source_rect, *rect, direction).map(|score| (score, *pane_id))
        })
        .min_by_key(|(score, pane_id)| (*score, *pane_id))
        .map(|(_, pane_id)| pane_id)
}

fn directional_score(
    source: PaneRect,
    candidate: PaneRect,
    direction: Direction,
) -> Option<(i64, u8, i64)> {
    let (primary, overlap, orthogonal_gap) = match direction {
        Direction::Left if candidate.center_x() < source.center_x() => (
            source.center_x() - candidate.center_x(),
            intervals_overlap(source.top, source.bottom, candidate.top, candidate.bottom),
            interval_gap(source.top, source.bottom, candidate.top, candidate.bottom),
        ),
        Direction::Right if candidate.center_x() > source.center_x() => (
            candidate.center_x() - source.center_x(),
            intervals_overlap(source.top, source.bottom, candidate.top, candidate.bottom),
            interval_gap(source.top, source.bottom, candidate.top, candidate.bottom),
        ),
        Direction::Up if candidate.center_y() < source.center_y() => (
            source.center_y() - candidate.center_y(),
            intervals_overlap(source.left, source.right, candidate.left, candidate.right),
            interval_gap(source.left, source.right, candidate.left, candidate.right),
        ),
        Direction::Down if candidate.center_y() > source.center_y() => (
            candidate.center_y() - source.center_y(),
            intervals_overlap(source.left, source.right, candidate.left, candidate.right),
            interval_gap(source.left, source.right, candidate.left, candidate.right),
        ),
        _ => return None,
    };
    Some((primary, u8::from(!overlap), orthogonal_gap))
}

fn intervals_overlap(first_start: u32, first_end: u32, second_start: u32, second_end: u32) -> bool {
    first_start < second_end && second_start < first_end
}

fn interval_gap(first_start: u32, first_end: u32, second_start: u32, second_end: u32) -> i64 {
    if intervals_overlap(first_start, first_end, second_start, second_end) {
        0
    } else if first_end <= second_start {
        i64::from(second_start - first_end)
    } else {
        i64::from(first_start - second_end)
    }
}

fn select_edge(rects: &[(PaneId, PaneRect)], edge: Edge) -> Option<PaneId> {
    rects
        .iter()
        .min_by_key(|(pane_id, rect)| match edge {
            Edge::Left => (i64::from(rect.left), i64::from(rect.top), *pane_id),
            Edge::Right => (-i64::from(rect.right), i64::from(rect.top), *pane_id),
            Edge::Top => (i64::from(rect.top), i64::from(rect.left), *pane_id),
            Edge::Bottom => (-i64::from(rect.bottom), i64::from(rect.left), *pane_id),
        })
        .map(|(pane_id, _)| *pane_id)
}

#[cfg(test)]
#[path = "../t163_multiplexer_topology_tests.rs"]
mod t163_multiplexer_topology_tests;
