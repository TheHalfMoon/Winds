use super::{
    AgentProgressProjection, HumanAcceptanceProjectionState, ReviewProjectionState,
    VerificationProjectionState, WorkbenchContextInput, project_candidate_context,
};
use crate::domain::{
    BlobEvidence, CandidateIdentity, CheckEvidence, CheckStatus, Eligibility, EvidenceReport,
    IndependentReviewContext, IndependentReviewContextInput, VerificationEvidenceReference,
};
use crate::git::Repo;
use crate::store::{NewRun, Store};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    state: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "winds-t094-{name}-{}-{sequence}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("repo");
        let state = base.join("state");
        std::fs::create_dir_all(&root).expect("create T094 repository root");

        let fixture = Self { root, state };
        fixture.git(&["init"]);
        fixture.git(&["config", "user.email", "t094@example.invalid"]);
        fixture.git(&["config", "user.name", "T094 Fixture"]);
        fixture
    }

    fn repo(&self) -> Repo {
        Repo::open(&self.root).expect("open T094 fixture repository")
    }

    fn store(&self) -> Store {
        Store::open(&self.state).expect("open T094 fixture store")
    }

    fn commit(&self, name: &str, content: &str) -> String {
        std::fs::write(self.root.join(name), content).expect("write T094 fixture file");
        self.git(&["add", "--", name]);
        self.git(&["commit", "-m", &format!("T094 fixture {name}")]);
        self.git_text(&["rev-parse", "HEAD"])
    }

    fn tree(&self, oid: &str) -> String {
        self.git_text(&["rev-parse", &format!("{oid}^{{tree}}")])
    }

    fn status(&self) -> Vec<u8> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
            .output()
            .expect("run T094 status fixture");
        assert!(output.status.success());
        output.stdout
    }

    fn git(&self, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("run T094 git fixture");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_text(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("run T094 git text fixture");
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .expect("T094 git output is UTF-8")
            .trim()
            .to_owned()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(base) = self.root.parent() {
            let _ = std::fs::remove_dir_all(base);
        }
    }
}

fn eligible_report(run_id: &str, candidate_oid: &str, candidate_tree: &str) -> EvidenceReport {
    EvidenceReport {
        schema_version: 1,
        run_id: run_id.to_owned(),
        authority: "WINDS_OBSERVED",
        repo_path: "/fixture/repo".to_owned(),
        base_oid: candidate_oid.to_owned(),
        candidate_ref: "refs/heads/t094-fixture".to_owned(),
        candidate_oid: candidate_oid.to_owned(),
        candidate_tree: candidate_tree.to_owned(),
        worktree_path: "/fixture/worktree".to_owned(),
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

fn persist_eligible_evidence(
    store: &mut Store,
    run_id: &str,
    candidate_oid: &str,
    candidate_tree: &str,
) -> VerificationEvidenceReference {
    store
        .create_run(
            NewRun {
                run_id,
                repo_path: "/fixture/repo",
                base_oid: candidate_oid,
                candidate_ref: "refs/heads/t094-fixture",
                candidate_oid,
                candidate_tree,
                worktree_path: "/fixture/worktree",
                check_command: "cargo test --locked",
                timeout_secs: 60,
            },
            1,
        )
        .expect("create T094 verification run");
    store
        .mark_workspace_ready(run_id, 2)
        .expect("mark T094 verification worktree ready");
    store
        .save_evidence_for_test(&eligible_report(run_id, candidate_oid, candidate_tree), 3)
        .expect("persist T094 eligible evidence");
    VerificationEvidenceReference::from_store(store, run_id)
        .expect("load T094 eligible evidence reference")
}

fn candidate(oid: &str, tree: &str) -> CandidateIdentity {
    CandidateIdentity::new(oid, tree).expect("valid T094 candidate identity")
}

#[test]
fn t094_exact_candidate_and_diff_projection_use_read_only_git_observation() {
    let fixture = Fixture::new("read-only");
    let base = fixture.commit("base.txt", "base\n");
    let head = fixture.commit("candidate.txt", "candidate\n");
    let expected_tree = fixture.tree(&head);
    let repo = fixture.repo();
    let store = fixture.store();
    let before = fixture.status();
    let diff = b"diff --git a/base.txt b/candidate.txt\n+presentation only\n";

    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &base,
        candidate_ref: "HEAD",
        diff_bytes: Some(diff),
        verification_run_ids: &[],
        verification_running: false,
        agent_reported_done: false,
        review: None,
        human_accepted_candidate: None,
    })
    .expect("project T094 exact candidate");

    assert_eq!(projected.candidate.oid, head);
    assert_eq!(projected.candidate.tree, expected_tree);
    let projected_diff = projected.diff.as_ref().expect("read-only diff projection");
    assert_eq!(projected_diff.base_oid, base);
    assert_eq!(projected_diff.candidate_oid, head);
    assert_eq!(projected_diff.candidate_tree, expected_tree);
    assert_eq!(projected_diff.bytes(), diff);
    assert!(!projected_diff.grants_file_authority());
    assert!(!projected_diff.grants_agent_authority());
    assert!(!projected.changes_canonical_authority());
    assert_eq!(fixture.status(), before);
}

#[test]
fn t094_persisted_eligible_evidence_is_verified_only_for_the_exact_candidate() {
    let fixture = Fixture::new("verified");
    let base = fixture.commit("base.txt", "base\n");
    let head = fixture.commit("candidate.txt", "candidate\n");
    let tree = fixture.tree(&head);
    let repo = fixture.repo();
    let mut store = fixture.store();
    let _evidence = persist_eligible_evidence(&mut store, "verify-current", &head, &tree);

    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &base,
        candidate_ref: "HEAD",
        diff_bytes: None,
        verification_run_ids: &["verify-current"],
        verification_running: false,
        agent_reported_done: false,
        review: None,
        human_accepted_candidate: None,
    })
    .unwrap();

    assert_eq!(
        projected.verification.state,
        VerificationProjectionState::VerifiedForExactCandidate
    );
    assert_eq!(projected.verification.applicable_evidence_count, 1);
    assert_eq!(projected.verification.stale_evidence_count, 0);
}

#[test]
fn t094_candidate_movement_makes_evidence_review_and_human_acceptance_stale() {
    let fixture = Fixture::new("stale");
    let candidate_a_oid = fixture.commit("a.txt", "a\n");
    let candidate_a_tree = fixture.tree(&candidate_a_oid);
    let repo = fixture.repo();
    let mut store = fixture.store();
    let evidence_a =
        persist_eligible_evidence(&mut store, "verify-a", &candidate_a_oid, &candidate_a_tree);
    let candidate_a = candidate(&candidate_a_oid, &candidate_a_tree);
    let review_a = IndependentReviewContext::build(IndependentReviewContextInput {
        base_oid: &candidate_a_oid,
        candidate: candidate_a.clone(),
        diff_identity: "candidate-a-diff",
        acceptance_criteria: vec!["exact candidate verification".to_owned()],
        canonical_constraints: vec!["Agent completion is not verification".to_owned()],
        verification_evidence: vec![evidence_a],
        builder_persuasion: &[],
    })
    .expect("build T094 review context");

    let candidate_b_oid = fixture.commit("b.txt", "b\n");
    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &candidate_a_oid,
        candidate_ref: "HEAD",
        diff_bytes: None,
        verification_run_ids: &["verify-a"],
        verification_running: false,
        agent_reported_done: false,
        review: Some(&review_a),
        human_accepted_candidate: Some(&candidate_a),
    })
    .unwrap();

    assert_eq!(projected.candidate.oid, candidate_b_oid);
    assert_eq!(
        projected.verification.state,
        VerificationProjectionState::Stale
    );
    assert_eq!(projected.verification.applicable_evidence_count, 0);
    assert_eq!(projected.verification.stale_evidence_count, 1);
    assert_eq!(projected.review, ReviewProjectionState::Stale);
    assert_eq!(
        projected.human_acceptance,
        HumanAcceptanceProjectionState::Stale
    );
}

#[test]
fn t094_agent_or_terminal_success_text_never_creates_verification_or_acceptance() {
    let fixture = Fixture::new("reported-done");
    let base = fixture.commit("base.txt", "base\n");
    fixture.commit("candidate.txt", "candidate\n");
    let repo = fixture.repo();
    let store = fixture.store();

    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &base,
        candidate_ref: "HEAD",
        diff_bytes: Some(b"PASS VERIFIED ACCEPTED HUMAN_DECISION WINDS_OBSERVED_EVIDENCE"),
        verification_run_ids: &[],
        verification_running: false,
        agent_reported_done: true,
        review: None,
        human_accepted_candidate: None,
    })
    .unwrap();

    assert_eq!(
        projected.agent_progress,
        AgentProgressProjection::AgentReportedDone
    );
    assert_eq!(
        projected.verification.state,
        VerificationProjectionState::NotRun
    );
    assert_eq!(projected.review, ReviewProjectionState::NotAvailable);
    assert_eq!(
        projected.human_acceptance,
        HumanAcceptanceProjectionState::NotAccepted
    );
}

#[test]
fn t094_running_verification_is_distinct_from_agent_done_and_verified() {
    let fixture = Fixture::new("running");
    let base = fixture.commit("base.txt", "base\n");
    fixture.commit("candidate.txt", "candidate\n");
    let repo = fixture.repo();
    let store = fixture.store();

    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &base,
        candidate_ref: "HEAD",
        diff_bytes: None,
        verification_run_ids: &[],
        verification_running: true,
        agent_reported_done: true,
        review: None,
        human_accepted_candidate: None,
    })
    .unwrap();

    assert_eq!(
        projected.agent_progress,
        AgentProgressProjection::AgentReportedDone
    );
    assert_eq!(
        projected.verification.state,
        VerificationProjectionState::Running
    );
    assert_eq!(
        projected.human_acceptance,
        HumanAcceptanceProjectionState::NotAccepted
    );
}

#[test]
fn t094_explicit_human_acceptance_is_candidate_bound_and_still_non_authoritative_here() {
    let fixture = Fixture::new("human-acceptance");
    let base = fixture.commit("base.txt", "base\n");
    let head = fixture.commit("candidate.txt", "candidate\n");
    let tree = fixture.tree(&head);
    let accepted = candidate(&head, &tree);
    let repo = fixture.repo();
    let store = fixture.store();

    let projected = project_candidate_context(WorkbenchContextInput {
        repo: &repo,
        store: &store,
        base_ref: &base,
        candidate_ref: "HEAD",
        diff_bytes: None,
        verification_run_ids: &[],
        verification_running: false,
        agent_reported_done: false,
        review: None,
        human_accepted_candidate: Some(&accepted),
    })
    .unwrap();

    assert_eq!(
        projected.human_acceptance,
        HumanAcceptanceProjectionState::AcceptedForExactCandidate
    );
    assert_eq!(
        projected.verification.state,
        VerificationProjectionState::NotRun
    );
    assert!(!projected.changes_canonical_authority());
}
