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
    pub(crate) display_alias: Option<String>,
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
        self.create_project_presentation(workspace_id, display_name, false, 0, false, now_ms)
    }

    fn create_project_presentation(
        &self,
        workspace_id: &str,
        display_name: &str,
        pinned: bool,
        sort_order: i64,
        collapsed: bool,
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
                pinned,
                sort_order,
                collapsed,
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
        let sessions = self.list_sessions(&workspace.workspace_id)?;
        self.project_summary_from_sessions(workspace, &sessions)
    }

    fn project_summary_from_sessions(
        &self,
        workspace: WorkspaceRecord,
        sessions: &[DesktopSessionSummary],
    ) -> Result<DesktopProjectSummary> {
        let presentation = self
            .store
            .load_desktop_project_presentation(&workspace.workspace_id)?;
        let attention_count = sessions
            .iter()
            .filter(|session| {
                !matches!(
                    session.attention,
                    DesktopAttentionState::Unknown | DesktopAttentionState::None
                )
            })
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
            display_alias: presentation
                .as_ref()
                .map(|value| value.display_alias.clone()),
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
        let mut requested = self
            .store
            .desktop_latest_requested_runtimes(session_id)?
            .into_iter()
            .map(DesktopRuntimeFamily::from)
            .collect::<Vec<_>>();
        requested.sort();
        requested.dedup();
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
            let candidate = if fact.source.is_none()
                && fact.lifecycle_state == StageLifecycleState::Prepared
            {
                DesktopAttentionState::None
            } else if !trusted {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopBridgeRuntimeFamily {
    Codex,
    Claude,
}

impl From<DesktopRuntimeFamily> for DesktopBridgeRuntimeFamily {
    fn from(value: DesktopRuntimeFamily) -> Self {
        match value {
            DesktopRuntimeFamily::Codex => Self::Codex,
            DesktopRuntimeFamily::Claude => Self::Claude,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopBridgeRuntimeState {
    Requested,
    Observed,
    Mismatch,
    Unknown,
    Unavailable,
    Stale,
    Conflicting,
}

impl From<DesktopRuntimeState> for DesktopBridgeRuntimeState {
    fn from(value: DesktopRuntimeState) -> Self {
        match value {
            DesktopRuntimeState::Requested => Self::Requested,
            DesktopRuntimeState::Observed => Self::Observed,
            DesktopRuntimeState::Mismatch => Self::Mismatch,
            DesktopRuntimeState::Unknown => Self::Unknown,
            DesktopRuntimeState::Unavailable => Self::Unavailable,
            DesktopRuntimeState::Stale => Self::Stale,
            DesktopRuntimeState::Conflicting => Self::Conflicting,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopBridgeAttentionState {
    RecoveryRequired,
    WaitingApproval,
    RetryRequired,
    WaitingExternal,
    Stale,
    Unknown,
    None,
}

impl From<DesktopAttentionState> for DesktopBridgeAttentionState {
    fn from(value: DesktopAttentionState) -> Self {
        match value {
            DesktopAttentionState::RecoveryRequired => Self::RecoveryRequired,
            DesktopAttentionState::WaitingApproval => Self::WaitingApproval,
            DesktopAttentionState::RetryRequired => Self::RetryRequired,
            DesktopAttentionState::WaitingExternal => Self::WaitingExternal,
            DesktopAttentionState::Stale => Self::Stale,
            DesktopAttentionState::Unknown => Self::Unknown,
            DesktopAttentionState::None => Self::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeRuntimeProjection {
    pub state: DesktopBridgeRuntimeState,
    pub requested: Vec<DesktopBridgeRuntimeFamily>,
    pub observed: Vec<DesktopBridgeRuntimeFamily>,
}

impl From<DesktopRuntimeProjection> for DesktopBridgeRuntimeProjection {
    fn from(value: DesktopRuntimeProjection) -> Self {
        Self {
            state: value.state.into(),
            requested: value.requested.into_iter().map(Into::into).collect(),
            observed: value.observed.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeProjectSummary {
    pub project_view_id: String,
    pub display_name: String,
    pub canonical_workspace_id: String,
    pub canonical_repo_root: String,
    pub canonical_git_common_dir: String,
    pub pinned: bool,
    pub presentation_order: i64,
    pub collapsed: bool,
    pub presentation_revision: Option<i64>,
    pub session_count: usize,
    pub attention_count: usize,
    pub attention: DesktopBridgeAttentionState,
    pub search_input: String,
}

impl From<DesktopProjectSummary> for DesktopBridgeProjectSummary {
    fn from(value: DesktopProjectSummary) -> Self {
        Self {
            project_view_id: value.project_view_id,
            display_name: value.display_name,
            canonical_workspace_id: value.canonical_workspace_id,
            canonical_repo_root: value.canonical_repo_root,
            canonical_git_common_dir: value.canonical_git_common_dir,
            pinned: value.pinned,
            presentation_order: value.presentation_order,
            collapsed: value.collapsed,
            presentation_revision: value.presentation_revision,
            session_count: value.session_count,
            attention_count: value.attention_count,
            attention: value.attention.into(),
            search_input: value.search_input,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeSessionSummary {
    pub canonical_session_id: String,
    pub canonical_workstream_id: String,
    pub canonical_workspace_id: String,
    pub display_name: String,
    pub display_alias: Option<String>,
    pub canonical_display_name: String,
    pub pinned: bool,
    pub presentation_order: i64,
    pub archived: bool,
    pub presentation_revision: Option<i64>,
    pub runtime: DesktopBridgeRuntimeProjection,
    pub attention: DesktopBridgeAttentionState,
    pub search_input: String,
}

impl From<DesktopSessionSummary> for DesktopBridgeSessionSummary {
    fn from(value: DesktopSessionSummary) -> Self {
        Self {
            canonical_session_id: value.canonical_session_id,
            canonical_workstream_id: value.canonical_workstream_id,
            canonical_workspace_id: value.canonical_workspace_id,
            display_name: value.display_name,
            display_alias: value.display_alias,
            canonical_display_name: value.canonical_display_name,
            pinned: value.pinned,
            presentation_order: value.presentation_order,
            archived: value.archived,
            presentation_revision: value.presentation_revision,
            runtime: value.runtime.into(),
            attention: value.attention.into(),
            search_input: value.search_input,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeWorkstream {
    pub workstream_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeProject {
    pub project: DesktopBridgeProjectSummary,
    pub sessions: Vec<DesktopBridgeSessionSummary>,
    pub available_workstreams: Vec<DesktopBridgeWorkstream>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeSnapshot {
    pub projects: Vec<DesktopBridgeProject>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeProjectPresentationRequest {
    pub workspace_id: String,
    pub display_name: String,
    pub pinned: bool,
    pub sort_order: i64,
    pub collapsed: bool,
    pub expected_revision: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeSessionPresentationRequest {
    pub session_id: String,
    pub display_alias: String,
    pub pinned: bool,
    pub sort_order: i64,
    pub archived: bool,
    pub expected_revision: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeCreateSessionRequest {
    pub workspace_id: String,
    pub workstream_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeRenameSessionRequest {
    pub session_id: String,
    pub display_name: String,
    pub presentation: Option<DesktopBridgeSessionPresentationRequest>,
}

pub fn desktop_bridge_default_home() -> Result<std::path::PathBuf> {
    let path = if let Some(path) = std::env::var_os("WINDS_HOME") {
        std::path::PathBuf::from(path)
    } else if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home).join(".winds")
    } else if let Some(profile) = std::env::var_os("USERPROFILE") {
        std::path::PathBuf::from(profile).join(".winds")
    } else {
        return Err(
            "desktop Winds home is unavailable: WINDS_HOME, HOME, and USERPROFILE are unset".into(),
        );
    };
    if !path.is_absolute() {
        return Err("desktop WINDS_HOME must resolve to an absolute path".into());
    }
    Ok(path)
}

pub fn desktop_bridge_snapshot(home: &std::path::Path) -> Result<DesktopBridgeSnapshot> {
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let mut projects = Vec::new();
    let mut workspaces = store.list_desktop_workspaces()?;
    workspaces.sort_by(|left, right| left.workspace_id.cmp(&right.workspace_id));
    for workspace in workspaces {
        let workspace_id = workspace.workspace_id.clone();
        let sessions = facade.list_sessions(&workspace_id)?;
        let project = facade.project_summary_from_sessions(workspace, &sessions)?;
        let mut workstreams = store
            .list_workstreams(&workspace_id)?
            .into_iter()
            .map(|workstream| DesktopBridgeWorkstream {
                workstream_id: workstream.workstream_id,
                display_name: workstream.display_name,
            })
            .collect::<Vec<_>>();
        workstreams.sort_by(|left, right| left.workstream_id.cmp(&right.workstream_id));
        workstreams.dedup_by(|left, right| left.workstream_id == right.workstream_id);
        projects.push(DesktopBridgeProject {
            project: project.into(),
            sessions: sessions.into_iter().map(Into::into).collect(),
            available_workstreams: workstreams,
        });
    }
    projects.sort_by(|left, right| {
        right
            .project
            .pinned
            .cmp(&left.project.pinned)
            .then(
                left.project
                    .presentation_order
                    .cmp(&right.project.presentation_order),
            )
            .then(
                left.project
                    .display_name
                    .to_ascii_lowercase()
                    .cmp(&right.project.display_name.to_ascii_lowercase()),
            )
            .then(
                left.project
                    .canonical_workspace_id
                    .cmp(&right.project.canonical_workspace_id),
            )
    });
    Ok(DesktopBridgeSnapshot { projects })
}

pub fn desktop_bridge_update_project(
    home: &std::path::Path,
    request: DesktopBridgeProjectPresentationRequest,
) -> Result<DesktopBridgeProjectSummary> {
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let now_ms = desktop_bridge_now_ms()?;
    let DesktopBridgeProjectPresentationRequest {
        workspace_id,
        display_name,
        pinned,
        sort_order,
        collapsed,
        expected_revision,
    } = request;
    match expected_revision {
        None => facade
            .create_project_presentation(
                &workspace_id,
                &display_name,
                pinned,
                sort_order,
                collapsed,
                now_ms,
            )
            .map(Into::into),
        Some(expected_revision) => facade
            .update_project_presentation(&DesktopProjectPresentationCommand {
                workspace_id,
                display_name,
                pinned,
                sort_order,
                collapsed,
                expected_revision,
                now_ms,
            })
            .map(Into::into),
    }
}

pub fn desktop_bridge_create_session(
    home: &std::path::Path,
    request: DesktopBridgeCreateSessionRequest,
) -> Result<DesktopBridgeSessionSummary> {
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let session_id = desktop_bridge_new_session_id()?;
    facade
        .create_session(
            &request.workspace_id,
            &request.workstream_id,
            &session_id,
            &request.display_name,
            desktop_bridge_now_ms()?,
        )
        .map(Into::into)
}

pub fn desktop_bridge_rename_session(
    home: &std::path::Path,
    request: DesktopBridgeRenameSessionRequest,
) -> Result<DesktopBridgeSessionSummary> {
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let now_ms = desktop_bridge_now_ms()?;
    let DesktopBridgeRenameSessionRequest {
        session_id,
        display_name,
        presentation,
    } = request;
    let stored_presentation = store.load_desktop_session_presentation(&session_id)?;
    match (presentation, stored_presentation) {
        (None, None) => {}
        (None, Some(_)) => {
            return Err("desktop Session presentation changed; refresh before renaming".into());
        }
        (Some(_), None) => {
            return Err(
                "desktop Session presentation no longer exists; refresh before renaming".into(),
            );
        }
        (Some(presentation), Some(_)) => {
            if presentation.session_id != session_id || presentation.display_alias != display_name {
                return Err(
                    "desktop Session rename presentation does not match rename target".into(),
                );
            }
            if presentation.expected_revision.is_none() {
                return Err(
                    "desktop Session rename requires the current presentation revision".into(),
                );
            }
            facade.update_session_presentation(&DesktopSessionPresentationCommand {
                session_id: presentation.session_id,
                display_alias: presentation.display_alias,
                pinned: presentation.pinned,
                sort_order: presentation.sort_order,
                archived: presentation.archived,
                expected_revision: presentation.expected_revision,
                now_ms,
            })?;
        }
    }
    facade
        .rename_session(&session_id, &display_name, now_ms)
        .map(Into::into)
}

pub fn desktop_bridge_update_session(
    home: &std::path::Path,
    request: DesktopBridgeSessionPresentationRequest,
) -> Result<DesktopBridgeSessionSummary> {
    let store = Store::open(home)?;
    DesktopFacade::new(&store)
        .update_session_presentation(&DesktopSessionPresentationCommand {
            session_id: request.session_id,
            display_alias: request.display_alias,
            pinned: request.pinned,
            sort_order: request.sort_order,
            archived: request.archived,
            expected_revision: request.expected_revision,
            now_ms: desktop_bridge_now_ms()?,
        })
        .map(Into::into)
}

fn desktop_bridge_now_ms() -> Result<i64> {
    let duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    i64::try_from(duration.as_millis())
        .map_err(|_| "desktop system time exceeds supported millisecond range".into())
}

fn desktop_bridge_new_session_id() -> Result<String> {
    static NEXT_SESSION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let duration = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
    let sequence = NEXT_SESSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(format!(
        "desktop-session-{}-{}-{sequence}",
        std::process::id(),
        duration.as_nanos()
    ))
}
