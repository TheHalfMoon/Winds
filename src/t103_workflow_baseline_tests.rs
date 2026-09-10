use super::{NewWorkspace, NewWorkstream, Store};
use crate::domain::workflow::{
    ArtifactBaselineIdentity, ArtifactBaselineKind, ArtifactBaselineRequirement, BaselineFreshness,
    CandidateBaselineIdentity, StageRunIdentity, WorkflowRunIdentity,
    evaluate_artifact_baseline_requirement,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

fn test_home(name: &str) -> PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t103-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&home).unwrap();
    home
}

fn cleanup(home: PathBuf) {
    if home.exists() {
        fs::remove_dir_all(home).unwrap();
    }
}

fn seeded_store(name: &str) -> (PathBuf, Store) {
    let home = test_home(name);
    let store = Store::open(&home).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: "/tmp/t103-workspace-1",
                git_common_dir: "/tmp/t103-git-1",
            },
            1,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "T103 workstream",
            },
            2,
        )
        .unwrap();
    let workflow = WorkflowRunIdentity::new("workflow-1", "workspace-1", "workstream-1").unwrap();
    store.create_workflow_run(&workflow, 3).unwrap();
    let stage = StageRunIdentity::new("stage-1", "workflow-1", "build", 1, None).unwrap();
    store.create_stage_run(&stage, None, 4).unwrap();
    (home, store)
}

fn candidate(hex: char) -> CandidateBaselineIdentity {
    let oid = hex.to_string().repeat(40);
    let tree = if hex == 'a' { "b" } else { "c" }.repeat(40);
    CandidateBaselineIdentity::new(&oid, &tree).unwrap()
}

fn exact_candidate_baseline(
    baseline_id: &str,
    stage_run_id: &str,
    candidate: CandidateBaselineIdentity,
) -> ArtifactBaselineIdentity {
    ArtifactBaselineIdentity::new(
        baseline_id,
        stage_run_id,
        ArtifactBaselineKind::ExactGitCandidate,
        "canonical-candidate",
        Some(candidate),
    )
    .unwrap()
}

#[test]
fn t103_candidate_movement_is_stale_and_exact_identity_is_applicable() {
    let candidate_a = candidate('a');
    let candidate_b = candidate('d');
    let observed = exact_candidate_baseline("baseline-a", "stage-1", candidate_a.clone());

    let requirement_a = ArtifactBaselineRequirement::new(
        "stage-1",
        ArtifactBaselineKind::ExactGitCandidate,
        "canonical-candidate",
        Some(candidate_a),
    )
    .unwrap();
    let exact =
        evaluate_artifact_baseline_requirement(&requirement_a, std::slice::from_ref(&observed));
    assert_eq!(exact.freshness, BaselineFreshness::Applicable);
    assert_eq!(exact.matching_baseline_ids, vec!["baseline-a"]);

    let requirement_b = ArtifactBaselineRequirement::new(
        "stage-1",
        ArtifactBaselineKind::ExactGitCandidate,
        "canonical-candidate",
        Some(candidate_b),
    )
    .unwrap();
    let moved = evaluate_artifact_baseline_requirement(&requirement_b, &[observed]);
    assert_eq!(moved.freshness, BaselineFreshness::Stale);
    assert_eq!(moved.matching_baseline_ids, vec!["baseline-a"]);
}

#[test]
fn t103_stage_identity_and_exact_reference_prevent_path_or_name_similarity_from_proving_freshness()
{
    let candidate_a = candidate('a');
    let old_stage = exact_candidate_baseline("old-stage", "stage-1", candidate_a.clone());
    let new_stage_requirement = ArtifactBaselineRequirement::new(
        "stage-2",
        ArtifactBaselineKind::ExactGitCandidate,
        "canonical-candidate",
        Some(candidate_a.clone()),
    )
    .unwrap();
    assert_eq!(
        evaluate_artifact_baseline_requirement(&new_stage_requirement, &[old_stage]).freshness,
        BaselineFreshness::Missing
    );

    let similarly_named = ArtifactBaselineIdentity::new(
        "similar-name",
        "stage-2",
        ArtifactBaselineKind::PriorStageOutput,
        "outputs/build/result.json",
        None,
    )
    .unwrap();
    let exact_reference = ArtifactBaselineRequirement::new(
        "stage-2",
        ArtifactBaselineKind::PriorStageOutput,
        "outputs/build/./result.json",
        None,
    )
    .unwrap();
    assert_eq!(
        evaluate_artifact_baseline_requirement(&exact_reference, &[similarly_named]).freshness,
        BaselineFreshness::Missing
    );
}

#[test]
fn t103_missing_and_ambiguous_requirements_fail_closed_with_deterministic_blocker_ids() {
    let requirement = ArtifactBaselineRequirement::new(
        "stage-1",
        ArtifactBaselineKind::PriorStageOutput,
        "stage-output:compile",
        None,
    )
    .unwrap();
    let missing = evaluate_artifact_baseline_requirement(&requirement, &[]);
    assert_eq!(missing.freshness, BaselineFreshness::Missing);
    assert!(missing.matching_baseline_ids.is_empty());

    let first = ArtifactBaselineIdentity::new(
        "baseline-z",
        "stage-1",
        ArtifactBaselineKind::PriorStageOutput,
        "stage-output:compile",
        None,
    )
    .unwrap();
    let second = ArtifactBaselineIdentity::new(
        "baseline-a",
        "stage-1",
        ArtifactBaselineKind::PriorStageOutput,
        "stage-output:compile",
        None,
    )
    .unwrap();
    let ambiguous =
        evaluate_artifact_baseline_requirement(&requirement, &[first.clone(), second.clone()]);
    let repeated = evaluate_artifact_baseline_requirement(&requirement, &[first, second]);
    assert_eq!(ambiguous, repeated);
    assert_eq!(ambiguous.freshness, BaselineFreshness::Ambiguous);
    assert_eq!(
        ambiguous.matching_baseline_ids,
        vec!["baseline-a", "baseline-z"]
    );
}

#[test]
fn t103_closed_baseline_validation_rejects_incomplete_candidate_and_unbounded_blob_identity() {
    assert!(
        ArtifactBaselineIdentity::new(
            "baseline-1",
            "stage-1",
            ArtifactBaselineKind::ExactGitCandidate,
            "canonical-candidate",
            None,
        )
        .is_err()
    );
    assert!(CandidateBaselineIdentity::new("ABC", "def").is_err());
    assert!(
        ArtifactBaselineIdentity::new(
            "baseline-2",
            "stage-1",
            ArtifactBaselineKind::BoundedBlobArtifact,
            "artifact.txt",
            None,
        )
        .is_err()
    );
    let digest = format!("sha256:{}", "a".repeat(64));
    assert!(
        ArtifactBaselineIdentity::new(
            "baseline-3",
            "stage-1",
            ArtifactBaselineKind::BoundedBlobArtifact,
            &digest,
            None,
        )
        .is_ok()
    );
}

#[test]
fn t103_store_persists_immutable_history_across_restart_and_evaluates_only_exact_stage_inputs() {
    let (home, store) = seeded_store("restart-history");
    let baseline = exact_candidate_baseline("baseline-a", "stage-1", candidate('a'));
    store.create_artifact_baseline(&baseline, 5).unwrap();
    assert!(store.create_artifact_baseline(&baseline, 6).is_err());
    assert!(
        store
            .connection
            .execute(
                "UPDATE workflow_artifact_baselines SET stable_reference = 'rewritten' WHERE baseline_id = 'baseline-a'",
                [],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM workflow_artifact_baselines WHERE baseline_id = 'baseline-a'",
                [],
            )
            .is_err()
    );
    drop(store);

    let store = Store::open(&home).unwrap();
    let loaded = store.load_artifact_baseline("baseline-a").unwrap();
    assert_eq!(loaded.identity, baseline);
    assert_eq!(loaded.created_unix_ms, 5);
    let listed = store.list_artifact_baselines_for_stage("stage-1").unwrap();
    assert_eq!(listed, vec![loaded]);

    let requirement = ArtifactBaselineRequirement::new(
        "stage-1",
        ArtifactBaselineKind::ExactGitCandidate,
        "canonical-candidate",
        Some(candidate('a')),
    )
    .unwrap();
    assert_eq!(
        store
            .evaluate_artifact_baseline(&requirement)
            .unwrap()
            .freshness,
        BaselineFreshness::Applicable
    );
    let moved = ArtifactBaselineRequirement::new(
        "stage-1",
        ArtifactBaselineKind::ExactGitCandidate,
        "canonical-candidate",
        Some(candidate('d')),
    )
    .unwrap();
    assert_eq!(
        store.evaluate_artifact_baseline(&moved).unwrap().freshness,
        BaselineFreshness::Stale
    );

    drop(store);
    cleanup(home);
}

#[test]
fn t103_store_rejects_orphan_and_malformed_durable_baseline_truth_on_read() {
    let (home, store) = seeded_store("malformed-read");
    assert!(
        store
            .connection
            .execute(
                "INSERT INTO workflow_artifact_baselines(
                    baseline_id, stage_run_id, baseline_kind, stable_reference, created_unix_ms
                 ) VALUES ('orphan', 'missing-stage', 'PRIOR_STAGE_OUTPUT', 'stage-output:x', 5)",
                [],
            )
            .is_err()
    );

    store
        .connection
        .execute(
            "INSERT INTO workflow_artifact_baselines(
                baseline_id, stage_run_id, baseline_kind, stable_reference, created_unix_ms
             ) VALUES ('missing-candidate', 'stage-1', 'EXACT_GIT_CANDIDATE', 'canonical-candidate', 5)",
            [],
        )
        .unwrap();
    assert!(store.load_artifact_baseline("missing-candidate").is_err());

    let uppercase = "A".repeat(40);
    let lowercase = "b".repeat(40);
    store
        .connection
        .execute(
            "INSERT INTO workflow_artifact_baselines(
                baseline_id, stage_run_id, baseline_kind, stable_reference,
                candidate_oid, candidate_tree, created_unix_ms
             ) VALUES ('malformed-candidate', 'stage-1', 'EXACT_GIT_CANDIDATE',
                       'canonical-candidate', ?1, ?2, 6)",
            rusqlite::params![uppercase, lowercase],
        )
        .unwrap();
    assert!(store.load_artifact_baseline("malformed-candidate").is_err());

    drop(store);
    cleanup(home);
}
