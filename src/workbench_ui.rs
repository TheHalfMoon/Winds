use super::terminal::WorkbenchTerminals;
use super::terminal::input::{
    MultilineSubmitPolicy, ShellCursorMove, ShellDispatchReceipt, ShellSubmitTerminator,
    WorkbenchShellEditor,
};
use super::{PaneId, PaneLifecycleView, PaneSize, SplitAxis, WorkbenchState};
use crate::domain::{WindsSessionRecord, WorkspaceRecord};
use crate::git::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use std::collections::BTreeMap;
use std::time::Duration;

pub(crate) const HOST_EVENT_WAIT: Duration = Duration::from_millis(250);
const DEFAULT_PANE_SIZE: PaneSize = PaneSize::new(80, 24);
type CanonicalFindSources<'a> = (&'a [WorkspaceRecord], &'a [WindsSessionRecord]);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NavigationTarget {
    Pane(PaneId),
    Workspace {
        workspace_id: String,
    },
    Session {
        session_id: String,
        workstream_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FindRank {
    ExactCanonicalId,
    ExactNormalizedLabel,
    NormalizedPrefix,
    NormalizedSubstring,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindMatch {
    pub(crate) target: NavigationTarget,
    pub(crate) display_label: String,
    pub(crate) canonical_context: String,
    rank: FindRank,
    stable_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FindResolution {
    NotFound,
    Unique(FindMatch),
    Ambiguous(Vec<FindMatch>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PaneHitRegion {
    pub(crate) pane_id: PaneId,
    pub(crate) column: u16,
    pub(crate) row: u16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

impl PaneHitRegion {
    fn contains(self, column: u16, row: u16) -> bool {
        column >= self.column
            && row >= self.row
            && column < self.column.saturating_add(self.width)
            && row < self.row.saturating_add(self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NavigationEffect {
    None,
    Quit,
    Find(FindResolution),
    Dispatch(ShellDispatchReceipt),
    CanonicalTopologyIntent(CanonicalTopologyBoundIntent),
    CanonicalPaneInteraction(super::terminal_ux::BoundPaneInteraction),
}

#[derive(Debug, Default)]
pub(crate) struct WorkbenchNavigation {
    search_query: Option<String>,
    last_find: Option<FindResolution>,
    selected_canonical_target: Option<NavigationTarget>,
    hit_regions: Vec<PaneHitRegion>,
    canonical_topology: Option<CanonicalTopologyPresentation>,
    canonical_topology_hit_regions: Vec<CanonicalTopologyPaneHitRegion>,
    canonical_pointer_capture: Option<CanonicalPointerCapture>,
}

/// Exact pane identity captured at pointer-down together with the gesture-local
/// drag state. Nothing here is re-read from focus at action time, so a focus or
/// topology change between press and release cannot retarget the interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanonicalPointerCapture {
    target: super::terminal_ux::ExactPaneTarget,
    dragged: bool,
}

impl WorkbenchNavigation {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }

    pub(crate) fn last_find(&self) -> Option<&FindResolution> {
        self.last_find.as_ref()
    }

    pub(crate) fn selected_canonical_target(&self) -> Option<&NavigationTarget> {
        self.selected_canonical_target.as_ref()
    }

    pub(crate) fn set_hit_regions(&mut self, hit_regions: Vec<PaneHitRegion>) {
        self.hit_regions = hit_regions;
    }

    pub(crate) fn install_canonical_topology(
        &mut self,
        presentation: CanonicalTopologyPresentation,
        hit_regions: Vec<CanonicalTopologyPaneHitRegion>,
    ) {
        self.canonical_topology = Some(presentation);
        self.canonical_topology_hit_regions = hit_regions;
        self.search_query = None;
        self.last_find = None;
        self.selected_canonical_target = None;
        self.canonical_pointer_capture = None;
    }

    pub(crate) fn clear_canonical_topology(&mut self) {
        self.canonical_topology = None;
        self.canonical_topology_hit_regions.clear();
        self.search_query = None;
        self.canonical_pointer_capture = None;
    }

    pub(crate) fn canonical_topology_presentation(&self) -> Option<&CanonicalTopologyPresentation> {
        self.canonical_topology.as_ref()
    }

    pub(crate) fn handle_event(
        &mut self,
        state: &mut WorkbenchState,
        terminals: &mut WorkbenchTerminals,
        editor: &mut WorkbenchShellEditor,
        workspaces: &[WorkspaceRecord],
        sessions: &[WindsSessionRecord],
        event: Event,
    ) -> Result<NavigationEffect> {
        if matches!(&event, Event::Key(key) if key.kind == KeyEventKind::Release) {
            return Ok(NavigationEffect::None);
        }

        if self.canonical_topology.is_some()
            && let Some(effect) = self.handle_canonical_topology_event(&event)
        {
            return Ok(effect);
        }

        if self.search_query.is_some() && !matches!(&event, Event::Resize(_, _)) {
            return self.handle_search_event(state, workspaces, sessions, event);
        }

        match event {
            Event::Key(key) => self.handle_shell_key(state, terminals, editor, key),
            Event::Paste(text) => {
                if text.contains('\n') || text.contains('\r') {
                    editor.paste_multiline_literal(&text)?;
                } else {
                    editor.paste_single_line(&text)?;
                }
                Ok(NavigationEffect::None)
            }
            Event::Mouse(mouse) => {
                if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                    let matches: Vec<PaneId> = self
                        .hit_regions
                        .iter()
                        .filter(|region| region.contains(mouse.column, mouse.row))
                        .map(|region| region.pane_id)
                        .collect();
                    if matches.len() == 1 {
                        state.focus_pane(matches[0]);
                    }
                }
                Ok(NavigationEffect::None)
            }
            Event::Resize(columns, rows) => {
                // Host dimensions determine pane dimensions only when topology is unambiguous.
                // Multi-pane geometry is presentation state that T092 must not guess.
                if state.panes().len() == 1 {
                    let size = PaneSize::new(columns.max(1), rows.max(1));
                    resize_selected(state, terminals, size)?;
                }
                Ok(NavigationEffect::None)
            }
            Event::FocusGained | Event::FocusLost => Ok(NavigationEffect::None),
        }
    }

    fn handle_canonical_topology_event(&mut self, event: &Event) -> Option<NavigationEffect> {
        let presentation = self.canonical_topology.as_ref()?;

        if self.search_query.is_some() && !matches!(event, Event::Resize(_, _)) {
            let Event::Key(key) = event else {
                return Some(NavigationEffect::None);
            };
            let effect = match key.code {
                KeyCode::Esc => {
                    self.search_query = None;
                    NavigationEffect::None
                }
                KeyCode::Backspace => {
                    if let Some(query) = &mut self.search_query {
                        query.pop();
                    }
                    NavigationEffect::None
                }
                KeyCode::Enter => {
                    let query = self.search_query.take().unwrap_or_default();
                    presentation
                        .bind_search_focus(&query)
                        .map(NavigationEffect::CanonicalTopologyIntent)
                        .unwrap_or(NavigationEffect::None)
                }
                KeyCode::Char(character)
                    if !key.modifiers.contains(KeyModifiers::CONTROL)
                        && !key.modifiers.contains(KeyModifiers::ALT) =>
                {
                    if let Some(query) = &mut self.search_query {
                        query.push(character);
                    }
                    NavigationEffect::None
                }
                _ => NavigationEffect::None,
            };
            return Some(effect);
        }

        match event {
            Event::Key(key)
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('f') =>
            {
                self.search_query = Some(String::new());
                self.last_find = None;
                Some(NavigationEffect::None)
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                self.canonical_pointer_capture = canonical_topology_exact_pane_target(
                    presentation,
                    &self.canonical_topology_hit_regions,
                    mouse.column,
                    mouse.row,
                )
                .map(|target| CanonicalPointerCapture {
                    target,
                    dragged: false,
                });
                Some(
                    canonical_topology_bind_pointer_focus_intent(
                        presentation,
                        &self.canonical_topology_hit_regions,
                        mouse.column,
                        mouse.row,
                    )
                    .map(NavigationEffect::CanonicalTopologyIntent)
                    .unwrap_or(NavigationEffect::None),
                )
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(capture) = self.canonical_pointer_capture.as_mut() {
                    capture.dragged = true;
                }
                let effect = self
                    .canonical_pointer_capture
                    .map(|capture| capture.target)
                    .and_then(|target| {
                        super::terminal_ux::bind_exact_pane_interaction(
                            target,
                            presentation.topology_generation(),
                            super::terminal_ux::PaneInteractionKind::MouseCapture {
                                button: super::terminal_ux::PointerButton::Left,
                                phase: super::terminal_ux::PointerPhase::Drag,
                            },
                            mouse.column,
                            mouse.row,
                        )
                    });
                Some(
                    effect
                        .map(NavigationEffect::CanonicalPaneInteraction)
                        .unwrap_or(NavigationEffect::None),
                )
            }
            Event::Mouse(mouse) if mouse.kind == MouseEventKind::Up(MouseButton::Left) => {
                let captured = self.canonical_pointer_capture.take();
                let effect = captured.and_then(|capture| {
                    let kind = if capture.dragged {
                        super::terminal_ux::PaneInteractionKind::CopyOnSelect
                    } else {
                        super::terminal_ux::PaneInteractionKind::MouseCapture {
                            button: super::terminal_ux::PointerButton::Left,
                            phase: super::terminal_ux::PointerPhase::Up,
                        }
                    };
                    super::terminal_ux::bind_exact_pane_interaction(
                        capture.target,
                        presentation.topology_generation(),
                        kind,
                        mouse.column,
                        mouse.row,
                    )
                });
                Some(
                    effect
                        .map(NavigationEffect::CanonicalPaneInteraction)
                        .unwrap_or(NavigationEffect::None),
                )
            }
            Event::Mouse(mouse) => Some(
                canonical_topology_bind_terminal_interaction(
                    presentation,
                    &self.canonical_topology_hit_regions,
                    *mouse,
                )
                .map(NavigationEffect::CanonicalPaneInteraction)
                .unwrap_or(NavigationEffect::None),
            ),
            Event::Key(key) if canonical_topology_local_mutation_binding(*key) => {
                Some(NavigationEffect::None)
            }
            _ => None,
        }
    }

    fn handle_search_event(
        &mut self,
        state: &mut WorkbenchState,
        workspaces: &[WorkspaceRecord],
        sessions: &[WindsSessionRecord],
        event: Event,
    ) -> Result<NavigationEffect> {
        let Event::Key(key) = event else {
            return Ok(NavigationEffect::None);
        };
        match key.code {
            KeyCode::Esc => {
                self.search_query = None;
                self.last_find = None;
                Ok(NavigationEffect::None)
            }
            KeyCode::Backspace => {
                if let Some(query) = &mut self.search_query {
                    query.pop();
                }
                Ok(NavigationEffect::None)
            }
            KeyCode::Enter => {
                let query = self.search_query.as_deref().unwrap_or_default();
                let resolution = resolve_find_query(query, state, workspaces, sessions);
                self.selected_canonical_target = None;
                if let FindResolution::Unique(found) = &resolution {
                    match &found.target {
                        NavigationTarget::Pane(pane_id) => {
                            state.focus_pane(*pane_id);
                        }
                        target @ (NavigationTarget::Workspace { .. }
                        | NavigationTarget::Session { .. }) => {
                            self.selected_canonical_target = Some(target.clone());
                        }
                    }
                }
                self.last_find = Some(resolution.clone());
                self.search_query = None;
                Ok(NavigationEffect::Find(resolution))
            }
            KeyCode::Char(character)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                if let Some(query) = &mut self.search_query {
                    query.push(character);
                }
                Ok(NavigationEffect::None)
            }
            _ => Ok(NavigationEffect::None),
        }
    }

    fn handle_shell_key(
        &mut self,
        state: &mut WorkbenchState,
        terminals: &mut WorkbenchTerminals,
        editor: &mut WorkbenchShellEditor,
        key: KeyEvent,
    ) -> Result<NavigationEffect> {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('q') => return Ok(NavigationEffect::Quit),
                KeyCode::Char('f') => {
                    self.search_query = Some(String::new());
                    self.last_find = None;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Char('n') => {
                    let pane_id = create_presentation_pane(state);
                    state.focus_pane(pane_id);
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Char('h') => {
                    split_selected(state, SplitAxis::Horizontal)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Char('w') => {
                    close_selected(state, terminals)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Enter => {
                    let receipt = editor.submit_selected(
                        state,
                        terminals,
                        MultilineSubmitPolicy::Literal,
                        ShellSubmitTerminator::LineFeed,
                    )?;
                    return Ok(NavigationEffect::Dispatch(receipt));
                }
                _ => {}
            }
        }

        if key.modifiers.contains(KeyModifiers::ALT) {
            match key.code {
                KeyCode::Char('v') => {
                    split_selected(state, SplitAxis::Vertical)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    resize_selected_by(state, terminals, -1, 0)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    resize_selected_by(state, terminals, 1, 0)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    resize_selected_by(state, terminals, 0, -1)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    resize_selected_by(state, terminals, 0, 1)?;
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Left => {
                    focus_relative(state, -1);
                    return Ok(NavigationEffect::None);
                }
                KeyCode::Right => {
                    focus_relative(state, 1);
                    return Ok(NavigationEffect::None);
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
                editor.insert_newline();
                Ok(NavigationEffect::None)
            }
            KeyCode::Enter => {
                let receipt = editor.submit_selected(
                    state,
                    terminals,
                    MultilineSubmitPolicy::Reject,
                    ShellSubmitTerminator::LineFeed,
                )?;
                Ok(NavigationEffect::Dispatch(receipt))
            }
            KeyCode::Backspace => {
                editor.backspace();
                Ok(NavigationEffect::None)
            }
            KeyCode::Delete => {
                editor.delete_next();
                Ok(NavigationEffect::None)
            }
            KeyCode::Left => {
                editor.move_cursor(ShellCursorMove::Back);
                Ok(NavigationEffect::None)
            }
            KeyCode::Right => {
                editor.move_cursor(ShellCursorMove::Forward);
                Ok(NavigationEffect::None)
            }
            KeyCode::Up => {
                editor.move_cursor(ShellCursorMove::Up);
                Ok(NavigationEffect::None)
            }
            KeyCode::Down => {
                editor.move_cursor(ShellCursorMove::Down);
                Ok(NavigationEffect::None)
            }
            KeyCode::Home => {
                editor.move_cursor(ShellCursorMove::Head);
                Ok(NavigationEffect::None)
            }
            KeyCode::End => {
                editor.move_cursor(ShellCursorMove::End);
                Ok(NavigationEffect::None)
            }
            KeyCode::Tab => {
                editor.insert_char('\t')?;
                Ok(NavigationEffect::None)
            }
            KeyCode::Char(character)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                editor.insert_char(character)?;
                Ok(NavigationEffect::None)
            }
            _ => Ok(NavigationEffect::None),
        }
    }
}

pub(crate) trait HostEventSource {
    fn next_event(&mut self, wait: Duration) -> Result<Option<Event>>;
}

#[derive(Debug, Default)]
pub(crate) struct CrosstermHostEventSource;

impl HostEventSource for CrosstermHostEventSource {
    fn next_event(&mut self, wait: Duration) -> Result<Option<Event>> {
        if event::poll(wait)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

pub(crate) fn run_host_event_loop<S, R>(
    navigation: &mut WorkbenchNavigation,
    state: &mut WorkbenchState,
    terminals: &mut WorkbenchTerminals,
    editor: &mut WorkbenchShellEditor,
    find_sources: CanonicalFindSources<'_>,
    source: &mut S,
    mut render: R,
) -> Result<()>
where
    S: HostEventSource,
    R: FnMut(&WorkbenchState, &WorkbenchNavigation, &WorkbenchShellEditor) -> Result<()>,
{
    render(state, navigation, editor)?;
    loop {
        let Some(event) = source.next_event(HOST_EVENT_WAIT)? else {
            continue;
        };
        let effect = navigation.handle_event(
            state,
            terminals,
            editor,
            find_sources.0,
            find_sources.1,
            event,
        )?;
        render(state, navigation, editor)?;
        if effect == NavigationEffect::Quit {
            return Ok(());
        }
    }
}

pub(crate) fn find_matches(
    query: &str,
    state: &WorkbenchState,
    workspaces: &[WorkspaceRecord],
    sessions: &[WindsSessionRecord],
) -> Vec<FindMatch> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for (index, pane) in state.panes().iter().enumerate() {
        if let Some(rank) = match_rank(&normalized_query, None, &[&pane.display_title]) {
            matches.push(FindMatch {
                target: NavigationTarget::Pane(pane.pane_id),
                display_label: pane.display_title.clone(),
                canonical_context: format!(
                    "workspace={} session={}",
                    pane.canonical_workspace_id.as_deref().unwrap_or("UNKNOWN"),
                    pane.canonical_winds_session_id
                        .as_deref()
                        .unwrap_or("UNKNOWN")
                ),
                rank,
                stable_key: format!("0-pane-{index:020}"),
            });
        }
    }

    let mut sorted_workspaces: Vec<&WorkspaceRecord> = workspaces.iter().collect();
    sorted_workspaces.sort_by(|left, right| left.workspace_id.cmp(&right.workspace_id));
    for workspace in sorted_workspaces {
        if let Some(rank) = match_rank(
            &normalized_query,
            Some(&workspace.workspace_id),
            &[&workspace.canonical_worktree_root],
        ) {
            matches.push(FindMatch {
                target: NavigationTarget::Workspace {
                    workspace_id: workspace.workspace_id.clone(),
                },
                display_label: workspace.canonical_worktree_root.clone(),
                canonical_context: format!("workspace_id={}", workspace.workspace_id),
                rank,
                stable_key: format!("1-workspace-{}", workspace.workspace_id),
            });
        }
    }

    let mut sorted_sessions: Vec<&WindsSessionRecord> = sessions.iter().collect();
    sorted_sessions.sort_by(|left, right| left.session_id.cmp(&right.session_id));
    for session in sorted_sessions {
        if let Some(rank) = match_rank(
            &normalized_query,
            Some(&session.session_id),
            &[&session.display_name, &session.workstream_id],
        ) {
            matches.push(FindMatch {
                target: NavigationTarget::Session {
                    session_id: session.session_id.clone(),
                    workstream_id: session.workstream_id.clone(),
                },
                display_label: session.display_name.clone(),
                canonical_context: format!(
                    "workstream_id={} session_id={}",
                    session.workstream_id, session.session_id
                ),
                rank,
                stable_key: format!("2-session-{}", session.session_id),
            });
        }
    }

    matches.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.stable_key.cmp(&right.stable_key))
    });
    matches
}

pub(crate) fn resolve_find_query(
    query: &str,
    state: &WorkbenchState,
    workspaces: &[WorkspaceRecord],
    sessions: &[WindsSessionRecord],
) -> FindResolution {
    let matches = find_matches(query, state, workspaces, sessions);
    let Some(best_rank) = matches.first().map(|found| found.rank) else {
        return FindResolution::NotFound;
    };
    let best: Vec<FindMatch> = matches
        .into_iter()
        .take_while(|found| found.rank == best_rank)
        .collect();
    if best.len() == 1 {
        FindResolution::Unique(
            best.into_iter()
                .next()
                .expect("one best find match was just proven"),
        )
    } else {
        FindResolution::Ambiguous(best)
    }
}

fn normalize(value: &str) -> String {
    value.trim().chars().flat_map(char::to_lowercase).collect()
}

fn match_rank(query: &str, canonical_id: Option<&str>, labels: &[&str]) -> Option<FindRank> {
    if canonical_id.is_some_and(|value| normalize(value) == query) {
        return Some(FindRank::ExactCanonicalId);
    }
    let normalized_labels: Vec<String> = labels.iter().map(|value| normalize(value)).collect();
    if normalized_labels.iter().any(|value| value == query) {
        return Some(FindRank::ExactNormalizedLabel);
    }

    let mut searchable = normalized_labels;
    if let Some(canonical_id) = canonical_id {
        searchable.push(normalize(canonical_id));
    }
    if searchable.iter().any(|value| value.starts_with(query)) {
        return Some(FindRank::NormalizedPrefix);
    }
    searchable
        .iter()
        .any(|value| value.contains(query))
        .then_some(FindRank::NormalizedSubstring)
}

fn create_presentation_pane(state: &mut WorkbenchState) -> PaneId {
    let selected = state
        .selected_pane()
        .and_then(|pane_id| state.pane(pane_id))
        .cloned();
    let size = selected
        .as_ref()
        .map_or(DEFAULT_PANE_SIZE, |pane| pane.size);
    state.create_pane(
        "shell",
        selected
            .as_ref()
            .and_then(|pane| pane.canonical_workspace_id.clone()),
        selected
            .as_ref()
            .and_then(|pane| pane.canonical_winds_session_id.clone()),
        size,
    )
}

fn split_selected(state: &mut WorkbenchState, axis: SplitAxis) -> Result<PaneId> {
    let pane_id = state
        .selected_pane()
        .ok_or("workbench split requires an explicitly selected pane")?;
    state
        .split_pane(pane_id, axis, "shell")
        .ok_or_else(|| "selected pane disappeared before split".into())
}

fn focus_relative(state: &mut WorkbenchState, delta: isize) -> bool {
    let panes = state.panes();
    if panes.is_empty() {
        return false;
    }
    let current = state
        .selected_pane()
        .and_then(|pane_id| panes.iter().position(|pane| pane.pane_id == pane_id))
        .unwrap_or(0);
    let len = panes.len() as isize;
    let next = (current as isize + delta).rem_euclid(len) as usize;
    let pane_id = panes[next].pane_id;
    state.focus_pane(pane_id)
}

fn close_selected(state: &mut WorkbenchState, terminals: &mut WorkbenchTerminals) -> Result<()> {
    let pane_id = state
        .selected_pane()
        .ok_or("workbench close requires an explicitly selected pane")?;
    terminals.close_pane(state, pane_id)?;
    Ok(())
}

fn resize_selected_by(
    state: &mut WorkbenchState,
    terminals: &mut WorkbenchTerminals,
    columns_delta: i16,
    rows_delta: i16,
) -> Result<()> {
    let pane_id = state
        .selected_pane()
        .ok_or("workbench resize requires an explicitly selected pane")?;
    let current = state
        .pane(pane_id)
        .ok_or("selected workbench pane disappeared before resize")?
        .size;
    let columns = i32::from(current.columns)
        .saturating_add(i32::from(columns_delta))
        .clamp(1, i32::from(u16::MAX)) as u16;
    let rows = i32::from(current.rows)
        .saturating_add(i32::from(rows_delta))
        .clamp(1, i32::from(u16::MAX)) as u16;
    resize_selected(state, terminals, PaneSize::new(columns, rows))
}

fn resize_selected(
    state: &mut WorkbenchState,
    terminals: &mut WorkbenchTerminals,
    size: PaneSize,
) -> Result<()> {
    let Some(pane_id) = state.selected_pane() else {
        return Ok(());
    };
    let lifecycle = state
        .pane(pane_id)
        .ok_or("selected workbench pane disappeared before resize")?
        .lifecycle;
    if lifecycle == PaneLifecycleView::Live {
        terminals.resize(state, pane_id, size)
    } else {
        if !state.resize_pane(pane_id, size) {
            return Err("selected workbench pane disappeared during resize".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTopologyBoundIntent {
    intent: crate::persistent_runtime::protocol::ApplyTopologyOperationV2,
}

impl CanonicalTopologyBoundIntent {
    fn new(intent: crate::persistent_runtime::protocol::ApplyTopologyOperationV2) -> Self {
        Self { intent }
    }

    pub(crate) fn into_intent(
        self,
    ) -> crate::persistent_runtime::protocol::ApplyTopologyOperationV2 {
        self.intent
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalTopologySearchBinding {
    canonical_id: String,
    display_label: String,
    stable_key: String,
    intent: crate::persistent_runtime::protocol::ApplyTopologyOperationV2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalTopologyPresentation {
    topology_generation: crate::multiplexer::domain::TopologyGeneration,
    text: String,
    search_bindings: Vec<CanonicalTopologySearchBinding>,
}

impl CanonicalTopologyPresentation {
    pub(crate) fn from_client(
        client: &crate::persistent_runtime::client::RustLocalControlClient,
        reduced_motion: bool,
        high_contrast: bool,
        scaled_text: bool,
    ) -> Option<Self> {
        let (topology_generation, lines) =
            client.tui_topology_rendered_snapshot(reduced_motion, high_contrast, scaled_text)?;
        let bindings = client.tui_topology_search_bindings(topology_generation)?;
        let search_bindings = bindings
            .into_iter()
            .map(|(canonical_id, display_label, stable_key, intent)| {
                CanonicalTopologySearchBinding {
                    canonical_id,
                    display_label,
                    stable_key,
                    intent,
                }
            })
            .collect();
        Some(Self {
            topology_generation,
            text: lines.join("\n"),
            search_bindings,
        })
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) const fn topology_generation(
        &self,
    ) -> crate::multiplexer::domain::TopologyGeneration {
        self.topology_generation
    }

    fn bind_search_focus(&self, query: &str) -> Option<CanonicalTopologyBoundIntent> {
        let query = normalize_canonical_topology_query(query);
        if query.is_empty() {
            return None;
        }
        let mut matches: Vec<(FindRank, &CanonicalTopologySearchBinding)> = self
            .search_bindings
            .iter()
            .filter_map(|binding| {
                canonical_topology_match_rank(&query, binding).map(|rank| (rank, binding))
            })
            .collect();
        matches.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.stable_key.cmp(&right.1.stable_key))
        });
        let best_rank = matches.first()?.0;
        let mut best = matches.into_iter().take_while(|item| item.0 == best_rank);
        let binding = best.next()?.1;
        if best.next().is_some() {
            return None;
        }
        (binding.intent.expected_topology_generation == self.topology_generation)
            .then(|| CanonicalTopologyBoundIntent::new(binding.intent.clone()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalTopologyPaneHitRegion {
    pub(crate) multiplexer_workspace_id: crate::multiplexer::domain::MultiplexerWorkspaceId,
    pub(crate) tab_id: crate::multiplexer::domain::TabId,
    pub(crate) pane_id: crate::multiplexer::domain::PaneId,
    pub(crate) topology_generation: crate::multiplexer::domain::TopologyGeneration,
    pub(crate) column: u16,
    pub(crate) row: u16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

impl CanonicalTopologyPaneHitRegion {
    fn contains(self, column: u16, row: u16) -> bool {
        column >= self.column
            && row >= self.row
            && column < self.column.saturating_add(self.width)
            && row < self.row.saturating_add(self.height)
    }
}

fn canonical_topology_exact_pane_target(
    presentation: &CanonicalTopologyPresentation,
    hit_regions: &[CanonicalTopologyPaneHitRegion],
    column: u16,
    row: u16,
) -> Option<super::terminal_ux::ExactPaneTarget> {
    let mut matches = hit_regions.iter().copied().filter(|region| {
        region.topology_generation == presentation.topology_generation()
            && region.contains(column, row)
    });
    let region = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(super::terminal_ux::ExactPaneTarget {
        multiplexer_workspace_id: region.multiplexer_workspace_id,
        tab_id: region.tab_id,
        pane_id: region.pane_id,
        topology_generation: region.topology_generation,
    })
}

pub(crate) fn canonical_topology_bind_terminal_interaction(
    presentation: &CanonicalTopologyPresentation,
    hit_regions: &[CanonicalTopologyPaneHitRegion],
    mouse: MouseEvent,
) -> Option<super::terminal_ux::BoundPaneInteraction> {
    use super::terminal_ux::{PaneInteractionKind, PointerPhase};
    let target =
        canonical_topology_exact_pane_target(presentation, hit_regions, mouse.column, mouse.row)?;
    let kind = match mouse.kind {
        MouseEventKind::Down(MouseButton::Right) => PaneInteractionKind::ContextMenu,
        MouseEventKind::Down(button) => PaneInteractionKind::MouseCapture {
            button: map_pointer_button(button),
            phase: PointerPhase::Down,
        },
        MouseEventKind::Up(button) => PaneInteractionKind::MouseCapture {
            button: map_pointer_button(button),
            phase: PointerPhase::Up,
        },
        MouseEventKind::Drag(button) => PaneInteractionKind::MouseCapture {
            button: map_pointer_button(button),
            phase: PointerPhase::Drag,
        },
        MouseEventKind::ScrollUp => PaneInteractionKind::Scroll {
            vertical_lines: -3,
            horizontal_columns: 0,
        },
        MouseEventKind::ScrollDown => PaneInteractionKind::Scroll {
            vertical_lines: 3,
            horizontal_columns: 0,
        },
        MouseEventKind::ScrollLeft => PaneInteractionKind::Scroll {
            vertical_lines: 0,
            horizontal_columns: -3,
        },
        MouseEventKind::ScrollRight => PaneInteractionKind::Scroll {
            vertical_lines: 0,
            horizontal_columns: 3,
        },
        MouseEventKind::Moved => return None,
    };
    super::terminal_ux::bind_exact_pane_interaction(
        target,
        presentation.topology_generation(),
        kind,
        mouse.column,
        mouse.row,
    )
}

fn map_pointer_button(button: MouseButton) -> super::terminal_ux::PointerButton {
    match button {
        MouseButton::Left => super::terminal_ux::PointerButton::Left,
        MouseButton::Middle => super::terminal_ux::PointerButton::Middle,
        MouseButton::Right => super::terminal_ux::PointerButton::Right,
    }
}

fn normalize_canonical_topology_query(value: &str) -> String {
    value.trim().chars().flat_map(char::to_lowercase).collect()
}

fn canonical_topology_match_rank(
    query: &str,
    binding: &CanonicalTopologySearchBinding,
) -> Option<FindRank> {
    let canonical_id = normalize_canonical_topology_query(&binding.canonical_id);
    if canonical_id == query {
        return Some(FindRank::ExactCanonicalId);
    }
    let label = normalize_canonical_topology_query(&binding.display_label);
    if label == query {
        return Some(FindRank::ExactNormalizedLabel);
    }
    if canonical_id.starts_with(query) || label.starts_with(query) {
        return Some(FindRank::NormalizedPrefix);
    }
    (canonical_id.contains(query) || label.contains(query)).then_some(FindRank::NormalizedSubstring)
}

fn canonical_topology_local_mutation_binding(key: KeyEvent) -> bool {
    (key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('n' | 'h' | 'w')))
        || (key.modifiers.contains(KeyModifiers::ALT)
            && matches!(
                key.code,
                KeyCode::Char('v') | KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down
            ))
}

pub(crate) fn canonical_topology_bind_pointer_focus_intent(
    presentation: &CanonicalTopologyPresentation,
    hit_regions: &[CanonicalTopologyPaneHitRegion],
    column: u16,
    row: u16,
) -> Option<CanonicalTopologyBoundIntent> {
    let target = canonical_topology_exact_pane_target(presentation, hit_regions, column, row)?;
    Some(CanonicalTopologyBoundIntent::new(
        crate::persistent_runtime::protocol::ApplyTopologyOperationV2 {
            expected_topology_generation: target.topology_generation,
            operation: crate::persistent_runtime::protocol::TopologyOperationV2::FocusPane {
                multiplexer_workspace_id: target.multiplexer_workspace_id,
                tab_id: target.tab_id,
                pane_id: target.pane_id,
            },
        },
    ))
}

const T174_MAX_DISPLAY_ALIAS_BYTES: usize = 128;
const T174_DETECTION_ONLY_NONCLAIM: &str = "DETECTION_ONLY_UNPROVEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanonicalAgentDockSort {
    Family,
    Recent,
    Source,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalAgentDockItem {
    pub(crate) observation: crate::persistent_runtime::protocol::AgentObservationV2,
    pub(crate) display_alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalAgentDockListItem {
    pub(crate) observation_id: crate::multiplexer::domain::AgentObservationId,
    pub(crate) display_label: String,
    pub(crate) family: crate::persistent_runtime::protocol::AgentFamilyV2,
    pub(crate) source_class: crate::persistent_runtime::protocol::AgentObservationSourceV2,
    pub(crate) freshness: crate::persistent_runtime::protocol::AgentObservationFreshnessV2,
    pub(crate) multiplexer_workspace_id: crate::multiplexer::domain::MultiplexerWorkspaceId,
    pub(crate) tab_id: crate::multiplexer::domain::TabId,
    pub(crate) pane_id: crate::multiplexer::domain::PaneId,
    pub(crate) runtime_namespace_id: Option<crate::persistent_runtime::domain::RuntimeNamespaceId>,
    pub(crate) observed_unix_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CanonicalAgentPaneViewTarget {
    pub(crate) observation_id: crate::multiplexer::domain::AgentObservationId,
    pub(crate) multiplexer_workspace_id: crate::multiplexer::domain::MultiplexerWorkspaceId,
    pub(crate) tab_id: crate::multiplexer::domain::TabId,
    pub(crate) pane_id: crate::multiplexer::domain::PaneId,
    pub(crate) runtime_namespace_id: Option<crate::persistent_runtime::domain::RuntimeNamespaceId>,
    pub(crate) owner_generation_id: crate::persistent_runtime::domain::OwnerGenerationId,
    pub(crate) topology_generation: crate::multiplexer::domain::TopologyGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalAgentDockPresentation {
    items: BTreeMap<crate::multiplexer::domain::AgentObservationId, CanonicalAgentDockItem>,
    selected_observation_id: Option<crate::multiplexer::domain::AgentObservationId>,
    focused_observation_id: Option<crate::multiplexer::domain::AgentObservationId>,
}

impl CanonicalAgentDockPresentation {
    pub(crate) fn from_observations(
        observations: Vec<crate::persistent_runtime::protocol::AgentObservationV2>,
    ) -> std::result::Result<Self, String> {
        let mut items = BTreeMap::new();
        for observation in observations {
            let observation_id = observation.observation_id;
            if items
                .insert(
                    observation_id,
                    CanonicalAgentDockItem {
                        observation,
                        display_alias: None,
                    },
                )
                .is_some()
            {
                return Err(
                    "T174 agent dock received a duplicate immutable observation id".to_owned(),
                );
            }
        }
        Ok(Self {
            items,
            selected_observation_id: None,
            focused_observation_id: None,
        })
    }

    pub(crate) fn from_client(
        client: &crate::persistent_runtime::client::RustLocalControlClient,
    ) -> Option<Self> {
        Self::from_observations(client.trusted_agent_observations()?).ok()
    }

    pub(crate) fn list(
        &self,
        query: &str,
        sort: CanonicalAgentDockSort,
        group_by_workspace: bool,
    ) -> Vec<CanonicalAgentDockListItem> {
        let query = query.trim().to_ascii_lowercase();
        let mut rows = self
            .items
            .values()
            .filter(|item| t174_agent_item_matches(item, &query))
            .map(t174_agent_list_item)
            .collect::<Vec<_>>();
        rows.sort_by(|left, right| {
            let group_order = if group_by_workspace {
                left.multiplexer_workspace_id
                    .cmp(&right.multiplexer_workspace_id)
            } else {
                std::cmp::Ordering::Equal
            };
            if group_order != std::cmp::Ordering::Equal {
                return group_order;
            }
            let primary = match sort {
                CanonicalAgentDockSort::Family => {
                    t174_agent_family_label(left.family).cmp(t174_agent_family_label(right.family))
                }
                CanonicalAgentDockSort::Recent => {
                    right.observed_unix_ms.cmp(&left.observed_unix_ms)
                }
                CanonicalAgentDockSort::Source => t174_agent_source_label(left.source_class)
                    .cmp(t174_agent_source_label(right.source_class)),
            };
            primary.then(left.observation_id.cmp(&right.observation_id))
        });
        rows
    }

    pub(crate) fn get(
        &self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<&CanonicalAgentDockItem> {
        self.items.get(&observation_id)
    }

    pub(crate) fn read(
        &self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<&crate::persistent_runtime::protocol::AgentObservationV2> {
        self.get(observation_id).map(|item| &item.observation)
    }

    pub(crate) fn explain(
        &self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<String> {
        let observation = self.read(observation_id)?;
        Some(format!(
            "observation={}\nfamily={}\nsource={}\nconfidence={}\nfreshness={}\nworkspace={}\ntab={}\npane={}\nruntime={}\nowner_generation={}\nexecution={}\nevidence={}",
            observation.observation_id,
            t174_agent_family_label(observation.family),
            t174_agent_source_label(observation.source_class),
            t174_agent_confidence_label(observation.confidence_class),
            t174_agent_freshness_label(observation.freshness),
            observation.multiplexer_workspace_id,
            observation.tab_id,
            observation.pane_id,
            observation
                .runtime_namespace_id
                .map(|value| value.to_string())
                .unwrap_or_else(|| "UNBOUND".to_owned()),
            observation.owner_generation_id,
            T174_DETECTION_ONLY_NONCLAIM,
            observation.structured_evidence_summary,
        ))
    }

    pub(crate) fn view(
        &mut self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
    ) -> Option<CanonicalAgentDockItem> {
        let item = self.items.get(&observation_id)?.clone();
        self.selected_observation_id = Some(observation_id);
        Some(item)
    }

    pub(crate) fn rename(
        &mut self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
        display_alias: Option<&str>,
    ) -> std::result::Result<(), String> {
        let item = self
            .items
            .get_mut(&observation_id)
            .ok_or_else(|| "T174 agent rename target is absent".to_owned())?;
        let alias = display_alias
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if alias.is_some_and(|value| value.len() > T174_MAX_DISPLAY_ALIAS_BYTES) {
            return Err(format!(
                "T174 display alias exceeds {T174_MAX_DISPLAY_ALIAS_BYTES} bytes"
            ));
        }
        item.display_alias = alias.map(str::to_owned);
        Ok(())
    }

    pub(crate) fn focus(
        &mut self,
        observation_id: crate::multiplexer::domain::AgentObservationId,
        topology: &CanonicalTopologyPresentation,
    ) -> Option<CanonicalAgentPaneViewTarget> {
        let observation = &self.items.get(&observation_id)?.observation;
        if observation.freshness
            != crate::persistent_runtime::protocol::AgentObservationFreshnessV2::Current
        {
            return None;
        }
        let exact_pane_is_present = topology.search_bindings.iter().any(|binding| {
            if binding.intent.expected_topology_generation != topology.topology_generation() {
                return false;
            }
            matches!(
                &binding.intent.operation,
                crate::persistent_runtime::protocol::TopologyOperationV2::FocusPane {
                    multiplexer_workspace_id,
                    tab_id,
                    pane_id,
                } if *multiplexer_workspace_id == observation.multiplexer_workspace_id
                    && *tab_id == observation.tab_id
                    && *pane_id == observation.pane_id
            )
        });
        if !exact_pane_is_present {
            return None;
        }
        let target = CanonicalAgentPaneViewTarget {
            observation_id,
            multiplexer_workspace_id: observation.multiplexer_workspace_id,
            tab_id: observation.tab_id,
            pane_id: observation.pane_id,
            runtime_namespace_id: observation.runtime_namespace_id,
            owner_generation_id: observation.owner_generation_id,
            topology_generation: topology.topology_generation(),
        };
        self.selected_observation_id = Some(observation_id);
        self.focused_observation_id = Some(observation_id);
        Some(target)
    }

    pub(crate) const fn focused_observation_id(
        &self,
    ) -> Option<crate::multiplexer::domain::AgentObservationId> {
        self.focused_observation_id
    }

    pub(crate) fn text(&self) -> String {
        let mut lines = vec![
            "AGENT_DOCK=OWNER_AUTHORITATIVE_AGENT_OBSERVATIONS".to_owned(),
            format!("execution={T174_DETECTION_ONLY_NONCLAIM}"),
        ];
        for row in self.list("", CanonicalAgentDockSort::Family, true) {
            lines.push(format!(
                "{} | family={} | source={} | freshness={} | workspace={} | tab={} | pane={} | observation={}",
                row.display_label,
                t174_agent_family_label(row.family),
                t174_agent_source_label(row.source_class),
                t174_agent_freshness_label(row.freshness),
                row.multiplexer_workspace_id,
                row.tab_id,
                row.pane_id,
                row.observation_id,
            ));
        }
        lines.join("\n")
    }
}

fn t174_agent_list_item(item: &CanonicalAgentDockItem) -> CanonicalAgentDockListItem {
    let observation = &item.observation;
    CanonicalAgentDockListItem {
        observation_id: observation.observation_id,
        display_label: item
            .display_alias
            .clone()
            .unwrap_or_else(|| t174_agent_family_label(observation.family).to_owned()),
        family: observation.family,
        source_class: observation.source_class,
        freshness: observation.freshness,
        multiplexer_workspace_id: observation.multiplexer_workspace_id,
        tab_id: observation.tab_id,
        pane_id: observation.pane_id,
        runtime_namespace_id: observation.runtime_namespace_id,
        observed_unix_ms: observation.observed_unix_ms,
    }
}

fn t174_agent_item_matches(item: &CanonicalAgentDockItem, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let observation = &item.observation;
    [
        item.display_alias.as_deref().unwrap_or_default().to_owned(),
        t174_agent_family_label(observation.family).to_owned(),
        t174_agent_source_label(observation.source_class).to_owned(),
        t174_agent_freshness_label(observation.freshness).to_owned(),
        observation.observation_id.to_string(),
        observation.multiplexer_workspace_id.to_string(),
        observation.tab_id.to_string(),
        observation.pane_id.to_string(),
        observation
            .runtime_namespace_id
            .map(|value| value.to_string())
            .unwrap_or_default(),
        observation
            .provider_native_session_id
            .clone()
            .unwrap_or_default(),
        observation.structured_evidence_summary.clone(),
    ]
    .into_iter()
    .any(|value| value.to_ascii_lowercase().contains(query))
}

fn t174_agent_family_label(
    value: crate::persistent_runtime::protocol::AgentFamilyV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentFamilyV2;
    match value {
        AgentFamilyV2::Pi => "PI",
        AgentFamilyV2::Claude => "CLAUDE",
        AgentFamilyV2::Codex => "CODEX",
        AgentFamilyV2::Gemini => "GEMINI",
        AgentFamilyV2::Cursor => "CURSOR",
        AgentFamilyV2::Devin => "DEVIN",
        AgentFamilyV2::Antigravity => "ANTIGRAVITY",
        AgentFamilyV2::Cline => "CLINE",
        AgentFamilyV2::Omp => "OMP",
        AgentFamilyV2::Mastracode => "MASTRACODE",
        AgentFamilyV2::OpenCode => "OPEN_CODE",
        AgentFamilyV2::GithubCopilot => "GITHUB_COPILOT",
        AgentFamilyV2::Kimi => "KIMI",
        AgentFamilyV2::Kiro => "KIRO",
        AgentFamilyV2::Droid => "DROID",
        AgentFamilyV2::Amp => "AMP",
        AgentFamilyV2::Grok => "GROK",
        AgentFamilyV2::Hermes => "HERMES",
        AgentFamilyV2::Kilo => "KILO",
        AgentFamilyV2::Qodercli => "QODERCLI",
        AgentFamilyV2::Qwen => "QWEN",
        AgentFamilyV2::Letta => "LETTA",
        AgentFamilyV2::Maki => "MAKI",
        AgentFamilyV2::Muse => "MUSE",
    }
}

fn t174_agent_source_label(
    value: crate::persistent_runtime::protocol::AgentObservationSourceV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentObservationSourceV2;
    match value {
        AgentObservationSourceV2::WindsLaunchMetadata => "WINDS_LAUNCH_METADATA",
        AgentObservationSourceV2::OwnedProcessMetadata => "OWNED_PROCESS_METADATA",
        AgentObservationSourceV2::ProviderStructuredMetadata => "PROVIDER_STRUCTURED_METADATA",
        AgentObservationSourceV2::UserDeclaredPresentation => "USER_DECLARED_PRESENTATION",
    }
}

fn t174_agent_confidence_label(
    value: crate::persistent_runtime::protocol::AgentObservationConfidenceV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentObservationConfidenceV2;
    match value {
        AgentObservationConfidenceV2::Exact => "EXACT",
        AgentObservationConfidenceV2::Strong => "STRONG",
        AgentObservationConfidenceV2::UserDeclared => "USER_DECLARED",
        AgentObservationConfidenceV2::Unknown => "UNKNOWN",
    }
}

fn t174_agent_freshness_label(
    value: crate::persistent_runtime::protocol::AgentObservationFreshnessV2,
) -> &'static str {
    use crate::persistent_runtime::protocol::AgentObservationFreshnessV2;
    match value {
        AgentObservationFreshnessV2::Current => "CURRENT",
        AgentObservationFreshnessV2::Ambiguous => "AMBIGUOUS",
        AgentObservationFreshnessV2::Stale => "STALE",
        AgentObservationFreshnessV2::Unavailable => "UNAVAILABLE",
        AgentObservationFreshnessV2::Unknown => "UNKNOWN",
    }
}

#[cfg(test)]
#[path = "t174_agent_dock_tests.rs"]
mod t174_agent_dock_tests;

#[cfg(test)]
mod t168_workbench_topology_binding_tests {
    use super::*;
    use crate::multiplexer::domain::{MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};
    use crate::persistent_runtime::protocol::TopologyOperationV2;

    fn workspace_id(byte: u8) -> MultiplexerWorkspaceId {
        MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).expect("valid workspace id")
    }

    fn tab_id(byte: u8) -> TabId {
        TabId::from_entropy_bytes([byte; 16]).expect("valid tab id")
    }

    fn pane_id(byte: u8) -> PaneId {
        PaneId::from_entropy_bytes([byte; 16]).expect("valid pane id")
    }

    #[test]
    fn t168_workbench_pointer_binding_retains_presented_generation() {
        let workspace_id = workspace_id(1);
        let tab_id = tab_id(2);
        let pane_id = pane_id(3);
        let presented_generation = TopologyGeneration::new(7).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: presented_generation,
            text: "TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let hit_regions = [CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id,
            tab_id,
            pane_id,
            topology_generation: presented_generation,
            column: 10,
            row: 0,
            width: 10,
            height: 10,
        }];

        let intent =
            canonical_topology_bind_pointer_focus_intent(&presentation, &hit_regions, 12, 3)
                .expect("presented exact target should bind")
                .into_intent();
        assert_eq!(intent.expected_topology_generation, presented_generation);
        assert_eq!(
            intent.operation,
            TopologyOperationV2::FocusPane {
                multiplexer_workspace_id: workspace_id,
                tab_id,
                pane_id,
            }
        );
        assert_ne!(
            intent.expected_topology_generation,
            TopologyGeneration::new(8).expect("valid generation")
        );
    }

    #[test]
    fn t168_workbench_pointer_binding_fails_closed_on_overlapping_regions() {
        let presented_generation = TopologyGeneration::new(7).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: presented_generation,
            text: "TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let first = CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id(1),
            tab_id: tab_id(2),
            pane_id: pane_id(3),
            topology_generation: presented_generation,
            column: 0,
            row: 0,
            width: 20,
            height: 10,
        };
        let second = CanonicalTopologyPaneHitRegion {
            pane_id: pane_id(4),
            ..first
        };
        assert!(
            canonical_topology_bind_pointer_focus_intent(&presentation, &[first, second], 5, 5)
                .is_none()
        );
    }

    #[test]
    fn t168_workbench_navigation_routes_canonical_pointer_without_local_focus_mutation() {
        use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let workspace_id = workspace_id(9);
        let tab_id = tab_id(10);
        let pane_id = pane_id(11);
        let presented_generation = TopologyGeneration::new(12).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: presented_generation,
            text: "TOPOLOGY authority=READ_ONLY_TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let region = CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id,
            tab_id,
            pane_id,
            topology_generation: presented_generation,
            column: 10,
            row: 2,
            width: 10,
            height: 4,
        };

        let mut state = crate::workbench::WorkbenchState::new();
        let legacy = state.create_pane(
            "legacy",
            None,
            None,
            crate::workbench::PaneSize::new(80, 24),
        );
        let mut terminals = crate::workbench::terminal::WorkbenchTerminals::new();
        let mut editor = crate::workbench::terminal::input::WorkbenchShellEditor::new();
        let mut navigation = WorkbenchNavigation::new();
        navigation.install_canonical_topology(presentation, vec![region]);

        let effect = navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 12,
                    row: 3,
                    modifiers: KeyModifiers::NONE,
                }),
            )
            .expect("canonical pointer navigation must be handled");
        let NavigationEffect::CanonicalTopologyIntent(bound) = effect else {
            panic!("canonical pointer must return a bound topology intent");
        };
        let intent = bound.into_intent();
        assert_eq!(intent.expected_topology_generation, presented_generation);
        assert_eq!(
            intent.operation,
            TopologyOperationV2::FocusPane {
                multiplexer_workspace_id: workspace_id,
                tab_id,
                pane_id,
            }
        );
        assert_eq!(state.selected_pane(), Some(legacy));
    }

    #[test]
    fn t171_copy_on_select_retains_pointer_down_target_across_focus_race() {
        use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let generation = TopologyGeneration::new(12).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: generation,
            text: "TOPOLOGY authority=READ_ONLY_TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let first = CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id(9),
            tab_id: tab_id(10),
            pane_id: pane_id(11),
            topology_generation: generation,
            column: 0,
            row: 0,
            width: 10,
            height: 5,
        };
        let second = CanonicalTopologyPaneHitRegion {
            pane_id: pane_id(12),
            column: 10,
            ..first
        };
        let mut state = crate::workbench::WorkbenchState::new();
        let mut terminals = crate::workbench::terminal::WorkbenchTerminals::new();
        let mut editor = crate::workbench::terminal::input::WorkbenchShellEditor::new();
        let mut navigation = WorkbenchNavigation::new();
        navigation.install_canonical_topology(presentation, vec![first, second]);

        let down = navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 2,
                    row: 2,
                    modifiers: KeyModifiers::NONE,
                }),
            )
            .unwrap();
        assert!(matches!(down, NavigationEffect::CanonicalTopologyIntent(_)));

        let up = navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Up(MouseButton::Left),
                    column: 12,
                    row: 2,
                    modifiers: KeyModifiers::NONE,
                }),
            )
            .unwrap();
        let NavigationEffect::CanonicalPaneInteraction(click) = up else {
            panic!("a release must return a bound pane interaction");
        };
        assert_eq!(click.target().pane_id, first.pane_id);
        assert_eq!(
            click.kind(),
            super::super::terminal_ux::PaneInteractionKind::MouseCapture {
                button: super::super::terminal_ux::PointerButton::Left,
                phase: super::super::terminal_ux::PointerPhase::Up,
            }
        );
    }

    #[test]
    fn t171_drag_release_copies_on_select_from_pointer_down_pane() {
        use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let generation = TopologyGeneration::new(12).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: generation,
            text: "TOPOLOGY authority=READ_ONLY_TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let first = CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id(9),
            tab_id: tab_id(10),
            pane_id: pane_id(11),
            topology_generation: generation,
            column: 0,
            row: 0,
            width: 10,
            height: 5,
        };
        let second = CanonicalTopologyPaneHitRegion {
            pane_id: pane_id(12),
            column: 10,
            ..first
        };
        let mut state = crate::workbench::WorkbenchState::new();
        let mut terminals = crate::workbench::terminal::WorkbenchTerminals::new();
        let mut editor = crate::workbench::terminal::input::WorkbenchShellEditor::new();
        let mut navigation = WorkbenchNavigation::new();
        navigation.install_canonical_topology(presentation, vec![first, second]);

        let mut send = |navigation: &mut WorkbenchNavigation, kind, column| {
            navigation
                .handle_event(
                    &mut state,
                    &mut terminals,
                    &mut editor,
                    &[],
                    &[],
                    Event::Mouse(MouseEvent {
                        kind,
                        column,
                        row: 2,
                        modifiers: KeyModifiers::NONE,
                    }),
                )
                .unwrap()
        };

        assert!(matches!(
            send(&mut navigation, MouseEventKind::Down(MouseButton::Left), 2),
            NavigationEffect::CanonicalTopologyIntent(_)
        ));
        assert!(matches!(
            send(&mut navigation, MouseEventKind::Drag(MouseButton::Left), 6),
            NavigationEffect::CanonicalPaneInteraction(_)
        ));

        // Focus moves to the second pane before the release; the action must still
        // resolve to the pane the gesture started on.
        let up = send(&mut navigation, MouseEventKind::Up(MouseButton::Left), 12);
        let NavigationEffect::CanonicalPaneInteraction(copy) = up else {
            panic!("copy-on-select must retain the pointer-down pane target");
        };
        assert_eq!(copy.target().pane_id, first.pane_id);
        assert_eq!(
            copy.kind(),
            super::super::terminal_ux::PaneInteractionKind::CopyOnSelect
        );
    }

    #[test]
    fn t171_pointer_capture_fails_closed_when_generation_advances_mid_gesture() {
        use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let generation = TopologyGeneration::new(12).expect("valid generation");
        let next = TopologyGeneration::new(13).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: generation,
            text: "TOPOLOGY authority=READ_ONLY_TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let region = CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id(9),
            tab_id: tab_id(10),
            pane_id: pane_id(11),
            topology_generation: generation,
            column: 0,
            row: 0,
            width: 10,
            height: 5,
        };
        let mut state = crate::workbench::WorkbenchState::new();
        let mut terminals = crate::workbench::terminal::WorkbenchTerminals::new();
        let mut editor = crate::workbench::terminal::input::WorkbenchShellEditor::new();
        let mut navigation = WorkbenchNavigation::new();
        navigation.install_canonical_topology(presentation.clone(), vec![region]);
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 2,
                    row: 2,
                    modifiers: KeyModifiers::NONE,
                }),
            )
            .unwrap();

        let advanced = CanonicalTopologyPresentation {
            topology_generation: next,
            text: presentation.text,
            search_bindings: Vec::new(),
        };
        navigation.install_canonical_topology(advanced, Vec::new());

        let up = navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Up(MouseButton::Left),
                    column: 2,
                    row: 2,
                    modifiers: KeyModifiers::NONE,
                }),
            )
            .unwrap();
        assert_eq!(up, NavigationEffect::None);
    }

    #[test]
    fn t171_scroll_and_right_click_bind_exact_pane_and_current_generation() {
        use crate::workbench::terminal_ux::PaneInteractionKind;
        use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

        let generation = TopologyGeneration::new(4).expect("valid generation");
        let presentation = CanonicalTopologyPresentation {
            topology_generation: generation,
            text: "TOPOLOGY authority=READ_ONLY_TOPOLOGY".to_owned(),
            search_bindings: Vec::new(),
        };
        let first = CanonicalTopologyPaneHitRegion {
            multiplexer_workspace_id: workspace_id(1),
            tab_id: tab_id(2),
            pane_id: pane_id(3),
            topology_generation: generation,
            column: 0,
            row: 0,
            width: 10,
            height: 5,
        };
        let second = CanonicalTopologyPaneHitRegion {
            pane_id: pane_id(4),
            column: 10,
            ..first
        };
        let regions = [first, second];

        let mut state = crate::workbench::WorkbenchState::new();
        let legacy = state.create_pane(
            "legacy",
            None,
            None,
            crate::workbench::PaneSize::new(80, 24),
        );
        let mut terminals = crate::workbench::terminal::WorkbenchTerminals::new();
        let mut editor = crate::workbench::terminal::input::WorkbenchShellEditor::new();
        let mut navigation = WorkbenchNavigation::new();
        navigation.install_canonical_topology(presentation, regions.to_vec());

        state.focus_pane(legacy);
        assert_eq!(state.selected_pane(), Some(legacy));

        let mut send = |navigation: &mut WorkbenchNavigation, kind| {
            navigation
                .handle_event(
                    &mut state,
                    &mut terminals,
                    &mut editor,
                    &[],
                    &[],
                    Event::Mouse(MouseEvent {
                        kind,
                        column: 12,
                        row: 2,
                        modifiers: KeyModifiers::NONE,
                    }),
                )
                .unwrap()
        };

        // The focused legacy pane is not the pane under the pointer, so both the
        // right click and the scroll prove the target is the exact hit pane and
        // never the focused pane.
        let right = send(&mut navigation, MouseEventKind::Down(MouseButton::Right));
        let NavigationEffect::CanonicalPaneInteraction(context) = right else {
            panic!("right click must return a bound pane interaction");
        };
        assert_eq!(context.target().pane_id, second.pane_id);
        assert_eq!(context.kind(), PaneInteractionKind::ContextMenu);

        let scrolled = send(&mut navigation, MouseEventKind::ScrollUp);
        let NavigationEffect::CanonicalPaneInteraction(scroll) = scrolled else {
            panic!("scroll must return a bound pane interaction");
        };
        assert_eq!(scroll.target().pane_id, second.pane_id);
        assert_eq!(
            scroll.kind(),
            PaneInteractionKind::Scroll {
                vertical_lines: -3,
                horizontal_columns: 0,
            }
        );
        assert_eq!(state.selected_pane(), Some(legacy));

        let stale_regions = [CanonicalTopologyPaneHitRegion {
            topology_generation: TopologyGeneration::new(5).expect("valid generation"),
            ..first
        }];
        let mut stale_navigation = WorkbenchNavigation::new();
        stale_navigation.install_canonical_topology(
            CanonicalTopologyPresentation {
                topology_generation: TopologyGeneration::new(4).expect("valid generation"),
                text: "TOPOLOGY authority=READ_ONLY_TOPOLOGY".to_owned(),
                search_bindings: Vec::new(),
            },
            stale_regions.to_vec(),
        );
        let stale = stale_navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::ScrollDown,
                    column: 2,
                    row: 1,
                    modifiers: KeyModifiers::NONE,
                }),
            )
            .unwrap();
        assert_eq!(stale, NavigationEffect::None);
    }
}

#[cfg(test)]
#[path = "t092_workbench_navigation_tests.rs"]
mod t092_workbench_navigation_tests;
