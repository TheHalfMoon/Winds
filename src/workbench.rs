use crate::git::Repo;
use crate::git::shell_profiles::discover_native_shell_profiles;
use crate::git::workspace::open_existing_workspace;
use crate::git::workspace_inventory::inventory_workspace_environment;
use crossterm::event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture};
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

fn render_workbench(
    frame: &mut Frame<'_>,
    state: &WorkbenchState,
    editor: &terminal::input::WorkbenchShellEditor,
) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("Ctrl+Q quit | Enter submit | Ctrl+F find | Alt+Left/Right focus"),
        areas[0],
    );

    let pane_text = if state.panes().is_empty() {
        EMPTY_WORKBENCH_MESSAGE.to_owned()
    } else {
        state
            .panes()
            .iter()
            .map(|pane| {
                let selected = if state.selected_pane() == Some(pane.pane_id) {
                    ">"
                } else {
                    " "
                };
                format!(
                    "{selected} {} [{}] workspace={} session={}",
                    pane.display_title,
                    pane.lifecycle.label(),
                    pane.canonical_workspace_id.as_deref().unwrap_or("UNKNOWN"),
                    pane.canonical_winds_session_id.as_deref().unwrap_or("UNBOUND")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    frame.render_widget(
        Paragraph::new(pane_text).block(Block::bordered().title(" Panes ")),
        areas[1],
    );

    let input = editor.lines().join("\n");
    frame.render_widget(
        Paragraph::new(input).block(Block::bordered().title(" Shell input ")),
        areas[2],
    );
}

pub(crate) fn run_cli(args: Vec<String>) -> crate::Result<()> {
    let (repo_path, exit_after_ready) = parse_workbench_args(&args)?;
    if exit_after_ready && std::env::var("WINDS_T097_BENCHMARK").as_deref() != Ok("1") {
        return Err(
            "--t097-exit-after-ready is restricted to WINDS_T097_BENCHMARK=1".into(),
        );
    }

    let requested_repo = match repo_path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    let repo = Repo::open(&requested_repo)?;
    let home = crate::winds_home(None, &repo)?;
    let workspace = open_existing_workspace(repo.root(), &home, crate::unix_ms()?)?;
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
    let mut editor = terminal::input::WorkbenchShellEditor::new();
    let mut navigation = ui::WorkbenchNavigation::new();

    let mut host_guard = HostTerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut host_terminal = Terminal::new(backend)?;
    host_terminal.draw(|frame| render_workbench(frame, &state, &editor))?;

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
    let loop_result = ui::run_host_event_loop(
        &mut navigation,
        &mut state,
        &mut terminals,
        &mut editor,
        (&[], &[]),
        &mut source,
        |state, _navigation, editor| {
            host_terminal.draw(|frame| render_workbench(frame, state, editor))?;
            Ok(())
        },
    );
    let cleanup = close_all_workbench_panes(&mut terminals, &mut state);
    drop(host_terminal);
    let restore = host_guard.restore();

    loop_result?;
    cleanup?;
    restore?;
    Ok(())
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
                let value = args
                    .get(index + 1)
                    .ok_or("missing value for --repo")?;
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
#[path = "workbench_screen.rs"]
pub(crate) mod screen;
#[path = "workbench_terminal.rs"]
pub(crate) mod terminal;
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
