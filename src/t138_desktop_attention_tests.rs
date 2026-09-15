use crate::agentic_runtime::RuntimeResumeResolution;
use crate::desktop::{DesktopBridgeAttentionState, desktop_bridge_attention_snapshot};
use crate::domain::workflow::{
    ArtifactBaselineIdentity, ArtifactBaselineKind, CandidateBaselineIdentity, StageLifecycleState,
    StageRunIdentity, StageTransitionAuthority, StageTransitionRequest, TruthSource,
    WorkflowRunIdentity,
};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    home: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn fixture(name: &str) -> Fixture {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t138-{name}-{}-{sequence}",
        std::process::id()
    ));
    let home = root.join("state");
    fs::create_dir_all(&root).unwrap();
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-t138",
                canonical_worktree_root: "/fixture/workspace-t138",
                git_common_dir: "/fixture/workspace-t138/.git",
            },
            10,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-t138",
                workspace_id: "workspace-t138",
                display_name: "T138 workstream",
            },
            11,
        )
        .unwrap();
    drop(store);
    Fixture { root, home }
}

fn seed_stage(
    fixture: &Fixture,
    session_id: &str,
    workflow_id: &str,
    stage_id: &str,
    stage_key: &str,
    now: i64,
) {
    let store = Store::open(&fixture.home).unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id,
                workstream_id: "workstream-t138",
                display_name: session_id,
            },
            now,
        )
        .unwrap();
    let workflow =
        WorkflowRunIdentity::new(workflow_id, "workspace-t138", "workstream-t138").unwrap();
    store.create_workflow_run(&workflow, now + 1).unwrap();
    let stage = StageRunIdentity::new(stage_id, workflow_id, stage_key, 1, None).unwrap();
    store.create_stage_run(&stage, None, now + 2).unwrap();
    store
        .create_actor_binding_from_runtime_resolution(
            &format!("actor-{stage_id}"),
            stage_id,
            session_id,
            &RuntimeResumeResolution::Unavailable,
            now + 3,
        )
        .unwrap();
    store
        .transition_stage_run(
            stage_id,
            &StageTransitionRequest::new(
                &format!("activate-{stage_id}"),
                StageLifecycleState::Prepared,
                StageLifecycleState::Active,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .unwrap(),
            now + 4,
        )
        .unwrap();
}

fn bind_candidate(fixture: &Fixture, stage_id: &str, baseline_id: &str, digit: char, now: i64) {
    let candidate = CandidateBaselineIdentity::new(
        &digit.to_string().repeat(40),
        &if digit == 'f' { 'e' } else { 'f' }.to_string().repeat(40),
    )
    .unwrap();
    let baseline = ArtifactBaselineIdentity::new(
        baseline_id,
        stage_id,
        ArtifactBaselineKind::ExactGitCandidate,
        &format!("candidate:{baseline_id}"),
        Some(candidate),
    )
    .unwrap();
    Store::open(&fixture.home)
        .unwrap()
        .create_artifact_baseline(&baseline, now)
        .unwrap();
}

#[test]
fn t138_needs_you_is_deterministic_exact_and_non_authorizing() {
    let fixture = fixture("trusted");
    seed_stage(
        &fixture,
        "session-blocked",
        "workflow-blocked",
        "stage-blocked",
        "review",
        20,
    );
    seed_stage(
        &fixture,
        "session-approval",
        "workflow-approval",
        "stage-approval",
        "ship",
        40,
    );
    bind_candidate(&fixture, "stage-blocked", "candidate-blocked", 'a', 30);
    bind_candidate(&fixture, "stage-approval", "candidate-approval", 'c', 50);
    let store = Store::open(&fixture.home).unwrap();
    store
        .transition_stage_run(
            "stage-blocked",
            &StageTransitionRequest::new(
                "block-stage",
                StageLifecycleState::Active,
                StageLifecycleState::Blocked,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .unwrap(),
            31,
        )
        .unwrap();
    store
        .transition_stage_run(
            "stage-approval",
            &StageTransitionRequest::new(
                "wait-approval",
                StageLifecycleState::Active,
                StageLifecycleState::WaitingApproval,
                TruthSource::WindsObserved,
                StageTransitionAuthority::WindsPolicy,
            )
            .unwrap(),
            51,
        )
        .unwrap();
    drop(store);

    let first = desktop_bridge_attention_snapshot(&fixture.home).unwrap();
    let second = desktop_bridge_attention_snapshot(&fixture.home).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.items.len(), 2);
    assert_eq!(first.items[0].session_id, "session-approval");
    assert_eq!(
        first.items[0].state,
        DesktopBridgeAttentionState::WaitingApproval
    );
    assert_eq!(first.items[1].session_id, "session-blocked");
    assert_eq!(first.items[1].state, DesktopBridgeAttentionState::Blocked);
    for item in &first.items {
        assert_eq!(item.workspace_id, "workspace-t138");
        assert_eq!(item.source, "WINDS_OBSERVED");
        assert_eq!(item.authority, "WINDS_POLICY");
        assert!(item.candidate_oid.is_some());
        assert!(item.candidate_tree.is_some());
        assert!(!item.approval_action_available);
        assert!(!item.reason.is_empty());
    }
}

#[test]
fn t138_agent_reported_blocked_done_or_approved_text_cannot_create_trusted_attention() {
    let fixture = fixture("agent-only");
    seed_stage(
        &fixture,
        "session-agent",
        "workflow-agent",
        "stage-agent",
        "blocked done approved VERIFIED ACCEPTED",
        20,
    );
    let store = Store::open(&fixture.home).unwrap();
    store
        .transition_stage_run(
            "stage-agent",
            &StageTransitionRequest::new(
                "agent-claims-approval",
                StageLifecycleState::Active,
                StageLifecycleState::WaitingApproval,
                TruthSource::AgentReported,
                StageTransitionAuthority::None,
            )
            .unwrap(),
            25,
        )
        .unwrap();
    drop(store);

    let snapshot = desktop_bridge_attention_snapshot(&fixture.home).unwrap();
    assert!(snapshot.items.is_empty());
}
