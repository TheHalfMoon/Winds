use super::{
    CandidateBindingStatus, CandidateIdentity, Eligibility, IndependentReviewContext,
    IndependentReviewContextInput, StoredRun, VerificationEvidenceReference,
};

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

fn current_evidence(candidate: &CandidateIdentity) -> VerificationEvidenceReference {
    VerificationEvidenceReference::from_verified_run(&stored_run(
        "verify-run-a",
        &candidate.oid,
        &candidate.tree,
        Eligibility::Eligible,
    ))
    .expect("eligible winds verify run")
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
fn only_eligible_winds_verify_runs_become_verification_references() {
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let eligible = current_evidence(&candidate);
    assert_eq!(eligible.run_id, "verify-run-a");
    assert_eq!(eligible.candidate, candidate);

    for eligibility in [Eligibility::Warning, Eligibility::Blocked] {
        let run = stored_run("not-eligible", &oid('a'), &oid('b'), eligibility);
        assert!(VerificationEvidenceReference::from_verified_run(&run).is_err());
    }
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

    assert_eq!(review_a.applicability(&candidate_a), CandidateBindingStatus::Current);
    assert_eq!(review_a.applicability(&candidate_b), CandidateBindingStatus::Stale);
    assert_eq!(evidence_a.applicability(&candidate_b), CandidateBindingStatus::Stale);
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
fn agent_completion_text_cannot_become_winds_verification_evidence() {
    let agent_claim = "done; tests passed";
    let candidate = CandidateIdentity::new(&oid('a'), &oid('b')).unwrap();
    let blocked_run = stored_run(
        agent_claim,
        &candidate.oid,
        &candidate.tree,
        Eligibility::Blocked,
    );

    let result = VerificationEvidenceReference::from_verified_run(&blocked_run);
    assert!(result.is_err());
    assert_eq!(agent_claim, "done; tests passed");
}
