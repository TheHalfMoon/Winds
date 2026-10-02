#![allow(
    dead_code,
    reason = "Spec 007 workbench includes accepted presentation and safety seams not all reached by the initial production entry"
)]

use crate::git::Repo;
use crate::git::shell_profiles::discover_native_shell_profiles;
use crate::git::workspace::open_existing_workspace;
use crate::git::workspace_inventory::inventory_workspace_environment;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event,
    KeyCode, KeyEventKind, KeyModifiers,
};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    size as host_terminal_size,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::widgets::{Block, Paragraph};
use ratatui::{Frame, Terminal};
use serde_json::json;
use std::io::{self, Write};
use std::path::PathBuf;

const EMPTY_WORKBENCH_MESSAGE: &str = "No terminal panes are active.";
const COMPACT_WORKBENCH_MIN_WIDTH: u16 = 60;
const COMPACT_WORKBENCH_MIN_HEIGHT: u16 = 10;

const T174_TUI_AGENT_DOCK_MAX_ROWS: usize = 8;
const T174_TUI_AGENT_DOCK_NONCLAIM: &str = "DETECTION_ONLY_UNPROVEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct T174TuiAgentHitRegion {
    observation_id: crate::multiplexer::domain::AgentObservationId,
    column: u16,
    row: u16,
    width: u16,
    height: u16,
}

impl T174TuiAgentHitRegion {
    fn contains(self, column: u16, row: u16) -> bool {
        column >= self.column
            && row >= self.row
            && column < self.column.saturating_add(self.width)
            && row < self.row.saturating_add(self.height)
    }
}

#[derive(Debug, Default)]
struct T174TuiAgentDockState {
    open: bool,
    selected_observation_id: Option<crate::multiplexer::domain::AgentObservationId>,
    alias_input: Option<String>,
    hit_regions: Vec<T174TuiAgentHitRegion>,
    status: String,
}

impl T174TuiAgentDockState {
    fn open_with(&mut self, dock: &mut ui::CanonicalAgentDockPresentation) {
        self.open = true;
        self.alias_input = None;
        self.hit_regions.clear();
        self.selected_observation_id = dock
            .list("", ui::CanonicalAgentDockSort::Family, true)
            .first()
            .map(|row| row.observation_id);
        if let Some(observation_id) = self.selected_observation_id {
            let _ = dock.view(observation_id);
        }
        self.status = "CURRENT owner-authoritative observations loaded".to_owned();
    }

    fn open_unavailable(&mut self, reason: impl Into<String>) {
        self.open = true;
        self.selected_observation_id = None;
        self.alias_input = None;
        self.hit_regions.clear();
        self.status = format!("UNAVAILABLE · {}", reason.into());
    }

    fn close(&mut self) {
        self.open = false;
        self.alias_input = None;
        self.hit_regions.clear();
    }

    fn select_exact(
        &mut self,
        dock: &mut ui::CanonicalAgentDockPresentation,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) {
        if dock.view(observation_id).is_some() {
            self.selected_observation_id = Some(observation_id);
            self.status = format!("VIEW observation={observation_id}");
        }
    }

    fn cycle_selection(&mut self, dock: &mut ui::CanonicalAgentDockPresentation, delta: isize) {
        let rows = dock.list("", ui::CanonicalAgentDockSort::Family, true);
        if rows.is_empty() {
            self.selected_observation_id = None;
            return;
        }
        let current = self
            .selected_observation_id
            .and_then(|selected| rows.iter().position(|row| row.observation_id == selected))
            .unwrap_or(0);
        let next = if delta < 0 {
            current.checked_sub(1).unwrap_or(rows.len() - 1)
        } else {
            (current + 1) % rows.len()
        };
        self.select_exact(dock, rows[next].observation_id);
    }

    fn focus_selected(
        &mut self,
        dock: &mut ui::CanonicalAgentDockPresentation,
        topology: Option<&ui::CanonicalTopologyPresentation>,
    ) {
        let Some(observation_id) = self.selected_observation_id else {
            self.status = "FOCUS_UNAVAILABLE · no exact observation selected".to_owned();
            return;
        };
        let Some(topology) = topology else {
            self.status = "FOCUS_UNAVAILABLE · canonical topology unavailable".to_owned();
            return;
        };
        match dock.focus(observation_id, topology) {
            Some(target) => {
                self.status = format!(
                    "FOCUS_VALIDATED observation={} workspace={} tab={} pane={} topology_generation={:?} · READ_ONLY",
                    target.observation_id,
                    target.multiplexer_workspace_id,
                    target.tab_id,
                    target.pane_id,
                    target.topology_generation,
                );
            }
            None => {
                self.status =
                    "FOCUS_UNAVAILABLE · stale, absent, ambiguous, or substituted exact target"
                        .to_owned();
            }
        }
    }

    fn focus_target_for_event(
        &self,
        event: &Event,
    ) -> Option<crate::multiplexer::domain::AgentObservationId> {
        if !self.open || self.alias_input.is_some() {
            return None;
        }
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release && key.code == KeyCode::Enter => {
                self.selected_observation_id
            }
            Event::Mouse(mouse)
                if mouse.kind
                    == crossterm::event::MouseEventKind::Down(
                        crossterm::event::MouseButton::Right,
                    ) =>
            {
                self.hit_regions
                    .iter()
                    .copied()
                    .find(|region| region.contains(mouse.column, mouse.row))
                    .map(|region| region.observation_id)
            }
            _ => None,
        }
    }

    fn handle_event(
        &mut self,
        mut dock: Option<&mut ui::CanonicalAgentDockPresentation>,
        topology: Option<&ui::CanonicalTopologyPresentation>,
        event: &Event,
    ) -> bool {
        if !self.open {
            return false;
        }
        if matches!(event, Event::Key(key) if key.kind == KeyEventKind::Release) {
            return true;
        }
        if matches!(event, Event::Key(key) if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q'))
        {
            return false;
        }

        if self.alias_input.is_some() {
            let Event::Key(key) = event else {
                return true;
            };
            match key.code {
                KeyCode::Esc => {
                    self.alias_input = None;
                    self.status = "ALIAS_CANCELLED · immutable identity unchanged".to_owned();
                }
                KeyCode::Backspace => {
                    if let Some(alias) = &mut self.alias_input {
                        alias.pop();
                    }
                }
                KeyCode::Enter => {
                    let alias = self.alias_input.take().unwrap_or_default();
                    let Some(observation_id) = self.selected_observation_id else {
                        self.status =
                            "ALIAS_UNAVAILABLE · no exact observation selected".to_owned();
                        return true;
                    };
                    let Some(dock) = dock.as_deref_mut() else {
                        self.status =
                            "ALIAS_UNAVAILABLE · observation truth unavailable".to_owned();
                        return true;
                    };
                    match dock.rename(observation_id, Some(&alias)) {
                        Ok(()) => {
                            self.status = format!(
                                "ALIAS_UPDATED observation={observation_id} · DISPLAY_ONLY · immutable identity unchanged"
                            );
                        }
                        Err(error) => self.status = format!("ALIAS_REFUSED · {error}"),
                    }
                }
                KeyCode::Char(character)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT) =>
                {
                    if let Some(alias) = &mut self.alias_input {
                        alias.push(character);
                    }
                }
                _ => {}
            }
            return true;
        }

        match event {
            Event::Key(key) if key.code == KeyCode::Esc => {
                self.close();
                true
            }
            Event::Key(key) if key.code == KeyCode::Up => {
                if let Some(dock) = dock.as_deref_mut() {
                    self.cycle_selection(dock, -1);
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Down => {
                if let Some(dock) = dock.as_deref_mut() {
                    self.cycle_selection(dock, 1);
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                if let Some(dock) = dock.as_deref_mut() {
                    self.focus_selected(dock, topology);
                } else {
                    self.status = "FOCUS_UNAVAILABLE · observation truth unavailable".to_owned();
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Char('e') => {
                if let (Some(observation_id), Some(dock)) =
                    (self.selected_observation_id, dock.as_deref_mut())
                {
                    self.status = dock
                        .explain(observation_id)
                        .map(|explanation| format!("EXPLAIN\n{explanation}"))
                        .unwrap_or_else(|| "EXPLAIN_UNAVAILABLE".to_owned());
                }
                true
            }
            Event::Key(key) if key.code == KeyCode::Char('r') => {
                if let (Some(observation_id), Some(dock)) =
                    (self.selected_observation_id, dock.as_deref_mut())
                {
                    self.alias_input = Some(
                        dock.get(observation_id)
                            .and_then(|item| item.display_alias.clone())
                            .unwrap_or_default(),
                    );
                    self.status = format!(
                        "ALIAS_EDIT observation={observation_id} · DISPLAY_ONLY · Enter saves · Esc cancels"
                    );
                }
                true
            }
            Event::Mouse(mouse)
                if matches!(
                    mouse.kind,
                    crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
                        | crossterm::event::MouseEventKind::Down(
                            crossterm::event::MouseButton::Right
                        )
                ) =>
            {
                let target = self
                    .hit_regions
                    .iter()
                    .copied()
                    .find(|region| region.contains(mouse.column, mouse.row))
                    .map(|region| region.observation_id);
                let Some(observation_id) = target else {
                    return false;
                };
                let Some(dock) = dock else {
                    self.status = "VIEW_UNAVAILABLE · observation truth unavailable".to_owned();
                    return true;
                };
                self.select_exact(dock, observation_id);
                if mouse.kind
                    == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right)
                {
                    self.focus_selected(dock, topology);
                }
                true
            }
            _ => true,
        }
    }
}

fn t174_agent_dock_toggle_event(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(key)
            if key.kind != KeyEventKind::Release
                && key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('g')
    )
}

fn load_t174_agent_dock(
    store: &crate::store::Store,
) -> Result<ui::CanonicalAgentDockPresentation, String> {
    let owner_generation_id = store
        .latest_persistent_runtime_owner_generation()
        .map_err(|error| format!("owner generation record unavailable: {error}"))?
        .ok_or_else(|| "owner generation unavailable".to_owned())?;
    let mut client = crate::persistent_runtime::client::RustLocalControlClient::connect(
        None,
        Some(owner_generation_id),
    )
    .map_err(|error| format!("owner connection unavailable: {error}"))?;
    let observations = client
        .refresh_agent_observation_projection(None)
        .map_err(|error| format!("agent observation refresh unavailable: {error}"))?;
    ui::CanonicalAgentDockPresentation::from_observations(observations)
}

fn preserve_t174_display_aliases(
    previous: &ui::CanonicalAgentDockPresentation,
    refreshed: &mut ui::CanonicalAgentDockPresentation,
) -> Result<(), String> {
    for row in previous.list("", ui::CanonicalAgentDockSort::Family, false) {
        let Some(alias) = previous
            .get(row.observation_id)
            .and_then(|item| item.display_alias.as_deref())
        else {
            continue;
        };
        if refreshed.get(row.observation_id).is_some() {
            refreshed.rename(row.observation_id, Some(alias))?;
        }
    }
    Ok(())
}

fn load_t174_agent_focus_context(
    store: &crate::store::Store,
    observation_id: crate::multiplexer::domain::AgentObservationId,
) -> Result<
    (
        ui::CanonicalAgentDockPresentation,
        ui::CanonicalTopologyPresentation,
    ),
    String,
> {
    let owner_generation_id = store
        .latest_persistent_runtime_owner_generation()
        .map_err(|error| format!("owner generation record unavailable: {error}"))?
        .ok_or_else(|| "owner generation unavailable".to_owned())?;
    let mut client = crate::persistent_runtime::client::RustLocalControlClient::connect(
        None,
        Some(owner_generation_id),
    )
    .map_err(|error| format!("owner connection unavailable: {error}"))?;

    let first_observations = client
        .refresh_agent_observation_projection(None)
        .map_err(|error| format!("agent observation refresh unavailable: {error}"))?;
    let first = first_observations
        .iter()
        .find(|observation| observation.observation_id == observation_id)
        .ok_or_else(|| "focus observation is absent from current owner truth".to_owned())?;
    if first.freshness != crate::persistent_runtime::protocol::AgentObservationFreshnessV2::Current
    {
        return Err(format!(
            "focus observation is not current: {:?}",
            first.freshness
        ));
    }
    if first.owner_generation_id != owner_generation_id {
        return Err("focus observation owner generation does not match current owner".to_owned());
    }
    let first_workspace_id = first.multiplexer_workspace_id;
    let first_tab_id = first.tab_id;
    let first_pane_id = first.pane_id;
    let first_runtime_namespace_id = first.runtime_namespace_id;

    client
        .refresh_topology_projection(Some(first_workspace_id))
        .map_err(|error| format!("focus topology refresh unavailable: {error}"))?;
    let first_topology =
        ui::CanonicalTopologyPresentation::from_client(&client, false, false, false)
            .ok_or_else(|| "focus topology presentation unavailable".to_owned())?;
    let first_topology_generation = first_topology.topology_generation();

    let trailing_observations = client
        .refresh_agent_observation_projection(None)
        .map_err(|error| format!("trailing agent observation refresh unavailable: {error}"))?;
    let trailing = trailing_observations
        .iter()
        .find(|observation| observation.observation_id == observation_id)
        .ok_or_else(|| "focus observation disappeared during validation".to_owned())?;
    if trailing.freshness
        != crate::persistent_runtime::protocol::AgentObservationFreshnessV2::Current
        || trailing.owner_generation_id != owner_generation_id
        || trailing.multiplexer_workspace_id != first_workspace_id
        || trailing.tab_id != first_tab_id
        || trailing.pane_id != first_pane_id
        || trailing.runtime_namespace_id != first_runtime_namespace_id
    {
        return Err("focus observation changed during validation".to_owned());
    }

    client
        .refresh_topology_projection(Some(first_workspace_id))
        .map_err(|error| format!("trailing focus topology refresh unavailable: {error}"))?;
    let topology = ui::CanonicalTopologyPresentation::from_client(&client, false, false, false)
        .ok_or_else(|| "trailing focus topology presentation unavailable".to_owned())?;
    if topology.topology_generation() != first_topology_generation {
        return Err("focus topology generation changed during validation".to_owned());
    }

    let mut dock = ui::CanonicalAgentDockPresentation::from_observations(trailing_observations)?;
    dock.focus(observation_id, &topology)
        .ok_or_else(|| "focus target is stale, absent, ambiguous, or substituted".to_owned())?;

    let final_owner_generation_id = store
        .latest_persistent_runtime_owner_generation()
        .map_err(|error| format!("final owner generation record unavailable: {error}"))?
        .ok_or_else(|| "owner generation unavailable after focus validation".to_owned())?;
    if final_owner_generation_id != owner_generation_id {
        return Err("focus owner generation changed during validation".to_owned());
    }
    Ok((dock, topology))
}

fn t174_agent_dock_text(
    dock: Option<&ui::CanonicalAgentDockPresentation>,
    state: &T174TuiAgentDockState,
) -> String {
    let mut lines = vec![
        "T174_AGENT_DOCK=OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS".to_owned(),
        format!("execution={T174_TUI_AGENT_DOCK_NONCLAIM}"),
        format!("status={}", state.status),
        "keys=Up/Down view | Enter focus-validate | e explain | r rename-display-alias | Esc close | right-click focus-validate".to_owned(),
    ];
    if let Some(alias) = &state.alias_input {
        lines.push(format!("alias_input={alias}"));
    }
    match dock {
        Some(dock) => {
            for row in dock
                .list("", ui::CanonicalAgentDockSort::Family, true)
                .into_iter()
                .take(T174_TUI_AGENT_DOCK_MAX_ROWS)
            {
                let selected = if state.selected_observation_id == Some(row.observation_id) {
                    ">"
                } else {
                    " "
                };
                lines.push(format!(
                    "{selected} observation={} family={:?} freshness={:?}",
                    row.observation_id, row.family, row.freshness
                ));
            }
            if let Some(observation_id) = state.selected_observation_id
                && let Some(explanation) = dock.explain(observation_id)
            {
                lines.push("SELECTED_EXACT_BINDING".to_owned());
                lines.extend(explanation.lines().map(str::to_owned));
            }
        }
        None => lines.push(
            "UNAVAILABLE · no observation, provider, process, controller, write, Git, or verification authority inferred"
                .to_owned(),
        ),
    }
    lines.join("\n")
}

fn render_t174_agent_dock(
    frame: &mut Frame<'_>,
    area: ratatui::layout::Rect,
    dock: Option<&ui::CanonicalAgentDockPresentation>,
    state: &mut T174TuiAgentDockState,
) {
    state.hit_regions.clear();
    if !state.open || area.width < 24 || area.height < 7 {
        return;
    }
    let width = area.width.min(96);
    let height = area.height.min(20);
    let overlay = ratatui::layout::Rect::new(
        area.x.saturating_add(area.width.saturating_sub(width)),
        area.y,
        width,
        height,
    );
    frame.render_widget(
        Paragraph::new(t174_agent_dock_text(dock, state))
            .block(Block::bordered().title(" Agents · READ_ONLY · Ctrl+G ")),
        overlay,
    );
    let Some(dock) = dock else {
        return;
    };
    let first_row = overlay.y.saturating_add(5);
    for (index, row) in dock
        .list("", ui::CanonicalAgentDockSort::Family, true)
        .into_iter()
        .take(T174_TUI_AGENT_DOCK_MAX_ROWS)
        .enumerate()
    {
        let row_y = first_row.saturating_add(index as u16);
        if row_y >= overlay.y.saturating_add(overlay.height).saturating_sub(1) {
            break;
        }
        state.hit_regions.push(T174TuiAgentHitRegion {
            observation_id: row.observation_id,
            column: overlay.x.saturating_add(1),
            row: row_y,
            width: overlay.width.saturating_sub(2),
            height: 1,
        });
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct WorkbenchAccessibilityState {
    verification_inspection_open: bool,
}

impl WorkbenchAccessibilityState {
    const fn verification_inspection_open(self) -> bool {
        self.verification_inspection_open
    }

    fn handle_event(&mut self, event: &Event, search_active: bool) -> bool {
        if search_active {
            return false;
        }
        if matches!(event, Event::Key(key) if key.kind == KeyEventKind::Release) {
            return false;
        }

        if matches!(
            event,
            Event::Key(key)
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('e')
        ) {
            self.verification_inspection_open = !self.verification_inspection_open;
            return true;
        }

        if !self.verification_inspection_open {
            return false;
        }

        match event {
            Event::Key(key)
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('q') =>
            {
                false
            }
            Event::Key(key) if key.code == KeyCode::Esc => {
                self.verification_inspection_open = false;
                true
            }
            Event::Resize(_, _) | Event::FocusGained | Event::FocusLost => false,
            _ => true,
        }
    }
}

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

impl PaneLifecycleView {
    const fn label(self) -> &'static str {
        match self {
            Self::Live => "LIVE",
            Self::Exited => "EXITED",
            Self::Stopped => "STOPPED",
            Self::OwnershipLost => "OWNERSHIP_LOST",
            Self::Error => "ERROR",
        }
    }
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

    /// Return only the presentation-selected pane. Lifecycle integration must
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

    pub(crate) fn set_pane_lifecycle(
        &mut self,
        pane_id: PaneId,
        lifecycle: PaneLifecycleView,
    ) -> bool {
        let Some(index) = self.pane_index(pane_id) else {
            return false;
        };
        self.panes[index].lifecycle = lifecycle;
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

fn use_compact_workbench_layout(width: u16, height: u16) -> bool {
    width < COMPACT_WORKBENCH_MIN_WIDTH || height < COMPACT_WORKBENCH_MIN_HEIGHT
}

fn pane_accessibility_text(state: &WorkbenchState, include_titles: bool) -> String {
    if state.panes().is_empty() {
        return EMPTY_WORKBENCH_MESSAGE.to_owned();
    }

    state
        .panes()
        .iter()
        .map(|pane| {
            let selection = if state.selected_pane() == Some(pane.pane_id) {
                "SELECTED"
            } else {
                "NOT_SELECTED"
            };
            let identity = format!(
                "selection={selection} lifecycle={} workspace={} session={}",
                pane.lifecycle.label(),
                pane.canonical_workspace_id.as_deref().unwrap_or("UNKNOWN"),
                pane.canonical_winds_session_id
                    .as_deref()
                    .unwrap_or("UNBOUND")
            );
            if include_titles {
                format!("{identity} title={}", pane.display_title)
            } else {
                identity
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn agent_progress_accessibility_label(state: context::AgentProgressProjection) -> &'static str {
    match state {
        context::AgentProgressProjection::NotReported => "AGENT_PROGRESS_NOT_REPORTED",
        context::AgentProgressProjection::AgentReportedDone => "AGENT_REPORTED_DONE",
    }
}

fn verification_accessibility_label(state: context::VerificationProjectionState) -> &'static str {
    match state {
        context::VerificationProjectionState::NotRun => "VERIFICATION_NOT_RUN",
        context::VerificationProjectionState::Running => "VERIFICATION_RUNNING",
        context::VerificationProjectionState::VerifiedForExactCandidate => {
            "VERIFIED_FOR_EXACT_CANDIDATE"
        }
        context::VerificationProjectionState::Stale => "VERIFICATION_STALE_NOT_APPLICABLE",
    }
}

fn review_accessibility_label(state: context::ReviewProjectionState) -> &'static str {
    match state {
        context::ReviewProjectionState::NotAvailable => "REVIEW_NOT_AVAILABLE",
        context::ReviewProjectionState::ApplicableToExactCandidate => {
            "REVIEW_APPLICABLE_TO_EXACT_CANDIDATE"
        }
        context::ReviewProjectionState::Stale => "REVIEW_STALE_NOT_APPLICABLE",
    }
}

fn acceptance_accessibility_label(state: context::HumanAcceptanceProjectionState) -> &'static str {
    match state {
        context::HumanAcceptanceProjectionState::NotAccepted => "HUMAN_NOT_ACCEPTED",
        context::HumanAcceptanceProjectionState::AcceptedForExactCandidate => {
            "HUMAN_ACCEPTED_FOR_EXACT_CANDIDATE"
        }
        context::HumanAcceptanceProjectionState::Stale => "HUMAN_ACCEPTANCE_STALE",
    }
}

fn verification_inspection_text(
    candidate_oid: &str,
    candidate_tree: &str,
    projected: Option<&context::WorkbenchCandidateContext>,
) -> String {
    let details = match projected {
        Some(projected) => format!(
            "agent_progress=CANONICAL_STATE_NOT_LOADED\nverification_state={}\nevidence_applicability=applicable:{} stale:{}\nreview_state=CANONICAL_REVIEW_NOT_LOADED\nhuman_acceptance=CANONICAL_DECISION_NOT_LOADED",
            verification_accessibility_label(projected.verification.state),
            projected.verification.applicable_evidence_count,
            projected.verification.stale_evidence_count,
        ),
        None => concat!(
            "agent_progress=CANONICAL_STATE_NOT_LOADED\n",
            "verification_state=CANONICAL_EVIDENCE_NOT_LOADED\n",
            "evidence_applicability=CANONICAL_EVIDENCE_NOT_LOADED\n",
            "review_state=CANONICAL_REVIEW_NOT_LOADED\n",
            "human_acceptance=CANONICAL_DECISION_NOT_LOADED"
        )
        .to_owned(),
    };

    format!(
        "SOURCE=REPOSITORY_EVIDENCE_ONLY\nMODE=READ_ONLY\ncandidate_oid={candidate_oid}\ncandidate_tree={candidate_tree}\n{details}\nINVARIANT=AGENT_REPORTED_DONE != VERIFIED != ACCEPTED\nINVARIANT=TERMINAL_OUTPUT != VERIFICATION_EVIDENCE\nCtrl+E or Esc closes verification inspection"
    )
}

fn load_workbench_candidate_context(
    repo: &Repo,
    store: &crate::store::Store,
) -> Result<context::WorkbenchCandidateContext, String> {
    let repo_path = repo
        .root()
        .to_str()
        .ok_or_else(|| "T098 repository path is not valid UTF-8".to_owned())?;
    let runs = store
        .runs_for_repo(repo_path)
        .map_err(|error| format!("T098 verification-run discovery failed: {error}"))?;
    let mut eligible_run_ids = Vec::new();
    for run in runs {
        let stored = store.load_run(&run.run_id).map_err(|error| {
            format!(
                "T098 verification-run load failed for {}: {error}",
                run.run_id
            )
        })?;
        if stored.eligibility == crate::domain::Eligibility::Eligible {
            eligible_run_ids.push(run.run_id);
        }
    }
    let verification_run_ids = eligible_run_ids
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    context::project_candidate_context(context::WorkbenchContextInput {
        repo,
        store,
        base_ref: "HEAD",
        candidate_ref: "HEAD",
        diff_bytes: None,
        verification_run_ids: &verification_run_ids,
        verification_running: false,
        agent_reported_done: false,
        review: None,
        human_accepted_candidate: None,
    })
}

fn compact_workbench_text_with_context(
    state: &WorkbenchState,
    output: &output::WorkbenchOutput,
    accessibility: WorkbenchAccessibilityState,
    candidate_oid: &str,
    candidate_tree: &str,
    projected: Option<&context::WorkbenchCandidateContext>,
) -> String {
    let mut text = format!(
        "WINDS_WORKBENCH_COMPACT\nCtrl+E verify-inspect | Ctrl+Q quit\ncandidate_oid={candidate_oid}\ncandidate_tree={candidate_tree}\n{}",
        pane_accessibility_text(state, false)
    );
    if accessibility.verification_inspection_open() {
        text.push('\n');
        text.push_str(&verification_inspection_text(
            candidate_oid,
            candidate_tree,
            projected,
        ));
    } else {
        text.push_str("\nSOURCE=TERMINAL_DATA_ONLY\n");
        let terminal_text = state
            .selected_pane()
            .and_then(|pane_id| output.screen_contents(pane_id))
            .unwrap_or_else(|| "No observed terminal output is attached to this pane.".to_owned());
        text.push_str(&terminal_text);
    }
    text
}

fn compact_workbench_text(
    state: &WorkbenchState,
    output: &output::WorkbenchOutput,
    accessibility: WorkbenchAccessibilityState,
    candidate_oid: &str,
    candidate_tree: &str,
) -> String {
    compact_workbench_text_with_context(
        state,
        output,
        accessibility,
        candidate_oid,
        candidate_tree,
        None,
    )
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

#[allow(
    clippy::too_many_arguments,
    reason = "Spec 012 T168 adds presentation-only canonical topology alongside existing workbench render seams"
)]
fn render_workbench_accessible(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
    output: &output::WorkbenchOutput,
    accessibility: WorkbenchAccessibilityState,
    candidate: (&str, &str),
    projected: Option<&context::WorkbenchCandidateContext>,
    canonical_topology: Option<&ui::CanonicalTopologyPresentation>,
    agent_dock: Option<&ui::CanonicalAgentDockPresentation>,
    agent_dock_ui: &mut T174TuiAgentDockState,
) {
    let area = frame.area();
    if use_compact_workbench_layout(area.width, area.height) {
        let mut compact = compact_workbench_text_with_context(
            state,
            output,
            accessibility,
            candidate.0,
            candidate.1,
            projected,
        );
        if let Some(topology) = canonical_topology {
            compact.push_str("\nCANONICAL_TOPOLOGY_BEGIN\n");
            compact.push_str(topology.text());
            compact.push_str("\nCANONICAL_TOPOLOGY_END");
        }
        if agent_dock_ui.open {
            compact.push_str("\nT174_AGENT_DOCK_BEGIN\n");
            compact.push_str(&t174_agent_dock_text(agent_dock, agent_dock_ui));
            compact.push_str("\nT174_AGENT_DOCK_END");
            agent_dock_ui.hit_regions.clear();
        }
        frame.render_widget(
            Paragraph::new(compact).block(Block::bordered().title(" Winds Workbench · compact ")),
            area,
        );
        return;
    }

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(
            "Ctrl+N new | Ctrl+H split-h | Alt+V split-v | Alt+Arrows focus | Alt+Shift+Arrows resize | Ctrl+W close | Ctrl+F find | Ctrl+E verify-inspect | Ctrl+Q quit",
        ),
        areas[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Min(1)])
        .split(areas[1]);
    if let Some(topology) = canonical_topology {
        frame.render_widget(
            Paragraph::new(topology.text())
                .block(Block::bordered().title(" Canonical topology · READ_ONLY_TOPOLOGY ")),
            body[0],
        );
    } else {
        frame.render_widget(
            Paragraph::new(pane_accessibility_text(state, true))
                .block(Block::bordered().title(" Panes · textual state ")),
            body[0],
        );
    }

    if accessibility.verification_inspection_open() {
        frame.render_widget(
            Paragraph::new(verification_inspection_text(
                candidate.0,
                candidate.1,
                projected,
            ))
            .block(Block::bordered().title(" Verification inspection · REPOSITORY_EVIDENCE_ONLY ")),
            body[1],
        );
    } else {
        let terminal_text = state
            .selected_pane()
            .and_then(|pane_id| output.screen_contents(pane_id))
            .unwrap_or_else(|| "No observed terminal output is attached to this pane.".to_owned());
        frame.render_widget(
            Paragraph::new(terminal_text)
                .block(Block::bordered().title(" Terminal output · TERMINAL_DATA_ONLY ")),
            body[1],
        );
    }

    render_t174_agent_dock(frame, body[1], agent_dock, agent_dock_ui);

    let input = editor.lines().join("\n");
    frame.render_widget(
        Paragraph::new(input).block(Block::bordered().title(" Shell input ")),
        areas[2],
    );
}

#[cfg(test)]
fn render_workbench(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
    output: &output::WorkbenchOutput,
) {
    let mut agent_dock_ui = T174TuiAgentDockState::default();
    render_workbench_accessible(
        frame,
        state,
        editor,
        output,
        WorkbenchAccessibilityState::default(),
        (
            "CANONICAL_CANDIDATE_NOT_LOADED",
            "CANONICAL_CANDIDATE_TREE_NOT_LOADED",
        ),
        None,
        None,
        None,
        &mut agent_dock_ui,
    );
}

pub(crate) fn run_cli(args: Vec<String>) -> crate::Result<()> {
    let (repo_path, exit_after_ready) = parse_workbench_args(&args)?;
    if exit_after_ready && std::env::var("WINDS_T097_BENCHMARK").as_deref() != Ok("1") {
        return Err("--t097-exit-after-ready is restricted to WINDS_T097_BENCHMARK=1".into());
    }

    let requested_repo = match repo_path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    let repo = Repo::open(&requested_repo)?;
    let mut candidate_oid = repo.resolve_commit("HEAD")?;
    let mut candidate_tree = repo.tree_oid(&candidate_oid)?;
    let home = crate::winds_home(None, &repo)?;
    let workspace = open_existing_workspace(repo.root(), &home, crate::unix_ms()?)?;
    let store = crate::store::Store::open(&home)?;
    let inventory = inventory_workspace_environment(&workspace)?;
    let profiles = discover_native_shell_profiles(&inventory)?;
    let profile = profiles
        .first()
        .ok_or("no usable native shell profile is available for the workbench")?;
    let (columns, rows) = host_terminal_size()?;
    let size = PaneSize::new(columns.max(1), rows.max(1));

    let mut state = WorkbenchState::new();
    let pane_id = state.create_pane(
        profile.display_name.clone(),
        Some(workspace.workspace_id.clone()),
        None,
        size,
    );
    let mut terminals = terminal::WorkbenchTerminals::new();
    terminals.start_native(
        &mut state,
        pane_id,
        profile,
        std::path::Path::new(&workspace.canonical_worktree_root),
    )?;
    let mut output = output::WorkbenchOutput::new();
    output.attach_live_pane(&mut terminals, &mut state, pane_id)?;
    let mut editor = terminal::input::WorkbenchShellEditor::new();
    let mut navigation = ui::WorkbenchNavigation::new();
    let mut accessibility = WorkbenchAccessibilityState::default();
    let mut projected_context: Option<context::WorkbenchCandidateContext> = None;
    let mut agent_dock: Option<ui::CanonicalAgentDockPresentation> = None;
    let mut agent_dock_ui = T174TuiAgentDockState::default();

    let mut host_guard = HostTerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut host_terminal = Terminal::new(backend)?;
    output.drain_tick(&mut terminals, &mut state)?;
    host_terminal.draw(|frame| {
        render_workbench_accessible(
            frame,
            &state,
            &editor,
            &output,
            accessibility,
            (&candidate_oid, &candidate_tree),
            projected_context.as_ref(),
            navigation.canonical_topology_presentation(),
            agent_dock.as_ref(),
            &mut agent_dock_ui,
        )
    })?;

    if exit_after_ready {
        eprintln!(
            "WINDS_T097_READY={}",
            serde_json::to_string(&json!({
                "state": "INPUT_READY",
                "pane_count": state.panes().len(),
                "selected_live_owned": state
                    .selected_pane()
                    .is_some_and(|selected| terminals.has_owned_terminal(selected)),
                "workspace_id": workspace.workspace_id,
            }))?
        );
        io::stderr().flush()?;
        let cleanup = close_all_workbench_panes(&mut terminals, &mut state);
        drop(host_terminal);
        let restore = host_guard.restore();
        cleanup?;
        restore?;
        return Ok(());
    }

    let mut source = ui::CrosstermHostEventSource;
    let loop_result = (|| -> crate::Result<()> {
        loop {
            output.drain_tick(&mut terminals, &mut state)?;
            host_terminal.draw(|frame| {
                render_workbench_accessible(
                    frame,
                    &state,
                    &editor,
                    &output,
                    accessibility,
                    (&candidate_oid, &candidate_tree),
                    projected_context.as_ref(),
                    navigation.canonical_topology_presentation(),
                    agent_dock.as_ref(),
                    &mut agent_dock_ui,
                )
            })?;

            let Some(event) = <ui::CrosstermHostEventSource as ui::HostEventSource>::next_event(
                &mut source,
                ui::HOST_EVENT_WAIT,
            )?
            else {
                continue;
            };
            let inspection_was_open = accessibility.verification_inspection_open();
            if accessibility.handle_event(&event, navigation.search_query().is_some()) {
                if !inspection_was_open && accessibility.verification_inspection_open() {
                    match load_workbench_candidate_context(&repo, &store) {
                        Ok(projected) => {
                            candidate_oid = projected.candidate.oid.clone();
                            candidate_tree = projected.candidate.tree.clone();
                            projected_context = Some(projected);
                        }
                        Err(_) => {
                            candidate_oid = repo.resolve_commit("HEAD")?;
                            candidate_tree = repo.tree_oid(&candidate_oid)?;
                            projected_context = None;
                        }
                    }
                } else if inspection_was_open && !accessibility.verification_inspection_open() {
                    projected_context = None;
                }
                continue;
            }
            if t174_agent_dock_toggle_event(&event) {
                if agent_dock_ui.open {
                    agent_dock_ui.close();
                    agent_dock = None;
                } else {
                    match load_t174_agent_dock(&store) {
                        Ok(mut loaded) => {
                            agent_dock_ui.open_with(&mut loaded);
                            agent_dock = Some(loaded);
                        }
                        Err(error) => {
                            agent_dock = None;
                            agent_dock_ui.open_unavailable(error);
                        }
                    }
                }
                continue;
            }
            let mut exact_focus_topology = None;
            if let Some(focus_observation_id) = agent_dock_ui.focus_target_for_event(&event) {
                match load_t174_agent_focus_context(&store, focus_observation_id) {
                    Ok((mut refreshed_dock, topology)) => {
                        if let Some(previous_dock) = agent_dock.as_ref()
                            && let Err(error) =
                                preserve_t174_display_aliases(previous_dock, &mut refreshed_dock)
                        {
                            agent_dock = None;
                            agent_dock_ui.open_unavailable(format!(
                                "FOCUS_ALIAS_RECONCILIATION_FAILED · {error}"
                            ));
                            continue;
                        }
                        if refreshed_dock.view(focus_observation_id).is_none() {
                            agent_dock = None;
                            agent_dock_ui.open_unavailable(
                                "FOCUS_REFRESH_FAILED · exact observation disappeared",
                            );
                            continue;
                        }
                        agent_dock_ui.selected_observation_id = Some(focus_observation_id);
                        agent_dock = Some(refreshed_dock);
                        exact_focus_topology = Some(topology);
                    }
                    Err(error) => {
                        agent_dock = None;
                        agent_dock_ui.open_unavailable(format!("FOCUS_REFRESH_FAILED · {error}"));
                        continue;
                    }
                }
            }
            let agent_event_consumed = {
                let topology = exact_focus_topology
                    .as_ref()
                    .or_else(|| navigation.canonical_topology_presentation());
                agent_dock_ui.handle_event(agent_dock.as_mut(), topology, &event)
            };
            if agent_event_consumed {
                continue;
            }
            let effect = navigation.handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                event,
            )?;
            if effect == ui::NavigationEffect::Quit {
                return Ok(());
            }
        }
    })();
    let cleanup = close_all_workbench_panes(&mut terminals, &mut state);
    drop(host_terminal);
    let restore = host_guard.restore();

    loop_result?;
    cleanup?;
    restore?;
    Ok(())
}

#[cfg(test)]
mod t174_tui_agent_dock_integration_tests {
    use super::*;
    use crate::multiplexer::domain::{AgentObservationId, MultiplexerWorkspaceId, PaneId, TabId};
    use crate::persistent_runtime::domain::{OwnerGenerationId, RuntimeNamespaceId};
    use crate::persistent_runtime::protocol::{
        AgentFamilyV2, AgentObservationConfidenceV2, AgentObservationFreshnessV2,
        AgentObservationSourceV2, AgentObservationV2,
    };

    fn observation_id(byte: u8) -> AgentObservationId {
        AgentObservationId::from_entropy_bytes([byte; 16]).expect("valid observation id")
    }

    fn observation(byte: u8, pane_byte: u8) -> AgentObservationV2 {
        AgentObservationV2 {
            observation_id: observation_id(byte),
            family: AgentFamilyV2::Codex,
            source_class: AgentObservationSourceV2::ProviderStructuredMetadata,
            confidence_class: AgentObservationConfidenceV2::Strong,
            freshness: AgentObservationFreshnessV2::Current,
            multiplexer_workspace_id: MultiplexerWorkspaceId::from_entropy_bytes([0x21; 16])
                .expect("valid workspace id"),
            git_workspace_id: Some("git-workspace".to_owned()),
            tab_id: TabId::from_entropy_bytes([0x31; 16]).expect("valid tab id"),
            pane_id: PaneId::from_entropy_bytes([pane_byte; 16]).expect("valid pane id"),
            runtime_namespace_id: Some(
                RuntimeNamespaceId::from_entropy_bytes([0x51; 16]).expect("valid runtime id"),
            ),
            provider_native_session_id: Some(format!("provider-{byte}")),
            owner_generation_id: OwnerGenerationId::from_entropy_bytes([0x61; 16])
                .expect("valid owner generation"),
            observed_unix_ms: i64::from(byte),
            structured_evidence_summary: "structured evidence".to_owned(),
        }
    }

    #[test]
    fn t174_tui_render_exposes_detection_nonclaim_and_exact_selected_binding() {
        let first = observation(0x11, 0x41);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![first.clone()])
            .expect("observation should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        let text = t174_agent_dock_text(Some(&dock), &state);
        assert!(text.contains("DETECTION_ONLY_UNPROVEN"));
        assert!(text.contains(&first.observation_id.to_string()));
        assert!(text.contains(&first.multiplexer_workspace_id.to_string()));
        assert!(text.contains(&first.tab_id.to_string()));
        assert!(text.contains(&first.pane_id.to_string()));
    }

    #[test]
    fn t174_tui_pointer_regions_select_exact_observation_not_duplicate_label() {
        let first = observation(0x12, 0x42);
        let second = observation(0x13, 0x43);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            second.clone(),
        ])
        .expect("distinct observations should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        state.hit_regions = vec![
            T174TuiAgentHitRegion {
                observation_id: first.observation_id,
                column: 10,
                row: 4,
                width: 20,
                height: 1,
            },
            T174TuiAgentHitRegion {
                observation_id: second.observation_id,
                column: 10,
                row: 5,
                width: 20,
                height: 1,
            },
        ];
        let consumed = state.handle_event(
            Some(&mut dock),
            None,
            &Event::Mouse(crossterm::event::MouseEvent {
                kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
                column: 12,
                row: 5,
                modifiers: KeyModifiers::NONE,
            }),
        );
        assert!(consumed);
        assert_eq!(state.selected_observation_id, Some(second.observation_id));
        assert_eq!(dock.read(second.observation_id), Some(&second));
        assert_eq!(dock.read(first.observation_id), Some(&first));
    }

    #[test]
    fn t174_tui_focus_fails_closed_when_canonical_topology_is_unavailable() {
        let first = observation(0x14, 0x44);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![first.clone()])
            .expect("observation should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        let consumed = state.handle_event(
            Some(&mut dock),
            None,
            &Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )),
        );
        assert!(consumed);
        assert!(state.status.contains("canonical topology unavailable"));
        assert_eq!(dock.focused_observation_id(), None);
        assert_eq!(dock.read(first.observation_id), Some(&first));
    }

    #[test]
    fn t174_tui_focus_event_resolves_exact_pointer_observation_before_refresh() {
        let first = observation(0x16, 0x46);
        let second = observation(0x17, 0x47);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            second.clone(),
        ])
        .expect("distinct observations should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        state.hit_regions = vec![
            T174TuiAgentHitRegion {
                observation_id: first.observation_id,
                column: 20,
                row: 7,
                width: 20,
                height: 1,
            },
            T174TuiAgentHitRegion {
                observation_id: second.observation_id,
                column: 20,
                row: 8,
                width: 20,
                height: 1,
            },
        ];
        let event = Event::Mouse(crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right),
            column: 23,
            row: 8,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(
            state.focus_target_for_event(&event),
            Some(second.observation_id)
        );
    }

    #[test]
    fn t174_tui_refresh_preserves_display_aliases_only_for_surviving_exact_ids() {
        let first = observation(0x18, 0x48);
        let removed = observation(0x19, 0x49);
        let newcomer = observation(0x1a, 0x4a);
        let mut previous = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            removed.clone(),
        ])
        .expect("previous observations should project");
        previous
            .rename(first.observation_id, Some("Primary"))
            .expect("alias should be accepted");
        previous
            .rename(removed.observation_id, Some("Removed"))
            .expect("alias should be accepted");
        let mut refreshed = ui::CanonicalAgentDockPresentation::from_observations(vec![
            first.clone(),
            newcomer.clone(),
        ])
        .expect("refreshed observations should project");
        preserve_t174_display_aliases(&previous, &mut refreshed)
            .expect("valid aliases should reconcile");
        assert_eq!(
            refreshed
                .get(first.observation_id)
                .and_then(|item| item.display_alias.as_deref()),
            Some("Primary")
        );
        assert!(refreshed.get(removed.observation_id).is_none());
        assert_eq!(
            refreshed
                .get(newcomer.observation_id)
                .and_then(|item| item.display_alias.as_deref()),
            None
        );
    }

    #[test]
    fn t174_tui_alias_edit_changes_display_only_not_observation_truth() {
        let first = observation(0x15, 0x45);
        let mut dock = ui::CanonicalAgentDockPresentation::from_observations(vec![first.clone()])
            .expect("observation should project");
        let mut state = T174TuiAgentDockState::default();
        state.open_with(&mut dock);
        assert!(state.handle_event(
            Some(&mut dock),
            None,
            &Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Char('r'),
                KeyModifiers::NONE
            )),
        ));
        for character in "Reviewer".chars() {
            assert!(state.handle_event(
                Some(&mut dock),
                None,
                &Event::Key(crossterm::event::KeyEvent::new(
                    KeyCode::Char(character),
                    KeyModifiers::NONE,
                )),
            ));
        }
        assert!(state.handle_event(
            Some(&mut dock),
            None,
            &Event::Key(crossterm::event::KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE
            )),
        ));
        let item = dock
            .get(first.observation_id)
            .expect("item should remain present");
        assert_eq!(item.display_alias.as_deref(), Some("Reviewer"));
        assert_eq!(dock.read(first.observation_id), Some(&first));
    }
}

fn parse_workbench_args(args: &[String]) -> crate::Result<(Option<PathBuf>, bool)> {
    let mut repo = None;
    let mut exit_after_ready = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                if repo.is_some() {
                    return Err("duplicate flag --repo".into());
                }
                let value = args.get(index + 1).ok_or("missing value for --repo")?;
                if value.starts_with("--") {
                    return Err("missing value for --repo".into());
                }
                repo = Some(PathBuf::from(value));
                index += 2;
            }
            "--t097-exit-after-ready" => {
                if exit_after_ready {
                    return Err("duplicate flag --t097-exit-after-ready".into());
                }
                exit_after_ready = true;
                index += 1;
            }
            other => return Err(format!("unknown workbench argument: {other}").into()),
        }
    }
    Ok((repo, exit_after_ready))
}

fn close_all_workbench_panes(
    terminals: &mut terminal::WorkbenchTerminals,
    state: &mut WorkbenchState,
) -> crate::Result<()> {
    let pane_ids: Vec<PaneId> = state.panes().iter().map(|pane| pane.pane_id).collect();
    for pane_id in pane_ids {
        if terminals.has_owned_terminal(pane_id) {
            terminals.close_pane(state, pane_id)?;
        } else {
            state.close_pane(pane_id);
        }
    }
    Ok(())
}

struct HostTerminalGuard {
    active: bool,
}

impl HostTerminalGuard {
    fn enter() -> crate::Result<Self> {
        enable_raw_mode()?;
        let mut guard = Self { active: true };
        if let Err(error) = crossterm::execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        ) {
            let _ = guard.restore();
            return Err(error.into());
        }
        Ok(guard)
    }

    fn restore(&mut self) -> crate::Result<()> {
        if !self.active {
            return Ok(());
        }
        let terminal_restore = crossterm::execute!(
            io::stdout(),
            DisableBracketedPaste,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let raw_restore = disable_raw_mode();
        self.active = false;
        terminal_restore?;
        raw_restore?;
        Ok(())
    }
}

impl Drop for HostTerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[path = "workbench_context.rs"]
pub(crate) mod context;
#[path = "workbench_interaction.rs"]
pub(crate) mod interaction;
#[path = "workbench_output.rs"]
pub(crate) mod output;
#[path = "workbench_screen.rs"]
pub(crate) mod screen;
#[path = "workbench_terminal.rs"]
pub(crate) mod terminal;
#[path = "workbench_terminal_ux.rs"]
pub(crate) mod terminal_ux;
#[path = "workbench_ui.rs"]
pub(crate) mod ui;

#[cfg(test)]
#[path = "t088_workbench_topology_tests.rs"]
mod t088_workbench_topology_tests;
#[cfg(test)]
#[path = "t090_workbench_terminal_tests.rs"]
mod t090_workbench_terminal_tests;
#[cfg(test)]
#[path = "t096_workbench_platform_tests.rs"]
mod t096_workbench_platform_tests;
#[cfg(test)]
#[path = "t097_workbench_performance_tests.rs"]
mod t097_workbench_performance_tests;
#[cfg(test)]
#[path = "t098_workbench_accessibility_tests.rs"]
mod t098_workbench_accessibility_tests;
