use crate::desktop_files::{
    DesktopBoundDockRequest, DesktopRightDockTarget, desktop_right_dock_bind,
};
use crate::desktop_inspection::{
    DesktopArtifactCandidateState, DesktopEvidenceFreshness, desktop_right_dock_artifacts,
    desktop_right_dock_context, desktop_right_dock_evidence,
};
use crate::domain::workflow::{
    ArtifactBaselineIdentity, ArtifactBaselineKind, CandidateBaselineIdentity, StageRunIdentity,
    WorkflowRunIdentity,
};
use crate::domain::{BlobEvidence, CheckEvidence, CheckStatus, Eligibility, EvidenceReport};
use crate::store::{NewRun, NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    home: PathBuf,
    repo: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn fixture(name: &str) -> Fixture {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t137-{name}-{}-{sequence}",
        std::process::id()
    ));
    let repo = root.join("repo");
    let home = root.join("state");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "t137@example.invalid"]);
    git(&repo, &["config", "user.name", "T137 Fixture"]);
    fs::write(repo.join("README.md"), "T137 fixture\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-qm", "fixture"]);

    let canonical_repo = repo.canonicalize().unwrap();
    let common = canonical_repo.join(".git").canonicalize().unwrap();
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-t137",
                canonical_worktree_root: canonical_repo.to_str().unwrap(),
                git_common_dir: common.to_str().unwrap(),
            },
            100,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-t137",
                workspace_id: "workspace-t137",
                display_name: "T137 workstream",
            },
            101,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-t137",
                workstream_id: "workstream-t137",
                display_name: "T137 session",
            },
            102,
        )
        .unwrap();
    let workflow =
        WorkflowRunIdentity::new("workflow-t137", "workspace-t137", "workstream-t137").unwrap();
    store.create_workflow_run(&workflow, 103).unwrap();
    let stage = StageRunIdentity::new("stage-t137", "workflow-t137", "inspect", 1, None).unwrap();
    store.create_stage_run(&stage, None, 104).unwrap();
    store
        .connection
        .execute(
            "INSERT INTO workflow_actor_bindings(
                binding_id, stage_run_id, winds_session_id, runtime_binding_id,
                continuation_class, bound_unix_ms
             ) VALUES ('actor-t137', 'stage-t137', 'session-t137', NULL, 'UNAVAILABLE', 105)",
            [],
        )
        .unwrap();
    drop(store);
    Fixture {
        root,
        home,
        repo: canonical_repo,
    }
}

fn candidate(repo: &Path) -> CandidateBaselineIdentity {
    let oid = git(repo, &["rev-parse", "HEAD"]);
    let tree = git(repo, &["rev-parse", "HEAD^{tree}"]);
    CandidateBaselineIdentity::new(&oid, &tree).unwrap()
}

fn add_baseline(
    fixture: &Fixture,
    baseline_id: &str,
    kind: ArtifactBaselineKind,
    reference: &str,
    candidate: Option<CandidateBaselineIdentity>,
    now: i64,
) {
    let store = Store::open(&fixture.home).unwrap();
    let baseline =
        ArtifactBaselineIdentity::new(baseline_id, "stage-t137", kind, reference, candidate)
            .unwrap();
    store.create_artifact_baseline(&baseline, now).unwrap();
}

fn eligible_report(
    fixture: &Fixture,
    run_id: &str,
    candidate: &CandidateBaselineIdentity,
) -> EvidenceReport {
    EvidenceReport {
        schema_version: 1,
        run_id: run_id.to_owned(),
        authority: "WINDS_OBSERVED",
        repo_path: fixture.repo.to_str().unwrap().to_owned(),
        base_oid: candidate.oid.clone(),
        candidate_ref: "refs/heads/t137-fixture".to_owned(),
        candidate_oid: candidate.oid.clone(),
        candidate_tree: candidate.tree.clone(),
        worktree_path: fixture.repo.to_str().unwrap().to_owned(),
        check: CheckEvidence {
            authority: "WINDS_OBSERVED",
            command: "cargo test --locked".to_owned(),
            status: CheckStatus::Pass,
            exit_code: Some(0),
            duration_ms: 1,
            stdout: BlobEvidence {
                relative_path: "fixture/stdout".to_owned(),
                sha256: "0".repeat(64),
                captured_bytes: 0,
                truncated: false,
            },
            stderr: BlobEvidence {
                relative_path: "fixture/stderr".to_owned(),
                sha256: "1".repeat(64),
                captured_bytes: 0,
                truncated: false,
            },
        },
        eligibility: Eligibility::Eligible,
        warnings: Vec::new(),
    }
}

fn persist_evidence(
    fixture: &Fixture,
    run_id: &str,
    candidate: &CandidateBaselineIdentity,
    now: i64,
) {
    let mut store = Store::open(&fixture.home).unwrap();
    store
        .create_run(
            NewRun {
                run_id,
                repo_path: fixture.repo.to_str().unwrap(),
                base_oid: &candidate.oid,
                candidate_ref: "refs/heads/t137-fixture",
                candidate_oid: &candidate.oid,
                candidate_tree: &candidate.tree,
                worktree_path: fixture.repo.to_str().unwrap(),
                check_command: "cargo test --locked",
                timeout_secs: 60,
            },
            now,
        )
        .unwrap();
    store.mark_workspace_ready(run_id, now + 1).unwrap();
    store
        .save_evidence_for_test(&eligible_report(fixture, run_id, candidate), now + 2)
        .unwrap();
}

fn bind(fixture: &Fixture) -> crate::desktop_files::DesktopRightDockBinding {
    desktop_right_dock_bind(
        &fixture.home,
        DesktopRightDockTarget {
            workspace_id: "workspace-t137".to_owned(),
            session_id: "session-t137".to_owned(),
        },
    )
    .unwrap()
}

fn verification_snapshot(fixture: &Fixture) -> Vec<String> {
    let store = Store::open(&fixture.home).unwrap();
    let runs = store
        .connection
        .prepare("SELECT run_id || '|' || state FROM candidate_runs ORDER BY run_id")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    let reports = store
        .connection
        .prepare("SELECT run_id || '|' || eligibility || '|' || report_json FROM evidence_reports ORDER BY run_id")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    runs.into_iter().chain(reports).collect()
}

#[test]
fn evidence_context_and_artifacts_are_read_only_exact_binding_projections() {
    let fixture = fixture("projection");
    let current = candidate(&fixture.repo);
    add_baseline(
        &fixture,
        "candidate-current",
        ArtifactBaselineKind::ExactGitCandidate,
        "candidate:current",
        Some(current.clone()),
        110,
    );
    persist_evidence(&fixture, "verify-current", &current, 120);
    add_baseline(
        &fixture,
        "evidence-current",
        ArtifactBaselineKind::WindsVerificationEvidence,
        "verify-current",
        Some(current.clone()),
        130,
    );
    add_baseline(
        &fixture,
        "stage-output",
        ArtifactBaselineKind::PriorStageOutput,
        "stage-output:inspect",
        None,
        140,
    );
    let binding = bind(&fixture);
    let before = verification_snapshot(&fixture);

    let evidence = desktop_right_dock_evidence(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap();
    assert_eq!(evidence.binding, binding);
    assert_eq!(evidence.entries.len(), 1);
    assert_eq!(evidence.entries[0].run_id, "verify-current");
    assert_eq!(evidence.entries[0].authority, "WINDS_OBSERVED");
    assert_eq!(evidence.entries[0].eligibility, "ELIGIBLE");
    assert_eq!(
        evidence.entries[0].freshness,
        DesktopEvidenceFreshness::Current
    );
    assert!(evidence.entries[0].trusted);

    let context = desktop_right_dock_context(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap();
    for (key, value) in [
        ("session", "session-t137"),
        ("workflow", "workflow-t137"),
        ("stage", "stage-t137"),
        ("stage_lifecycle", "PREPARED"),
        ("continuity", "UNAVAILABLE"),
    ] {
        assert!(
            context
                .facts
                .iter()
                .any(|fact| fact.key == key && fact.value == value),
            "missing context fact {key}={value}"
        );
    }
    assert!(
        context
            .facts
            .iter()
            .all(|fact| fact.source != "AGENT_REPORTED")
    );

    let artifacts =
        desktop_right_dock_artifacts(&fixture.home, DesktopBoundDockRequest { binding }).unwrap();
    assert_eq!(artifacts.entries.len(), 3);
    assert!(
        artifacts
            .entries
            .iter()
            .all(|entry| !entry.grants_verification_authority)
    );
    let evidence_artifact = artifacts
        .entries
        .iter()
        .find(|entry| entry.baseline_id == "evidence-current")
        .unwrap();
    assert_eq!(
        evidence_artifact.candidate_state,
        DesktopArtifactCandidateState::Current
    );
    assert_eq!(
        evidence_artifact.provenance,
        "WINDS_VERIFICATION_EVIDENCE_REFERENCE"
    );
    assert_eq!(verification_snapshot(&fixture), before);
}

#[test]
fn candidate_movement_removes_trusted_evidence_and_stales_artifact_treatment() {
    let fixture = fixture("candidate-movement");
    let candidate_a = candidate(&fixture.repo);
    add_baseline(
        &fixture,
        "candidate-a",
        ArtifactBaselineKind::ExactGitCandidate,
        "candidate:a",
        Some(candidate_a.clone()),
        110,
    );
    persist_evidence(&fixture, "verify-a", &candidate_a, 120);
    add_baseline(
        &fixture,
        "evidence-a",
        ArtifactBaselineKind::WindsVerificationEvidence,
        "verify-a",
        Some(candidate_a),
        130,
    );
    let old_binding = bind(&fixture);

    fs::write(fixture.repo.join("README.md"), "candidate B\n").unwrap();
    git(&fixture.repo, &["add", "README.md"]);
    git(&fixture.repo, &["commit", "-qm", "candidate-b"]);
    let candidate_b = candidate(&fixture.repo);
    add_baseline(
        &fixture,
        "candidate-b",
        ArtifactBaselineKind::ExactGitCandidate,
        "candidate:b",
        Some(candidate_b),
        140,
    );
    let binding = bind(&fixture);

    let old_error = desktop_right_dock_evidence(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: old_binding,
        },
    )
    .unwrap_err()
    .to_string();
    assert!(old_error.contains("binding is stale"), "{old_error}");

    let evidence = desktop_right_dock_evidence(
        &fixture.home,
        DesktopBoundDockRequest {
            binding: binding.clone(),
        },
    )
    .unwrap();
    assert_eq!(evidence.entries.len(), 1);
    assert_eq!(
        evidence.entries[0].freshness,
        DesktopEvidenceFreshness::Stale
    );
    assert!(!evidence.entries[0].trusted);

    let artifacts =
        desktop_right_dock_artifacts(&fixture.home, DesktopBoundDockRequest { binding }).unwrap();
    let stale = artifacts
        .entries
        .iter()
        .find(|entry| entry.baseline_id == "evidence-a")
        .unwrap();
    assert_eq!(stale.candidate_state, DesktopArtifactCandidateState::Stale);
    assert!(!stale.grants_verification_authority);
}

#[test]
fn forged_verification_reference_fails_closed_in_evidence_surface() {
    let fixture = fixture("forged-evidence");
    let current = candidate(&fixture.repo);
    add_baseline(
        &fixture,
        "candidate-current",
        ArtifactBaselineKind::ExactGitCandidate,
        "candidate:current",
        Some(current.clone()),
        110,
    );
    add_baseline(
        &fixture,
        "forged-evidence-baseline",
        ArtifactBaselineKind::WindsVerificationEvidence,
        "agent-says-verified",
        Some(current),
        120,
    );
    let binding = bind(&fixture);
    let error = desktop_right_dock_evidence(&fixture.home, DesktopBoundDockRequest { binding })
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown Winds run"), "{error}");
}
