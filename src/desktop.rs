use crate::agentic_runtime::{
    RuntimeIdentityRevalidation, RuntimeKind, revalidate_runtime_identity,
};
use crate::domain::workflow::{
    RETRY_OUTCOME_AMBIGUOUS_EFFECT, RETRY_OUTCOME_BUDGET_EXHAUSTED, RETRY_OUTCOME_FAILURE_RECORDED,
    RETRY_OUTCOME_NO_PROGRESS, StageLifecycleState, StageTransitionAuthority, TruthSource,
};
use crate::domain::{WindsSessionRecord, WorkspaceRecord};
use crate::persistent_runtime::presentation::DesktopRuntimeTruthProjection as PersistentDesktopRuntimeTruthProjection;
use crate::store::{
    DesktopAttentionFact, DesktopLayoutPresentation, DesktopLayoutPresentationInput,
    DesktopProjectPresentationInput, DesktopSessionPresentationInput, NewWindsSession, Result,
    Store,
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
    Blocked,
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
            Self::Blocked => 2,
            Self::RetryRequired => 3,
            Self::WaitingExternal => 4,
            Self::Stale => 5,
            Self::Unknown => 6,
            Self::None => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopAttentionItem {
    pub(crate) workspace_id: String,
    pub(crate) session_id: String,
    pub(crate) workflow_run_id: String,
    pub(crate) stage_run_id: String,
    pub(crate) stage_key: String,
    pub(crate) state: DesktopAttentionState,
    pub(crate) reason: String,
    pub(crate) source: TruthSource,
    pub(crate) authority: StageTransitionAuthority,
    pub(crate) candidate_oid: Option<String>,
    pub(crate) candidate_tree: Option<String>,
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
        Ok(self
            .store
            .desktop_attention_facts(session_id)?
            .iter()
            .map(attention_state_for_fact)
            .min_by_key(|state| state.rank())
            .unwrap_or(DesktopAttentionState::None))
    }

    pub(crate) fn attention_items(&self) -> Result<Vec<DesktopAttentionItem>> {
        let mut items = self
            .store
            .desktop_attention_facts_all()?
            .into_iter()
            .filter_map(|fact| {
                let state = attention_state_for_fact(&fact);
                if matches!(
                    state,
                    DesktopAttentionState::Unknown | DesktopAttentionState::None
                ) {
                    return None;
                }
                let source = fact.source?;
                let authority = fact.authority.unwrap_or(StageTransitionAuthority::None);
                Some(DesktopAttentionItem {
                    workspace_id: fact.workspace_id,
                    session_id: fact.session_id,
                    workflow_run_id: fact.workflow_run_id,
                    stage_run_id: fact.stage_run_id,
                    stage_key: fact.stage_key,
                    state,
                    reason: attention_reason(state, fact.outcome_reason.as_deref()),
                    source,
                    authority,
                    candidate_oid: fact.candidate_oid,
                    candidate_tree: fact.candidate_tree,
                })
            })
            .collect::<Vec<_>>();
        items.sort_by(|left, right| {
            left.state
                .rank()
                .cmp(&right.state.rank())
                .then(left.workspace_id.cmp(&right.workspace_id))
                .then(left.session_id.cmp(&right.session_id))
                .then(left.workflow_run_id.cmp(&right.workflow_run_id))
                .then(left.stage_run_id.cmp(&right.stage_run_id))
        });
        Ok(items)
    }
}

fn attention_state_for_fact(fact: &DesktopAttentionFact) -> DesktopAttentionState {
    let trusted = matches!(
        fact.source,
        Some(TruthSource::WindsObserved | TruthSource::HumanDecided)
    );
    if fact.source.is_none() && fact.lifecycle_state == StageLifecycleState::Prepared {
        return DesktopAttentionState::None;
    }
    if !trusted {
        return DesktopAttentionState::Unknown;
    }
    match fact.lifecycle_state {
        StageLifecycleState::WaitingApproval => DesktopAttentionState::WaitingApproval,
        StageLifecycleState::WaitingExternal => DesktopAttentionState::WaitingExternal,
        StageLifecycleState::RecoveryRequired => DesktopAttentionState::RecoveryRequired,
        StageLifecycleState::Stale => DesktopAttentionState::Stale,
        StageLifecycleState::Blocked => DesktopAttentionState::Blocked,
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
}

fn attention_reason(state: DesktopAttentionState, outcome_reason: Option<&str>) -> String {
    if let Some(reason) = outcome_reason {
        return reason.to_owned();
    }
    match state {
        DesktopAttentionState::RecoveryRequired => "canonical stage requires recovery".to_owned(),
        DesktopAttentionState::WaitingApproval => {
            "canonical stage is waiting for human approval".to_owned()
        }
        DesktopAttentionState::Blocked => "canonical stage is blocked".to_owned(),
        DesktopAttentionState::RetryRequired => {
            "canonical stage requires an explicit retry".to_owned()
        }
        DesktopAttentionState::WaitingExternal => {
            "canonical stage is waiting on an external condition".to_owned()
        }
        DesktopAttentionState::Stale => "canonical stage context is stale".to_owned(),
        DesktopAttentionState::Unknown => "canonical attention state is unknown".to_owned(),
        DesktopAttentionState::None => "no material attention".to_owned(),
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgePersistentRuntimeTruthProjection {
    pub runtime_namespace_id: String,
    pub display_alias: Option<String>,
    pub ownership: String,
    pub liveness: String,
    pub continuity: String,
    pub authority: String,
    pub replay: String,
    pub replay_first_available_sequence: Option<u64>,
    pub replay_last_dropped_sequence: Option<u64>,
    pub health: String,
    pub truth_source: String,
    pub verification: String,
    pub human_acceptance: String,
}

impl From<PersistentDesktopRuntimeTruthProjection>
    for DesktopBridgePersistentRuntimeTruthProjection
{
    fn from(value: PersistentDesktopRuntimeTruthProjection) -> Self {
        Self {
            runtime_namespace_id: value.runtime_namespace_id,
            display_alias: value.display_alias,
            ownership: value.ownership,
            liveness: value.liveness,
            continuity: value.continuity,
            authority: value.authority,
            replay: value.replay,
            replay_first_available_sequence: value.replay_first_available_sequence,
            replay_last_dropped_sequence: value.replay_last_dropped_sequence,
            health: value.health,
            truth_source: value.truth_source,
            verification: value.verification,
            human_acceptance: value.human_acceptance,
        }
    }
}

pub(crate) fn desktop_bridge_persistent_runtime_truth(
    value: PersistentDesktopRuntimeTruthProjection,
) -> DesktopBridgePersistentRuntimeTruthProjection {
    value.into()
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
    Blocked,
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
            DesktopAttentionState::Blocked => Self::Blocked,
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
pub struct DesktopBridgeAttentionItem {
    pub workspace_id: String,
    pub session_id: String,
    pub workflow_run_id: String,
    pub stage_run_id: String,
    pub stage_key: String,
    pub state: DesktopBridgeAttentionState,
    pub reason: String,
    pub source: String,
    pub authority: String,
    pub candidate_oid: Option<String>,
    pub candidate_tree: Option<String>,
    pub approval_action_available: bool,
}

impl From<DesktopAttentionItem> for DesktopBridgeAttentionItem {
    fn from(value: DesktopAttentionItem) -> Self {
        Self {
            workspace_id: value.workspace_id,
            session_id: value.session_id,
            workflow_run_id: value.workflow_run_id,
            stage_run_id: value.stage_run_id,
            stage_key: value.stage_key,
            state: value.state.into(),
            reason: value.reason,
            source: value.source.as_db_str().to_owned(),
            authority: value.authority.as_db_str().to_owned(),
            candidate_oid: value.candidate_oid,
            candidate_tree: value.candidate_tree,
            approval_action_available: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeAttentionSnapshot {
    pub items: Vec<DesktopBridgeAttentionItem>,
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
pub struct DesktopBridgeProjectPresentationBatchRequest {
    pub updates: Vec<DesktopBridgeProjectPresentationRequest>,
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
pub struct DesktopBridgeSessionPresentationBatchRequest {
    pub updates: Vec<DesktopBridgeSessionPresentationRequest>,
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
    pub presentation: DesktopBridgeSessionPresentationRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeLayoutPresentation {
    pub workspace_id: String,
    pub layout_mode: String,
    pub left_session_id: Option<String>,
    pub right_session_id: Option<String>,
    pub split_basis_points: i64,
    pub revision: i64,
}

impl From<DesktopLayoutPresentation> for DesktopBridgeLayoutPresentation {
    fn from(value: DesktopLayoutPresentation) -> Self {
        Self {
            workspace_id: value.workspace_id,
            layout_mode: value.layout_mode,
            left_session_id: value.left_session_id,
            right_session_id: value.right_session_id,
            split_basis_points: value.split_basis_points,
            revision: value.revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBridgeLayoutRequest {
    pub workspace_id: String,
    pub layout_mode: String,
    pub left_session_id: Option<String>,
    pub right_session_id: Option<String>,
    pub split_basis_points: i64,
    pub expected_revision: Option<i64>,
}

pub(crate) fn desktop_bridge_resolve_default_home(
    winds_home: Option<std::path::PathBuf>,
    home: Option<std::path::PathBuf>,
    user_profile: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf> {
    if let Some(path) = winds_home {
        if !path.is_absolute() {
            return Err("desktop WINDS_HOME must resolve to an absolute path".into());
        }
        return Ok(path);
    }
    for path in [home, user_profile].into_iter().flatten() {
        if path.is_absolute() {
            return Ok(path.join(".winds"));
        }
    }
    Err(
        "desktop Winds home is unavailable: HOME and USERPROFILE do not resolve to absolute paths"
            .into(),
    )
}

pub fn desktop_bridge_default_home() -> Result<std::path::PathBuf> {
    desktop_bridge_resolve_default_home(
        std::env::var_os("WINDS_HOME").map(std::path::PathBuf::from),
        std::env::var_os("HOME").map(std::path::PathBuf::from),
        std::env::var_os("USERPROFILE").map(std::path::PathBuf::from),
    )
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

pub fn desktop_bridge_attention_snapshot(
    home: &std::path::Path,
) -> Result<DesktopBridgeAttentionSnapshot> {
    let store = Store::open(home)?;
    let items = DesktopFacade::new(&store)
        .attention_items()?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(DesktopBridgeAttentionSnapshot { items })
}

pub fn desktop_bridge_load_layout(
    home: &std::path::Path,
    workspace_id: &str,
) -> Result<Option<DesktopBridgeLayoutPresentation>> {
    let store = Store::open(home)?;
    DesktopFacade::new(&store)
        .load_layout(workspace_id)
        .map(|layout| layout.map(Into::into))
}

pub fn desktop_bridge_save_layout(
    home: &std::path::Path,
    request: DesktopBridgeLayoutRequest,
) -> Result<DesktopBridgeLayoutPresentation> {
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let existing = facade.load_layout(&request.workspace_id)?;
    let left_missing = request.left_session_id.is_none();
    let right_missing = request.right_session_id.is_none();
    let command = DesktopLayoutCommand {
        workspace_id: request.workspace_id,
        layout_mode: request.layout_mode,
        left_session_id: request.left_session_id,
        right_session_id: request.right_session_id,
        split_basis_points: request.split_basis_points,
        right_dock_surface: existing.as_ref().map_or_else(
            || "FILES".to_owned(),
            |value| value.right_dock_surface.clone(),
        ),
        right_dock_binding: existing.as_ref().map_or_else(
            || "FOLLOW_FOCUS".to_owned(),
            |value| match value.right_dock_binding.as_str() {
                "LEFT_SESSION" if left_missing => "FOLLOW_FOCUS".to_owned(),
                "RIGHT_SESSION" if right_missing => "FOLLOW_FOCUS".to_owned(),
                _ => value.right_dock_binding.clone(),
            },
        ),
        left_dock_collapsed: existing
            .as_ref()
            .is_some_and(|value| value.left_dock_collapsed),
        left_dock_width_px: existing
            .as_ref()
            .map_or(280, |value| value.left_dock_width_px),
        right_dock_collapsed: existing
            .as_ref()
            .is_some_and(|value| value.right_dock_collapsed),
        right_dock_width_px: existing
            .as_ref()
            .map_or(360, |value| value.right_dock_width_px),
        appearance: existing
            .as_ref()
            .map_or_else(|| "SYSTEM".to_owned(), |value| value.appearance.clone()),
        contrast: existing
            .as_ref()
            .map_or_else(|| "STANDARD".to_owned(), |value| value.contrast.clone()),
        density: existing
            .as_ref()
            .map_or_else(|| "COMPACT".to_owned(), |value| value.density.clone()),
        reduced_motion: existing.as_ref().is_some_and(|value| value.reduced_motion),
    };
    facade
        .save_layout(
            &command,
            request.expected_revision,
            desktop_bridge_now_ms()?,
        )
        .map(Into::into)
}

const MAX_DESKTOP_PRESENTATION_BATCH: usize = 4096;

pub fn desktop_bridge_update_project(
    home: &std::path::Path,
    request: DesktopBridgeProjectPresentationBatchRequest,
) -> Result<Vec<DesktopBridgeProjectSummary>> {
    if request.updates.is_empty() {
        return Err("desktop Project presentation batch must not be empty".into());
    }
    if request.updates.len() > MAX_DESKTOP_PRESENTATION_BATCH {
        return Err("desktop Project presentation batch exceeds bounded maximum".into());
    }
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let now_ms = desktop_bridge_now_ms()?;
    let workspace_ids = request
        .updates
        .iter()
        .map(|update| update.workspace_id.clone())
        .collect::<Vec<_>>();
    let transaction = store.connection.unchecked_transaction()?;
    for update in request.updates {
        let DesktopBridgeProjectPresentationRequest {
            workspace_id,
            display_name,
            pinned,
            sort_order,
            collapsed,
            expected_revision,
        } = update;
        match expected_revision {
            None => {
                facade.create_project_presentation(
                    &workspace_id,
                    &display_name,
                    pinned,
                    sort_order,
                    collapsed,
                    now_ms,
                )?;
            }
            Some(expected_revision) => {
                facade.update_project_presentation(&DesktopProjectPresentationCommand {
                    workspace_id,
                    display_name,
                    pinned,
                    sort_order,
                    collapsed,
                    expected_revision,
                    now_ms,
                })?;
            }
        }
    }
    transaction.commit()?;
    workspace_ids
        .into_iter()
        .map(|workspace_id| facade.open_project(&workspace_id).map(Into::into))
        .collect()
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
    let DesktopBridgeRenameSessionRequest {
        session_id,
        display_name,
        presentation,
    } = request;
    if presentation.session_id != session_id || presentation.display_alias != display_name {
        return Err("desktop Session rename presentation does not match rename target".into());
    }
    facade
        .update_session_presentation(&DesktopSessionPresentationCommand {
            session_id: presentation.session_id,
            display_alias: presentation.display_alias,
            pinned: presentation.pinned,
            sort_order: presentation.sort_order,
            archived: presentation.archived,
            expected_revision: presentation.expected_revision,
            now_ms: desktop_bridge_now_ms()?,
        })
        .map(Into::into)
}

pub fn desktop_bridge_update_session(
    home: &std::path::Path,
    request: DesktopBridgeSessionPresentationBatchRequest,
) -> Result<Vec<DesktopBridgeSessionSummary>> {
    if request.updates.is_empty() {
        return Err("desktop Session presentation batch must not be empty".into());
    }
    if request.updates.len() > MAX_DESKTOP_PRESENTATION_BATCH {
        return Err("desktop Session presentation batch exceeds bounded maximum".into());
    }
    let store = Store::open(home)?;
    let facade = DesktopFacade::new(&store);
    let now_ms = desktop_bridge_now_ms()?;
    let session_ids = request
        .updates
        .iter()
        .map(|update| update.session_id.clone())
        .collect::<Vec<_>>();
    let transaction = store.connection.unchecked_transaction()?;
    for update in request.updates {
        facade.update_session_presentation(&DesktopSessionPresentationCommand {
            session_id: update.session_id,
            display_alias: update.display_alias,
            pinned: update.pinned,
            sort_order: update.sort_order,
            archived: update.archived,
            expected_revision: update.expected_revision,
            now_ms,
        })?;
    }
    transaction.commit()?;
    session_ids
        .into_iter()
        .map(|session_id| {
            facade
                .session_summary(store.load_winds_session(&session_id)?)
                .map(Into::into)
        })
        .collect()
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
