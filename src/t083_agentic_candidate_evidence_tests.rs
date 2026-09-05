use super::{
    BlobEvidence, CandidateBindingStatus, CandidateIdentity, CheckEvidence, CheckStatus,
    Eligibility, EvidenceReport, IndependentReviewContext, IndependentReviewContextInput,
    StoredRun, VerificationEvidenceReference,
};
use crate::store::{NewRun, Store};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(1);
const OUTPUT_TRUNCATED: &str =
    "required check output exceeded the capture cap; evidence is incomplete";

struct TestHome {
    path: PathBuf,
}

impl TestHome {
    fn new(name: &str) -> Self {
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t083-{name}-{}-{sequence}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn oid(ch: char) -> String {
    ch.to_string().repeat(40)
}

fn stored_run(
    run_id: &str,
    candidate_oid: &str,
    candidate_tree: &str,
    eligibility: Eligibility,
) -> StoredRun {
    StoredRun {
        run_id: run_id.to_owned(),
        repo_path: "/fixture/repo".to_owned(),
        candidate_oid: candidate_oid.to_owned(),
        candidate_tree: candidate_tree.to_owned(),
        worktree_path: "/fixture/worktree".to_owned(),
        check_command: "cargo test --locked".to_owned(),
        timeout_secs: 60,
        eligibility,
    }
}

fn evidence_report(
    run_id: &str,
    candidate: &CandidateIdentity,
    eligibility: Eligibility,
) -> EvidenceReport {
    let (status, exit_code, stdout_truncated, warnings) = match &eligibility {
        Eligibility::Eligible => (CheckStatus::Pass, Some(0), false, Vec::new()),
        Eligibility::Warning => (
            CheckStatus::Pass,
            Some(0),
            true,
            vec![OUTPUT_TRUNCATED.to_owned()],
        ),
        Eligibility::Blocked => (CheckStatus::Fail, Some(1), false, Vec::new()),
    };

    EvidenceReport {
        schema_version: 1,
        run_id: run_id.to_owned(),
        authority: "WINDS_OBSERVED",
        repo_path: "/fixture/repo".to_owned(),
        base_oid: oid('e'),
        candidate_ref: "refs/heads/t083-fixture".to_owned(),
        candidate_oid: candidate.oid.clone(),
        candidate_tree: candidate.tree.clone(),
        worktree_path: "/fixture/worktree".to_owned(),
        check: CheckEvidence {
            authority: "WINDS_OBSERVED",
            command: "cargo test --locked".to_owned(),
            status,
            exit_code,
            duration_ms: 1,
            stdout: BlobEvidence {
                relative_path: "fixture/stdout".to_owned(),
                sha256: "0".repeat(64),
                captured_bytes: 0,
                truncated: stdout_truncated,
            },
            stderr: BlobEvidence {
                relative_path: "fixture/stderr".to_owned(),
                sha256: "1".repeat(64),
                captured_bytes: 0,
                truncated: false,
            },
        },
        eligibility,
        warnings,
    }
}

fn ready_store(name: &str, run_id: &str, candidate: &CandidateIdentity) -> (TestHome, Store) {
    let home = TestHome::new(name);
    let mut store = Store::open(home.path()).expect("open T083 fixture store");
    let base_oid = oid('e');
    store
        .create_run(
            NewRun {
                run_id,
                repo_path: "/fixture/repo",
                base_oid: &base_oid,
                candidate_ref: "refs/heads/t083-fixture",
                candidate_oid: &candidate.oid,
                candidate_tree: &candidate.tree,
                worktree_path: "/fixture/worktree",
                check_command: "cargo test --locked",
                timeout_secs: 60,
            },
            1,
        )
        .expect("persist T083 candidate run");
    store
        .mark_workspace_ready(run_id, 2)
        .expect("mark T083 fixture worktree ready");
    (home, store)
}

fn persisted_store(
    name: &str,
    run_id: &str,
    candidate: &CandidateIdentity,
    eligibility: Eligibility,
) -> (TestHome, Store) {
    let (home, mut store) = ready_store(name, run_id, candidate);
    let report = evidence_report(run_id, candidate, eligibility);
    store
        .save_evidence(&report, 3)
        .expect("persist T083 verification evidence");
    (home, store)
}

fn current_evidence(candidate: &CandidateIdentity) -> VerificationEvidenceReference {
    let (_home, store) = persisted_store(
        "current-evidence",
        "verify-run-a",
        candidate,
        Eligibility::Eligible,
    );
    VerificationEvidenceReference::from_store(&store, "verify-run-a")
        .expect("persisted eligible winds verify run")
}

#[test]
fn exact_oid_and_tree_form_the_candidate_acceptance_identity() {
    let candidate = CandidateIdentity::new(&oid('A'), &oid('B')).expect("exact candidate identity");
    assert_eq!(candidate.oid, oid('a'));
    assert_eq!(candidate.tree, oid('b'));

    assert!(CandidateIdentity::new("abc123", &oid('b')).is_err());
    assert!(CandidateIdentity::new(&oid('a'), "tree-not-an-object-id").is_err());
}

#[test]
fn only_persisted_eligible_winds_verify_runs_become_verification_references() {
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let eligible = current_evidence(&candidate);
    assert_eq!(eligible.run_id, "verify-run-a");
    assert_eq!(eligible.candidate, candidate);

    for (name, eligibility) in [
        ("warning", Eligibility::Warning),
        ("blocked", Eligibility::Blocked),
    ] {
        let (_home, store) = persisted_store(name, "not-eligible", &candidate, eligibility);
        assert!(VerificationEvidenceReference::from_store(&store, "not-eligible").is_err());
    }
}

#[test]
fn invented_eligible_report_with_blocked_check_cannot_be_persisted() {
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let (_home, mut store) = ready_store("invented-eligible", "invented-eligible", &candidate);
    let mut report = evidence_report("invented-eligible", &candidate, Eligibility::Blocked);
    report.eligibility = Eligibility::Eligible;

    assert!(store.save_evidence(&report, 3).is_err());
    assert_eq!(
        store.load_run("invented-eligible").unwrap().eligibility,
        Eligibility::Blocked
    );
}

#[test]
fn persisted_evidence_candidate_oid_or_tree_mismatch_fails_closed() {
    for (name, mutate) in [
        ("oid-mismatch", true),
        ("tree-mismatch", false),
    ] {
        let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
        let (_home, mut store) = ready_store(name, name, &candidate);
        let mut report = evidence_report(name, &candidate, Eligibility::Eligible);
        if mutate {
            report.candidate_oid = oid('c');
        } else {
            report.candidate_tree = oid('d');
        }

        assert!(store.save_evidence(&report, 3).is_err());
        assert_eq!(store.load_run(name).unwrap().eligibility, Eligibility::Blocked);
    }
}

#[test]
fn in_memory_eligible_stored_run_cannot_manufacture_verification_evidence() {
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let fabricated = stored_run(
        "fabricated-eligible",
        &candidate.oid,
        &candidate.tree,
        Eligibility::Eligible,
    );
    assert_eq!(fabricated.eligibility, Eligibility::Eligible);

    let home = TestHome::new("fabricated-run");
    let store = Store::open(home.path()).unwrap();
    let result = VerificationEvidenceReference::from_store(&store, &fabricated.run_id);
    assert!(result.is_err());
}

#[test]
fn candidate_movement_makes_old_review_and_evidence_stale_but_traceable() {
    let candidate_a = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let candidate_b = CandidateIdentity::new(&oid('c'), &oid('d')).unwrap();
    let evidence_a = current_evidence(&candidate_a);

    let review_a = IndependentReviewContext::build(IndependentReviewContextInput {
        base_oid: &oid('e'),
        candidate: candidate_a.clone(),
        diff_identity: "base...candidate-a",
        acceptance_criteria: vec!["exact candidate verification".to_owned()],
        canonical_constraints: vec!["Agent completion is not verification".to_owned()],
        verification_evidence: vec![evidence_a.clone()],
        builder_persuasion: &[],
    })
    .unwrap();

    assert_eq!(
        review_a.applicability(&candidate_a),
        CandidateBindingStatus::Current
    );
    assert_eq!(
        review_a.applicability(&candidate_b),
        CandidateBindingStatus::Stale
    );
    assert_eq!(
        evidence_a.applicability(&candidate_b),
        CandidateBindingStatus::Stale
    );
    assert_eq!(evidence_a.run_id, "verify-run-a");
    assert_eq!(evidence_a.candidate, candidate_a);
}

#[test]
fn review_context_contains_exact_review_inputs_and_excludes_builder_persuasion() {
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let evidence = current_evidence(&candidate);
    let persuasion = vec![
        "done; tests passed".to_owned(),
        "please merge this candidate".to_owned(),
    ];

    let context = IndependentReviewContext::build(IndependentReviewContextInput {
        base_oid: &oid('e'),
        candidate: candidate.clone(),
        diff_identity: "diff-sha256:0123456789abcdef",
        acceptance_criteria: vec![
            "zero unresolved material findings".to_owned(),
            "exact candidate verification".to_owned(),
            "exact candidate verification".to_owned(),
        ],
        canonical_constraints: vec![
            "landing remains human-decided".to_owned(),
            "Agent completion is not verification".to_owned(),
        ],
        verification_evidence: vec![evidence],
        builder_persuasion: &persuasion,
    })
    .unwrap();

    assert_eq!(context.base_oid, oid('e'));
    assert_eq!(context.candidate, candidate);
    assert_eq!(context.verification_evidence.len(), 1);
    assert_eq!(context.excluded_builder_persuasion_count, 2);
    assert_eq!(
        context.acceptance_criteria,
        vec![
            "exact candidate verification".to_owned(),
            "zero unresolved material findings".to_owned(),
        ]
    );

    let serialized = serde_json::to_string(&context).unwrap();
    assert!(serialized.contains("verify-run-a"));
    assert!(!serialized.contains("done; tests passed"));
    assert!(!serialized.contains("please merge this candidate"));
}

#[test]
fn stale_verification_evidence_cannot_be_reused_for_a_new_candidate_review() {
    let candidate_a = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let candidate_b = CandidateIdentity::new(&oid('c'), &oid('d')).unwrap();
    let evidence_a = current_evidence(&candidate_a);

    let result = IndependentReviewContext::build(IndependentReviewContextInput {
        base_oid: &oid('e'),
        candidate: candidate_b,
        diff_identity: "base...candidate-b",
        acceptance_criteria: vec!["exact candidate verification".to_owned()],
        canonical_constraints: vec!["stale evidence never becomes current".to_owned()],
        verification_evidence: vec![evidence_a],
        builder_persuasion: &["done; tests passed".to_owned()],
    });

    assert!(result.is_err());
}

#[test]
fn builder_persuasion_cannot_replace_missing_verification_evidence() {
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let result = IndependentReviewContext::build(IndependentReviewContextInput {
        base_oid: &oid('e'),
        candidate,
        diff_identity: "base...candidate-a",
        acceptance_criteria: vec!["exact candidate verification".to_owned()],
        canonical_constraints: vec!["Agent completion is not verification".to_owned()],
        verification_evidence: vec![],
        builder_persuasion: &["done; tests passed".to_owned()],
    });

    assert!(result.is_err());
}

#[test]
fn agent_completion_text_cannot_become_winds_verification_evidence() {
    let agent_claim = "done; tests passed";
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let (_home, store) =
        persisted_store("agent-claim", agent_claim, &candidate, Eligibility::Blocked);

    let result = VerificationEvidenceReference::from_store(&store, agent_claim);
    assert!(result.is_err());
    assert_eq!(agent_claim, "done; tests passed");
}
