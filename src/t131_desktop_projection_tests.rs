use crate::agentic_runtime::{
    AgentExecutionObservation, RuntimeKind, RuntimeResumeResolution, SafeVersionObservation,
    discover_runtime_from_safe_observations,
};
use crate::desktop::{
    DesktopAttentionState, DesktopFacade, DesktopRuntimeFamily, DesktopRuntimeState,
    DesktopSessionPresentationCommand, classify_runtime_state,
};
use crate::domain::workflow::{
    StageLifecycleState, StageRunIdentity, StageTransitionAuthority, StageTransitionRequest,
    TruthSource, WorkflowRunIdentity,
};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::ffi::OsStr;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t131-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}
fn cleanup_owned_root(root: &Path) {
    let canonical_root = root.canonicalize().unwrap();
    let canonical_temp = std::env::temp_dir().canonicalize().unwrap();
    let owned_name = canonical_root
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("winds-t131-"));
    assert!(canonical_root.starts_with(&canonical_temp));
    assert!(owned_name);
    fs::remove_dir_all(&canonical_root).unwrap();
}

fn seed_workspace(store: &Store, suffix: &str, now_ms: i64) -> (String, String) {
    let workspace_id = format!("workspace-{suffix}");
    let workstream_id = format!("workstream-{suffix}");
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: &workspace_id,
                canonical_worktree_root: &format!("/fixture/{workspace_id}"),
                git_common_dir: &format!("/fixture/{workspace_id}/.git"),
            },
            now_ms,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: &workstream_id,
                workspace_id: &workspace_id,
                display_name: "Desktop project workstream",
            },
            now_ms + 1,
        )
        .unwrap();
    (workspace_id, workstream_id)
}
fn seed_session(
    store: &Store,
    workstream_id: &str,
    session_id: &str,
    display_name: &str,
    now_ms: i64,
) {
    store
        .create_winds_session(
            NewWindsSession {
                session_id,
                workstream_id,
                display_name,
            },
            now_ms,
        )
        .unwrap();
}

fn fake_executable_path(root: &Path, stem: &str) -> PathBuf {
    #[cfg(windows)]
    {
        root.join(format!("{stem}.exe"))
    }
    #[cfg(not(windows))]
    {
        root.join(stem)
    }
}

fn create_fake_executable(root: &Path, stem: &str, bytes: &[u8]) -> PathBuf {
    let path = fake_executable_path(root, stem);
    fs::write(&path, bytes).unwrap();
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }
    path
}
#[test]
fn runtime_state_vocabulary_remains_explicit_and_non_promotional() {
    let codex = [DesktopRuntimeFamily::Codex];
    let claude = [DesktopRuntimeFamily::Claude];
    assert_eq!(
        classify_runtime_state(&[], &[], false, false),
        DesktopRuntimeState::Unknown
    );
    assert_eq!(
        classify_runtime_state(&codex, &[], false, false),
        DesktopRuntimeState::Requested
    );
    assert_eq!(
        classify_runtime_state(&[], &codex, false, false),
        DesktopRuntimeState::Observed
    );
    assert_eq!(
        classify_runtime_state(&codex, &claude, false, false),
        DesktopRuntimeState::Mismatch
    );
    assert_eq!(
        classify_runtime_state(&codex, &[], false, true),
        DesktopRuntimeState::Unavailable
    );
    assert_eq!(
        classify_runtime_state(&codex, &codex, true, false),
        DesktopRuntimeState::Stale
    );
    assert_eq!(
        classify_runtime_state(
            &[DesktopRuntimeFamily::Codex, DesktopRuntimeFamily::Claude],
            &[],
            false,
            false
        ),
        DesktopRuntimeState::Conflicting
    );
}
#[test]
fn project_session_commands_preserve_canonical_identity() {
    let root = test_root("identity");
    let store = Store::open(&root).unwrap();
    let (workspace_id, workstream_id) = seed_workspace(&store, "identity", 10);
    let facade = DesktopFacade::new(&store);

    let project = facade
        .create_project_context(&workspace_id, "Primary Project", 20)
        .unwrap();
    assert_eq!(project.project_view_id, workspace_id);
    assert_eq!(project.canonical_workspace_id, workspace_id);

    let session = facade
        .create_session(
            &workspace_id,
            &workstream_id,
            "session-identity",
            "Original session",
            30,
        )
        .unwrap();
    assert_eq!(session.canonical_session_id, "session-identity");

    let workflow =
        WorkflowRunIdentity::new("workflow-identity", &workspace_id, &workstream_id).unwrap();
    store.create_workflow_run(&workflow, 31).unwrap();
    let stage = StageRunIdentity::new(
        "stage-identity",
        "workflow-identity",
        "implementation",
        1,
        None,
    )
    .unwrap();
    store.create_stage_run(&stage, None, 32).unwrap();
    let workflow_before = store.load_workflow_run("workflow-identity").unwrap();
    let stage_before = store.load_stage_run("stage-identity").unwrap();

    let renamed = facade
        .rename_session("session-identity", "Renamed session", 40)
        .unwrap();
    assert_eq!(renamed.canonical_session_id, "session-identity");
    assert_eq!(renamed.canonical_workstream_id, workstream_id);
    assert_eq!(renamed.canonical_display_name, "Renamed session");
    let presented = facade
        .update_session_presentation(&DesktopSessionPresentationCommand {
            session_id: "session-identity".to_owned(),
            display_alias: "Alias only".to_owned(),
            pinned: true,
            sort_order: 17,
            archived: false,
            expected_revision: None,
            now_ms: 50,
        })
        .unwrap();
    assert_eq!(presented.display_name, "Alias only");
    assert_eq!(presented.canonical_display_name, "Renamed session");
    assert_eq!(presented.canonical_session_id, "session-identity");
    assert_eq!(
        store.load_workflow_run("workflow-identity").unwrap(),
        workflow_before
    );
    assert_eq!(
        store.load_stage_run("stage-identity").unwrap(),
        stage_before
    );
    assert_eq!(
        store
            .load_winds_session("session-identity")
            .unwrap()
            .workstream_id,
        workstream_id
    );

    let (_, foreign_workstream_id) = seed_workspace(&store, "foreign", 60);
    let error = facade
        .create_session(
            &workspace_id,
            &foreign_workstream_id,
            "session-crossed",
            "Crossed",
            70,
        )
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "desktop session workstream does not belong to the selected project"
    );
    drop(store);
    cleanup_owned_root(&root);
}
#[test]
fn forged_labels_cannot_create_observed_runtime_identity() {
    let root = test_root("runtime-proof");
    let store = Store::open(&root).unwrap();
    let (workspace_id, workstream_id) = seed_workspace(&store, "runtime", 10);
    seed_session(
        &store,
        &workstream_id,
        "session-runtime",
        "Claude Codex",
        20,
    );
    let facade = DesktopFacade::new(&store);

    let initial = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].runtime.state, DesktopRuntimeState::Unknown);
    assert!(initial[0].runtime.observed.is_empty());

    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");
    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed("codex-cli 1.2.3-fixture".to_owned()),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );
    store
        .create_runtime_session_binding(
            "runtime-binding-1",
            "session-runtime",
            &discovery,
            None,
            30,
        )
        .unwrap();
    let observed = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(observed[0].runtime.state, DesktopRuntimeState::Observed);
    assert_eq!(
        observed[0].runtime.observed,
        vec![DesktopRuntimeFamily::Codex]
    );

    facade
        .update_session_presentation(&DesktopSessionPresentationCommand {
            session_id: "session-runtime".to_owned(),
            display_alias: "Pretend Claude".to_owned(),
            pinned: false,
            sort_order: 0,
            archived: false,
            expected_revision: None,
            now_ms: 40,
        })
        .unwrap();
    let renamed = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(renamed[0].display_name, "Pretend Claude");
    assert_eq!(renamed[0].runtime.state, DesktopRuntimeState::Observed);
    assert_eq!(
        renamed[0].runtime.observed,
        vec![DesktopRuntimeFamily::Codex]
    );

    store
        .mark_runtime_binding_ownership_lost("runtime-binding-1", 50)
        .unwrap();
    let ownership_lost = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(
        ownership_lost[0].runtime.state,
        DesktopRuntimeState::Observed
    );

    fs::write(&executable, b"fixture-codex-x1\n").unwrap();
    let stale = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(stale[0].runtime.state, DesktopRuntimeState::Stale);

    fs::remove_file(&executable).unwrap();
    let unavailable = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(
        unavailable[0].runtime.state,
        DesktopRuntimeState::Unavailable
    );

    drop(store);
    cleanup_owned_root(&root);
}
#[test]
fn attention_rollup_ignores_agent_reported_blocking_claims() {
    let root = test_root("attention-truth");
    let store = Store::open(&root).unwrap();
    let (workspace_id, workstream_id) = seed_workspace(&store, "attention", 10);
    seed_session(&store, &workstream_id, "session-attention", "Attention", 20);

    let workflow =
        WorkflowRunIdentity::new("workflow-attention", &workspace_id, &workstream_id).unwrap();
    store.create_workflow_run(&workflow, 30).unwrap();
    let stage = StageRunIdentity::new(
        "stage-attention",
        "workflow-attention",
        "implementation",
        1,
        None,
    )
    .unwrap();
    store.create_stage_run(&stage, None, 31).unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            "actor-attention",
            "stage-attention",
            "session-attention",
            &RuntimeResumeResolution::Unavailable,
            32,
        )
        .unwrap();
    store
        .transition_stage_run(
            "stage-attention",
            &StageTransitionRequest::new(
                "activate-attention",
                StageLifecycleState::Prepared,
                StageLifecycleState::Active,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .unwrap(),
            33,
        )
        .unwrap();
    store
        .transition_stage_run(
            "stage-attention",
            &StageTransitionRequest::new(
                "agent-claims-waiting",
                StageLifecycleState::Active,
                StageLifecycleState::WaitingApproval,
                TruthSource::AgentReported,
                StageTransitionAuthority::None,
            )
            .unwrap(),
            34,
        )
        .unwrap();

    let facade = DesktopFacade::new(&store);
    let agent_only = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(agent_only[0].attention, DesktopAttentionState::None);
    store
        .transition_stage_run(
            "stage-attention",
            &StageTransitionRequest::new(
                "resume-after-agent-claim",
                StageLifecycleState::WaitingApproval,
                StageLifecycleState::Active,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .unwrap(),
            35,
        )
        .unwrap();
    store
        .transition_stage_run(
            "stage-attention",
            &StageTransitionRequest::new(
                "winds-observed-waiting",
                StageLifecycleState::Active,
                StageLifecycleState::WaitingApproval,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .unwrap(),
            36,
        )
        .unwrap();

    let trusted = facade.list_sessions(&workspace_id).unwrap();
    assert_eq!(trusted[0].attention, DesktopAttentionState::WaitingApproval);
    let project = facade.open_project(&workspace_id).unwrap();
    assert_eq!(project.attention_count, 1);
    assert_eq!(project.attention, DesktopAttentionState::WaitingApproval);

    drop(store);
    cleanup_owned_root(&root);
}
#[test]
fn hundred_projects_thousand_sessions_have_stable_order_and_search_inputs() {
    let root = test_root("scale-ordering");
    let store = Store::open(&root).unwrap();

    for project_index in (0..100).rev() {
        let suffix = format!("{project_index:03}");
        let base = 1000 + i64::from(project_index) * 100;
        let (workspace_id, workstream_id) = seed_workspace(&store, &suffix, base);
        for session_index in (0..10).rev() {
            seed_session(
                &store,
                &workstream_id,
                &format!("session-{suffix}-{session_index:02}"),
                &format!("Session {session_index:02}"),
                base + 2 + i64::from(session_index),
            );
        }
        assert_eq!(store.list_winds_sessions(&workstream_id).unwrap().len(), 10);
        assert_eq!(
            store.load_workspace(&workspace_id).unwrap().workspace_id,
            workspace_id
        );
    }

    let facade = DesktopFacade::new(&store);
    let projects_first = facade.list_projects().unwrap();
    let projects_second = facade.list_projects().unwrap();
    assert_eq!(projects_first.len(), 100);
    assert_eq!(projects_first, projects_second);
    let collect_sessions = |projects: &[crate::desktop::DesktopProjectSummary]| {
        let mut rows = Vec::new();
        for project in projects {
            for session in facade
                .list_sessions(&project.canonical_workspace_id)
                .unwrap()
            {
                rows.push((
                    project.canonical_workspace_id.clone(),
                    session.canonical_session_id,
                    session.search_input,
                ));
            }
        }
        rows
    };
    let sessions_first = collect_sessions(&projects_first);
    let sessions_second = collect_sessions(&projects_second);
    assert_eq!(sessions_first.len(), 1000);
    assert_eq!(sessions_first, sessions_second);
    assert!(
        projects_first
            .iter()
            .all(|project| !project.search_input.is_empty())
    );
    assert!(sessions_first.iter().all(|row| !row.2.is_empty()));

    drop(store);
    cleanup_owned_root(&root);
}
