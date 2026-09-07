use super::terminal::WorkbenchTerminals;
use super::terminal::input::{
    MultilineSubmitPolicy, ShellCursorMove, ShellDispatchReceipt, ShellSubmitTerminator,
    WorkbenchShellEditor,
};
use super::{PaneId, PaneLifecycleView, PaneSize, SplitAxis, WorkbenchState};
use crate::domain::{WindsSessionRecord, WorkspaceRecord};
use crate::git::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
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
}

#[derive(Debug, Default)]
pub(crate) struct WorkbenchNavigation {
    search_query: Option<String>,
    last_find: Option<FindResolution>,
    selected_canonical_target: Option<NavigationTarget>,
    hit_regions: Vec<PaneHitRegion>,
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

        if self.search_query.is_some() {
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
                let size = PaneSize::new(columns.max(1), rows.max(1));
                resize_selected(state, terminals, size)?;
                Ok(NavigationEffect::None)
            }
            Event::FocusGained | Event::FocusLost => Ok(NavigationEffect::None),
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
                if let FindResolution::Unique(found) = &resolution {
                    match &found.target {
                        NavigationTarget::Pane(pane_id) => {
                            state.focus_pane(*pane_id);
                            self.selected_canonical_target = None;
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
                KeyCode::Char('j') => {
                    split_selected(state, SplitAxis::Vertical)?;
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

#[cfg(test)]
#[path = "t092_workbench_navigation_tests.rs"]
mod t092_workbench_navigation_tests;
