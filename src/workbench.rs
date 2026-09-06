use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::widgets::{Block, Paragraph};

const EMPTY_WORKBENCH_MESSAGE: &str = "No terminal panes are active.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct PaneId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneLifecycleView {
    Live,
    Exited,
    Stopped,
    OwnershipLost,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaneSize {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
}

impl PaneSize {
    pub(crate) const fn new(columns: u16, rows: u16) -> Self {
        Self { columns, rows }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PanePresentationMetadata {
    pub(crate) display_title: String,
    pub(crate) canonical_workspace_id: Option<String>,
    pub(crate) canonical_winds_session_id: Option<String>,
    pub(crate) size: PaneSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneState {
    pub(crate) pane_id: PaneId,
    pub(crate) display_title: String,
    pub(crate) canonical_workspace_id: Option<String>,
    pub(crate) canonical_winds_session_id: Option<String>,
    pub(crate) lifecycle: PaneLifecycleView,
    pub(crate) size: PaneSize,
    pub(crate) split_from: Option<PaneId>,
    pub(crate) split_axis: Option<SplitAxis>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct WorkbenchState {
    panes: Vec<PaneState>,
    selected_pane: Option<PaneId>,
    next_pane_id: u64,
}

impl WorkbenchState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn panes(&self) -> &[PaneState] {
        &self.panes
    }

    pub(crate) fn pane(&self, pane_id: PaneId) -> Option<&PaneState> {
        self.panes.iter().find(|pane| pane.pane_id == pane_id)
    }

    pub(crate) fn selected_pane(&self) -> Option<PaneId> {
        self.selected_pane
    }

    /// Return only the presentation-selected pane. Later lifecycle integration must
    /// separately resolve this transient identifier to an accepted owned terminal.
    pub(crate) fn selected_dispatch_candidate(&self) -> Option<PaneId> {
        self.selected_pane
            .filter(|selected| self.panes.iter().any(|pane| pane.pane_id == *selected))
    }

    pub(crate) fn create_pane(
        &mut self,
        display_title: impl Into<String>,
        canonical_workspace_id: Option<String>,
        canonical_winds_session_id: Option<String>,
        size: PaneSize,
    ) -> PaneId {
        let pane_id = self.allocate_pane_id();
        self.panes.push(PaneState {
            pane_id,
            display_title: display_title.into(),
            canonical_workspace_id,
            canonical_winds_session_id,
            lifecycle: PaneLifecycleView::Stopped,
            size,
            split_from: None,
            split_axis: None,
        });
        if self.selected_pane.is_none() {
            self.selected_pane = Some(pane_id);
        }
        pane_id
    }

    pub(crate) fn restore_presentation(&mut self, metadata: PanePresentationMetadata) -> PaneId {
        let pane_id = self.allocate_pane_id();
        self.panes.push(PaneState {
            pane_id,
            display_title: metadata.display_title,
            canonical_workspace_id: metadata.canonical_workspace_id,
            canonical_winds_session_id: metadata.canonical_winds_session_id,
            lifecycle: PaneLifecycleView::OwnershipLost,
            size: metadata.size,
            split_from: None,
            split_axis: None,
        });
        if self.selected_pane.is_none() {
            self.selected_pane = Some(pane_id);
        }
        pane_id
    }

    pub(crate) fn split_pane(
        &mut self,
        target: PaneId,
        axis: SplitAxis,
        display_title: impl Into<String>,
    ) -> Option<PaneId> {
        let target_index = self.pane_index(target)?;
        let target_state = self.panes[target_index].clone();
        let pane_id = self.allocate_pane_id();
        self.panes.insert(
            target_index + 1,
            PaneState {
                pane_id,
                display_title: display_title.into(),
                canonical_workspace_id: target_state.canonical_workspace_id,
                canonical_winds_session_id: target_state.canonical_winds_session_id,
                lifecycle: PaneLifecycleView::Stopped,
                size: target_state.size,
                split_from: Some(target),
                split_axis: Some(axis),
            },
        );
        self.selected_pane = Some(pane_id);
        Some(pane_id)
    }

    pub(crate) fn focus_pane(&mut self, pane_id: PaneId) -> bool {
        if self.pane_index(pane_id).is_none() {
            return false;
        }
        self.selected_pane = Some(pane_id);
        true
    }

    pub(crate) fn resize_pane(&mut self, pane_id: PaneId, size: PaneSize) -> bool {
        let Some(index) = self.pane_index(pane_id) else {
            return false;
        };
        self.panes[index].size = size;
        true
    }

    pub(crate) fn rename_pane(
        &mut self,
        pane_id: PaneId,
        display_title: impl Into<String>,
    ) -> bool {
        let Some(index) = self.pane_index(pane_id) else {
            return false;
        };
        self.panes[index].display_title = display_title.into();
        true
    }

    pub(crate) fn move_pane_to_index(&mut self, pane_id: PaneId, requested_index: usize) -> bool {
        let Some(index) = self.pane_index(pane_id) else {
            return false;
        };
        let pane = self.panes.remove(index);
        let destination = requested_index.min(self.panes.len());
        self.panes.insert(destination, pane);
        true
    }

    pub(crate) fn close_pane(&mut self, pane_id: PaneId) -> bool {
        let Some(index) = self.pane_index(pane_id) else {
            return false;
        };
        let removed_parent = self.panes[index].split_from;
        self.panes.remove(index);

        for pane in &mut self.panes {
            if pane.split_from == Some(pane_id) {
                pane.split_from = removed_parent;
                if pane.split_from.is_none() {
                    pane.split_axis = None;
                }
            }
        }

        if self.selected_pane == Some(pane_id) {
            self.selected_pane = if self.panes.is_empty() {
                None
            } else {
                Some(self.panes[index.min(self.panes.len() - 1)].pane_id)
            };
        }
        true
    }

    fn allocate_pane_id(&mut self) -> PaneId {
        let pane_id = PaneId(self.next_pane_id);
        self.next_pane_id = self
            .next_pane_id
            .checked_add(1)
            .expect("transient PaneId space exhausted");
        pane_id
    }

    fn pane_index(&self, pane_id: PaneId) -> Option<usize> {
        self.panes.iter().position(|pane| pane.pane_id == pane_id)
    }
}

/// Render the inert T087 workbench shell without owning terminal runtime state.
pub(crate) fn render_inert_workbench(frame: &mut Frame<'_>) {
    let area = frame.area();
    let block = Block::bordered().title(" Winds Workbench ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width > 0 && inner.height > 0 {
        frame.render_widget(
            Paragraph::new(EMPTY_WORKBENCH_MESSAGE).alignment(Alignment::Center),
            inner,
        );
    }
}

#[path = "workbench_screen.rs"]
pub(crate) mod screen;

#[cfg(test)]
#[path = "t088_workbench_topology_tests.rs"]
mod t088_workbench_topology_tests;

