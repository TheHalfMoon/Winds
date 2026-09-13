use crate::agentic_runtime::{
    RuntimeIdentityRevalidation, RuntimeKind, revalidate_runtime_identity,
};
use crate::domain::workflow::{
    RETRY_OUTCOME_AMBIGUOUS_EFFECT, RETRY_OUTCOME_BUDGET_EXHAUSTED, RETRY_OUTCOME_FAILURE_RECORDED,
    RETRY_OUTCOME_NO_PROGRESS, StageLifecycleState, TruthSource,
};
use crate::domain::{WindsSessionRecord, WorkspaceRecord};
use crate::store::{
    DesktopLayoutPresentation, DesktopLayoutPresentationInput, DesktopProjectPresentationInput,
    DesktopSessionPresentationInput, NewWindsSession, Result, Store,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DesktopRuntimeFamily {
    Codex,
    Claude,
}

impl From<RuntimeKind> for DesktopRuntimeFamily {
    fn from(value: RuntimeKind) -> Self {
        match value {
            RuntimeKind::Codex => Self::Codex,
            RuntimeKind::Claude => Self::Claude,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DesktopRuntimeState {
    Requested,
    Observed,
    Mismatch,
    Unknown,
    Unavailable,
    Stale,
    Conflicting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopRuntimeProjection {
    pub(crate) state: DesktopRuntimeState,
    pub(crate) requested: Vec<DesktopRuntimeFamily>,
    pub(crate) observed: Vec<DesktopRuntimeFamily>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DesktopAttentionState {
    RecoveryRequired,
    WaitingApproval,
    RetryRequired,
    WaitingExternal,
    Stale,
    Unknown,
    None,
}

impl DesktopAttentionState {
    fn rank(self) -> u8 {
        match self {
            Self::RecoveryRequired => 0,
            Self::WaitingApproval => 1,
            Self::RetryRequired => 2,
            Self::WaitingExternal => 3,
            Self::Stale => 4,
            Self::Unknown => 5,
            Self::None => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopProjectSummary {
    pub(crate) project_view_id: String,
    pub(crate) display_name: String,
    pub(crate) canonical_workspace_id: String,
    pub(crate) canonical_repo_root: String,
    pub(crate) canonical_git_common_dir: String,
    pub(crate) pinned: bool,
    pub(crate) presentation_order: i64,
    pub(crate) collapsed: bool,
    pub(crate) presentation_revision: Option<i64>,
    pub(crate) session_count: usize,
    pub(crate) attention_count: usize,
    pub(crate) attention: DesktopAttentionState,
    pub(crate) search_input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopSessionSummary {
    pub(crate) canonical_session_id: String,
    pub(crate) canonical_workstream_id: String,
    pub(crate) canonical_workspace_id: String,
    pub(crate) display_name: String,
    pub(crate) canonical_display_name: String,
    pub(crate) pinned: bool,
    pub(crate) presentation_order: i64,
    pub(crate) archived: bool,
    pub(crate) presentation_revision: Option<i64>,
    pub(crate) runtime: DesktopRuntimeProjection,
    pub(crate) attention: DesktopAttentionState,
    pub(crate) search_input: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopProjectPresentationCommand {
    pub(crate) workspace_id: String,
    pub(crate) display_name: String,
    pub(crate) pinned: bool,
    pub(crate) sort_order: i64,
    pub(crate) collapsed: bool,
    pub(crate) expected_revision: i64,
    pub(crate) now_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopSessionPresentationCommand {
    pub(crate) session_id: String,
    pub(crate) display_alias: String,
    pub(crate) pinned: bool,
    pub(crate) sort_order: i64,
    pub(crate) archived: bool,
    pub(crate) expected_revision: Option<i64>,
    pub(crate) now_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopLayoutCommand {
    pub(crate) workspace_id: String,
    pub(crate) layout_mode: String,
    pub(crate) left_session_id: Option<String>,
    pub(crate) right_session_id: Option<String>,
    pub(crate) split_basis_points: i64,
    pub(crate) right_dock_surface: String,
    pub(crate) right_dock_binding: String,
    pub(crate) left_dock_collapsed: bool,
    pub(crate) left_dock_width_px: i64,
    pub(crate) right_dock_collapsed: bool,
    pub(crate) right_dock_width_px: i64,
    pub(crate) appearance: String,
    pub(crate) contrast: String,
    pub(crate) density: String,
    pub(crate) reduced_motion: bool,
}

pub(crate) struct DesktopFacade<'a> {
    store: &'a Store,
}

impl<'a> DesktopFacade<'a> {
    pub(crate) fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub(crate) fn list_projects(&self) -> Result<Vec<DesktopProjectSummary>> {
        let mut projects = self
            .store
            .list_desktop_workspaces()?
            .into_iter()
            .map(|workspace| self.project_summary(workspace))
            .collect::<Result<Vec<_>>>()?;
        projects.sort_by(|left, right| {
            right
                .pinned
                .cmp(&left.pinned)
                .then(left.presentation_order.cmp(&right.presentation_order))
                .then(
                    left.display_name
                        .to_ascii_lowercase()
                        .cmp(&right.display_name.to_ascii_lowercase()),
                )
                .then(
                    left.canonical_workspace_id
                        .cmp(&right.canonical_workspace_id),
                )
        });
        Ok(projects)
    }

    pub(crate) fn open_project(&self, workspace_id: &str) -> Result<DesktopProjectSummary> {
        self.project_summary(self.store.load_workspace(workspace_id)?)
    }

    pub(crate) fn create_project_context(
        &self,
        workspace_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<DesktopProjectSummary> {
        self.store.load_workspace(workspace_id)?;
        if self
            .store
            .load_desktop_project_presentation(workspace_id)?
            .is_some()
        {
            return Err(format!("desktop project context already exists: {workspace_id}").into());
        }
        self.store.save_desktop_project_presentation(
            DesktopProjectPresentationInput {
                workspace_id,
                display_name,
                pinned: false,
                sort_order: 0,
                collapsed: false,
            },
            None,
            now_ms,
        )?;
        self.open_project(workspace_id)
    }

    pub(crate) fn update_project_presentation(
        &self,
        command: &DesktopProjectPresentationCommand,
    ) -> Result<DesktopProjectSummary> {
        self.store.save_desktop_project_presentation(
            DesktopProjectPresentationInput {
                workspace_id: &command.workspace_id,
                display_name: &command.display_name,
                pinned: command.pinned,
                sort_order: command.sort_order,
                collapsed: command.collapsed,
            },
            Some(command.expected_revision),
            command.now_ms,
        )?;
        self.open_project(&command.workspace_id)
    }

    pub(crate) fn list_sessions(&self, workspace_id: &str) -> Result<Vec<DesktopSessionSummary>> {
        self.store.load_workspace(workspace_id)?;
        let mut sessions = Vec::new();
        for workstream in self.store.list_workstreams(workspace_id)? {
            for session in self.store.list_winds_sessions(&workstream.workstream_id)? {
                sessions.push(self.session_summary(session)?);
            }
        }
        sessions.sort_by(|left, right| {
            left.archived
                .cmp(&right.archived)
                .then(right.pinned.cmp(&left.pinned))
                .then(left.presentation_order.cmp(&right.presentation_order))
                .then(
                    left.display_name
                        .to_ascii_lowercase()
                        .cmp(&right.display_name.to_ascii_lowercase()),
                )
                .then(left.canonical_session_id.cmp(&right.canonical_session_id))
        });
        Ok(sessions)
    }

    pub(crate) fn create_session(
        &self,
        workspace_id: &str,
        workstream_id: &str,
        session_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<DesktopSessionSummary> {
        self.store.load_workspace(workspace_id)?;
        let workstream = self.store.load_workstream(workstream_id)?;
        if workstream.workspace_id != workspace_id {
            return Err(
                "desktop session workstream does not belong to the selected project".into(),
            );
        }
        self.store.create_winds_session(
            NewWindsSession {
                session_id,
                workstream_id,
                display_name,
            },
            now_ms,
        )?;
        self.session_summary(self.store.load_winds_session(session_id)?)
    }

    pub(crate) fn rename_session(
        &self,
        session_id: &str,
        display_name: &str,
        now_ms: i64,
    ) -> Result<DesktopSessionSummary> {
        self.store
            .rename_winds_session(session_id, display_name, now_ms)?;
        self.session_summary(self.store.load_winds_session(session_id)?)
    }

    pub(crate) fn update_session_presentation(
        &self,
        command: &DesktopSessionPresentationCommand,
    ) -> Result<DesktopSessionSummary> {
        if command.expected_revision.is_none()
            && self
                .store
                .load_desktop_session_presentation(&command.session_id)?
                .is_some()
        {
            return Err(format!(
                "desktop session presentation already exists: {}",
                command.session_id
            )
            .into());
        }
        self.store.save_desktop_session_presentation(
            DesktopSessionPresentationInput {
                session_id: &command.session_id,
                display_alias: &command.display_alias,
                pinned: command.pinned,
                sort_order: command.sort_order,
                archived: command.archived,
            },
            command.expected_revision,
            command.now_ms,
        )?;
        self.session_summary(self.store.load_winds_session(&command.session_id)?)
    }

    pub(crate) fn load_layout(
        &self,
        workspace_id: &str,
    ) -> Result<Option<DesktopLayoutPresentation>> {
        self.store.load_workspace(workspace_id)?;
        self.store.load_desktop_layout_presentation(workspace_id)
    }

    pub(crate) fn save_layout(
        &self,
        command: &DesktopLayoutCommand,
        expected_revision: Option<i64>,
        now_ms: i64,
    ) -> Result<DesktopLayoutPresentation> {
        self.store.load_workspace(&command.workspace_id)?;
        for session_id in [
            command.left_session_id.as_deref(),
            command.right_session_id.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            let session = self.store.load_winds_session(session_id)?;
            let workstream = self.store.load_workstream(&session.workstream_id)?;
            if workstream.workspace_id != command.workspace_id {
                return Err(
                    "desktop layout session does not belong to the selected project".into(),
                );
            }
        }
        self.store.save_desktop_layout_presentation(
            DesktopLayoutPresentationInput {
                workspace_id: &command.workspace_id,
                layout_mode: &command.layout_mode,
                left_session_id: command.left_session_id.as_deref(),
                right_session_id: command.right_session_id.as_deref(),
                split_basis_points: command.split_basis_points,
                right_dock_surface: &command.right_dock_surface,
                right_dock_binding: &command.right_dock_binding,
                left_dock_collapsed: command.left_dock_collapsed,
                left_dock_width_px: command.left_dock_width_px,
                right_dock_collapsed: command.right_dock_collapsed,
                right_dock_width_px: command.right_dock_width_px,
                appearance: &command.appearance,
                contrast: &command.contrast,
                density: &command.density,
                reduced_motion: command.reduced_motion,
            },
            expected_revision,
            now_ms,
        )
    }

    fn project_summary(&self, workspace: WorkspaceRecord) -> Result<DesktopProjectSummary> {
        let presentation = self
            .store
            .load_desktop_project_presentation(&workspace.workspace_id)?;
        let sessions = self.list_sessions(&workspace.workspace_id)?;
        let attention_count = sessions
            .iter()
            .filter(|session| session.attention != DesktopAttentionState::None)
            .count();
        let attention = sessions
            .iter()
            .map(|session| session.attention)
            .min_by_key(|state| state.rank())
            .unwrap_or(DesktopAttentionState::None);
        let display_name = presentation
            .as_ref()
            .map(|value| value.display_name.clone())
            .unwrap_or_else(|| workspace.workspace_id.clone());
        let search_input = normalized_search_input(&[
            &display_name,
            &workspace.workspace_id,
            &workspace.canonical_worktree_root,
            &workspace.git_common_dir,
        ]);
        Ok(DesktopProjectSummary {
            project_view_id: workspace.workspace_id.clone(),
            display_name,
            canonical_workspace_id: workspace.workspace_id,
            canonical_repo_root: workspace.canonical_worktree_root,
            canonical_git_common_dir: workspace.git_common_dir,
            pinned: presentation.as_ref().is_some_and(|value| value.pinned),
            presentation_order: presentation.as_ref().map_or(0, |value| value.sort_order),
            collapsed: presentation.as_ref().is_some_and(|value| value.collapsed),
            presentation_revision: presentation.as_ref().map(|value| value.revision),
            session_count: sessions.len(),
            attention_count,
            attention,
            search_input,
        })
    }

    fn session_summary(&self, session: WindsSessionRecord) -> Result<DesktopSessionSummary> {
        let workstream = self.store.load_workstream(&session.workstream_id)?;
        let presentation = self
            .store
            .load_desktop_session_presentation(&session.session_id)?;
        let display_name = presentation
            .as_ref()
            .map(|value| value.display_alias.clone())
            .unwrap_or_else(|| session.display_name.clone());
        let runtime = self.runtime_projection(&session.session_id)?;
        let attention = self.attention_projection(&session.session_id)?;
        let search_input = normalized_search_input(&[
            &display_name,
            &session.display_name,
            &session.session_id,
            &session.workstream_id,
            &workstream.workspace_id,
        ]);
        Ok(DesktopSessionSummary {
            canonical_session_id: session.session_id,
            canonical_workstream_id: session.workstream_id,
            canonical_workspace_id: workstream.workspace_id,
            display_name,
            canonical_display_name: session.display_name,
            pinned: presentation.as_ref().is_some_and(|value| value.pinned),
            presentation_order: presentation.as_ref().map_or(0, |value| value.sort_order),
            archived: presentation.as_ref().is_some_and(|value| value.archived),
            presentation_revision: presentation.as_ref().map(|value| value.revision),
            runtime,
            attention,
            search_input,
        })
    }

    fn runtime_projection(&self, session_id: &str) -> Result<DesktopRuntimeProjection> {
        let requested = self
            .store
            .desktop_latest_requested_runtimes(session_id)?
            .into_iter()
            .map(DesktopRuntimeFamily::from)
            .collect::<Vec<_>>();
        let mut latest_bindings = Vec::new();
        for runtime in [RuntimeKind::Codex, RuntimeKind::Claude] {
            latest_bindings.extend(
                self.store
                    .list_runtime_session_bindings(session_id, runtime)?
                    .into_iter()
                    .last(),
            );
        }
        let latest_time = latest_bindings
            .iter()
            .map(|value| value.bound_unix_ms)
            .max();
        latest_bindings.retain(|value| Some(value.bound_unix_ms) == latest_time);
        let mut observed = latest_bindings
            .iter()
            .map(|value| DesktopRuntimeFamily::from(value.runtime))
            .collect::<Vec<_>>();
        observed.sort();
        observed.dedup();
        let mut stale = false;
        let mut unavailable = false;
        for binding in &latest_bindings {
            match revalidate_runtime_identity(&binding.executable)? {
                RuntimeIdentityRevalidation::Match => {}
                RuntimeIdentityRevalidation::Changed => stale = true,
                RuntimeIdentityRevalidation::Unavailable => unavailable = true,
            }
        }
        let state = classify_runtime_state(&requested, &observed, stale, unavailable);
        Ok(DesktopRuntimeProjection {
            state,
            requested,
            observed,
        })
    }

    fn attention_projection(&self, session_id: &str) -> Result<DesktopAttentionState> {
        let mut state = DesktopAttentionState::None;
        for fact in self.store.desktop_attention_facts(session_id)? {
            let trusted = matches!(
                fact.source,
                Some(TruthSource::WindsObserved | TruthSource::HumanDecided)
            );
            let candidate = if !trusted {
                DesktopAttentionState::Unknown
            } else {
                match fact.lifecycle_state {
                    StageLifecycleState::WaitingApproval => DesktopAttentionState::WaitingApproval,
                    StageLifecycleState::WaitingExternal => DesktopAttentionState::WaitingExternal,
                    StageLifecycleState::RecoveryRequired => {
                        DesktopAttentionState::RecoveryRequired
                    }
                    StageLifecycleState::Stale => DesktopAttentionState::Stale,
                    StageLifecycleState::Blocked => DesktopAttentionState::Unknown,
                    StageLifecycleState::Failed => match fact.outcome_reason.as_deref() {
                        Some(RETRY_OUTCOME_BUDGET_EXHAUSTED | RETRY_OUTCOME_AMBIGUOUS_EFFECT) => {
                            DesktopAttentionState::RecoveryRequired
                        }
                        Some(RETRY_OUTCOME_FAILURE_RECORDED | RETRY_OUTCOME_NO_PROGRESS) => {
                            DesktopAttentionState::RetryRequired
                        }
                        _ => DesktopAttentionState::Unknown,
                    },
                    StageLifecycleState::Prepared
                    | StageLifecycleState::Active
                    | StageLifecycleState::Cancelled
                    | StageLifecycleState::Completed => DesktopAttentionState::None,
                }
            };
            if candidate.rank() < state.rank() {
                state = candidate;
            }
        }
        Ok(state)
    }
}

fn normalized_search_input(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn classify_runtime_state(
    requested: &[DesktopRuntimeFamily],
    observed: &[DesktopRuntimeFamily],
    stale: bool,
    unavailable: bool,
) -> DesktopRuntimeState {
    if requested.len() > 1 || observed.len() > 1 {
        DesktopRuntimeState::Conflicting
    } else if stale {
        DesktopRuntimeState::Stale
    } else if unavailable {
        DesktopRuntimeState::Unavailable
    } else if !requested.is_empty() && !observed.is_empty() && requested != observed {
        DesktopRuntimeState::Mismatch
    } else if !observed.is_empty() {
        DesktopRuntimeState::Observed
    } else if !requested.is_empty() {
        DesktopRuntimeState::Requested
    } else {
        DesktopRuntimeState::Unknown
    }
}
