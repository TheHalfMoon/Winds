use super::{
    ApprovalContent, ApprovalReason, AuthorityDecision, AuthorityPlane, AuthorityTarget,
    EnforcementEvidence, EnforcementQuality, HumanAction, approval_json_and_digest,
    ensure_approval_schema, load_human_approval, record_human_approval, revalidate_human_approval,
};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

struct TestEnvironment {
    owned_base: PathBuf,
    state_root: PathBuf,
    repo_root: PathBuf,
}

impl TestEnvironment {
    fn new(name: &str) -> Self {
        assert!(
            Path::new(name)
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        );
        let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
        let owned_base = std::env::temp_dir().join(format!(
            "winds-t076-approval-owned-{}-{sequence}-{name}",
            std::process::id()
        ));
        let state_root = owned_base.join("state");
        let repo_root = owned_base.join("repo");
        fs::create_dir(&owned_base).unwrap();
        fs::create_dir(&state_root).unwrap();
        fs::create_dir(&repo_root).unwrap();
        Self {
            owned_base,
            state_root,
            repo_root,
        }
    }

    fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn canonical_repo_root(&self) -> String {
        fs::canonicalize(&self.repo_root)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let Ok(canonical_base) = fs::canonicalize(&self.owned_base) else {
            return;
        };
        let _ = fs::remove_dir_all(canonical_base);
    }
}

fn target() -> AuthorityTarget {
    AuthorityTarget {
        capability: "edit".to_owned(),
        resource: "workspace:repo/src".to_owned(),
    }
}

fn plane(decision: AuthorityDecision) -> AuthorityPlane {
    AuthorityPlane {
        default_decision: AuthorityDecision::Deny,
        rules: BTreeMap::from([(target(), decision)]),
    }
}

fn base_content(repo_root: &str) -> ApprovalContent {
    ApprovalContent {
        workstream_id: "workstream-1".to_owned(),
        session_id: "session-1".to_owned(),
        planner_id: "planner-1".to_owned(),
        worker_id: "worker-1".to_owned(),
        worker_parent_planner_id: "planner-1".to_owned(),
        worker_role: "BUILDER".to_owned(),
        runtime_kind: "CODEX".to_owned(),
        workspace_id: "workspace-1".to_owned(),
        canonical_worktree_root: repo_root.to_owned(),
        authority_root: repo_root.to_owned(),
        target: target(),
        path_scopes: vec!["src".to_owned(), "tests".to_owned()],
        context_digest: "a".repeat(64),
        planner_delegation_ceiling: plane(AuthorityDecision::Allow),
        worker_grant: plane(AuthorityDecision::Allow),
        team_policy: plane(AuthorityDecision::Allow),
        human_ceiling: plane(AuthorityDecision::Ask),
        enforcement: EnforcementEvidence {
            claimed_quality: EnforcementQuality::ObservationOnly,
            winds_mediation_complete: false,
        },
        budgets: BTreeMap::from([
            ("max_operations".to_owned(), 10),
            ("max_wall_seconds".to_owned(), 60),
        ]),
        base_oid: "b".repeat(40),
        candidate_oid: "c".repeat(40),
        candidate_tree: "d".repeat(40),
    }
}

fn seeded_store(name: &str) -> (TestEnvironment, Store, ApprovalContent) {
    let environment = TestEnvironment::new(name);
    let repo_root = environment.canonical_repo_root();
    let git_dir = Path::new(&repo_root)
        .join(".git")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(environment.state_root()).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: &repo_root,
                git_common_dir: &git_dir,
            },
            10,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-1",
                workspace_id: "workspace-1",
                display_name: "Task",
            },
            20,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-1",
                workstream_id: "workstream-1",
                display_name: "Planner",
            },
            30,
        )
        .unwrap();
    let content = base_content(&repo_root);
    (environment, store, content)
}

#[test]
fn identical_normalized_content_has_stable_json_and_digest() {
    let environment = TestEnvironment::new("stable-digest");
    let repo_root = environment.canonical_repo_root();
    let first = base_content(&repo_root);
    let mut second = first.clone();
    second.workstream_id = "  workstream-1  ".to_owned();
    second.session_id = " session-1 ".to_owned();
    second.worker_role = " BUILDER ".to_owned();
    second.path_scopes = vec!["tests".to_owned(), "src".to_owned(), "tests".to_owned()];
    second.budgets = BTreeMap::from([
        (" max_wall_seconds ".to_owned(), 60),
        ("max_operations".to_owned(), 10),
    ]);
    second.context_digest = "A".repeat(64);
    second.base_oid = "B".repeat(40);
    second.candidate_oid = "C".repeat(40);
    second.candidate_tree = "D".repeat(40);

    let first = approval_json_and_digest(&first).unwrap();
    let second = approval_json_and_digest(&second).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.1.len(), 64);
}

#[test]
fn normalized_map_key_collisions_fail_closed() {
    let environment = TestEnvironment::new("normalized-collisions");
    let repo_root = environment.canonical_repo_root();

    let mut budget_collision = base_content(&repo_root);
    budget_collision
        .budgets
        .insert(" max_operations ".to_owned(), 10);
    assert!(approval_json_and_digest(&budget_collision).is_err());

    let mut authority_collision = base_content(&repo_root);
    authority_collision.worker_grant.rules = BTreeMap::from([
        (
            AuthorityTarget {
                capability: "edit".to_owned(),
                resource: "workspace:repo/src".to_owned(),
            },
            AuthorityDecision::Allow,
        ),
        (
            AuthorityTarget {
                capability: " edit ".to_owned(),
                resource: "workspace:repo/src".to_owned(),
            },
            AuthorityDecision::Allow,
        ),
    ]);
    assert!(approval_json_and_digest(&authority_collision).is_err());
}

#[test]
fn unproven_winds_enforcement_canonicalizes_to_unavailable() {
    let environment = TestEnvironment::new("truthful-enforcement");
    let repo_root = environment.canonical_repo_root();
    let mut overclaimed = base_content(&repo_root);
    overclaimed.enforcement = EnforcementEvidence {
        claimed_quality: EnforcementQuality::WindsEnforced,
        winds_mediation_complete: false,
    };
    let mut unavailable = overclaimed.clone();
    unavailable.enforcement.claimed_quality = EnforcementQuality::Unavailable;

    let overclaimed = approval_json_and_digest(&overclaimed).unwrap();
    let unavailable = approval_json_and_digest(&unavailable).unwrap();

    assert_eq!(overclaimed, unavailable);
    assert!(overclaimed.0.contains("UNAVAILABLE"));
    assert!(!overclaimed.0.contains("WINDS_ENFORCED"));
}

#[test]
fn every_material_approval_identity_change_requires_reapproval() {
    let (_environment, store, content) = seeded_store("material-change");
    let stored = record_human_approval(&store, "approval-1", &content, 40).unwrap();

    for case in 0_u8..18 {
        let mut changed = content.clone();
        match case {
            0 => changed.workstream_id = "workstream-other".to_owned(),
            1 => changed.session_id = "session-other".to_owned(),
            2 => changed.worker_role = "REVIEWER".to_owned(),
            3 => changed.runtime_kind = "CLAUDE".to_owned(),
            4 => changed.workspace_id = "workspace-other".to_owned(),
            5 => changed.authority_root.push_str("/subtree"),
            6 => changed.target.capability = "network".to_owned(),
            7 => changed.target.resource = "host:internet".to_owned(),
            8 => changed.path_scopes.push("docs".to_owned()),
            9 => changed.context_digest = "e".repeat(64),
            10 => {
                changed.planner_delegation_ceiling = plane(AuthorityDecision::Ask);
            }
            11 => changed.worker_grant = plane(AuthorityDecision::Ask),
            12 => changed.team_policy = plane(AuthorityDecision::Ask),
            13 => changed.human_ceiling = plane(AuthorityDecision::Allow),
            14 => {
                changed.budgets.insert("max_operations".to_owned(), 11);
            }
            15 => changed.base_oid = "e".repeat(40),
            16 => changed.candidate_oid = "e".repeat(40),
            17 => changed.candidate_tree = "e".repeat(40),
            _ => unreachable!(),
        }

        let evaluation = revalidate_human_approval(&store, "approval-1", &changed).unwrap();
        assert_eq!(evaluation.decision, AuthorityDecision::Ask, "case={case}");
        assert_eq!(
            evaluation.reason,
            ApprovalReason::MaterialContentChanged,
            "case={case}"
        );
        assert_eq!(evaluation.human_action, HumanAction::ApproveRequest);
        assert_ne!(evaluation.current_digest, evaluation.approved_digest);
    }

    assert_eq!(load_human_approval(&store, "approval-1").unwrap(), stored);
}

#[test]
fn exact_content_revalidates_without_expanding_authority() {
    let (_environment, store, content) = seeded_store("exact-revalidate");
    let stored = record_human_approval(&store, "approval-1", &content, 40).unwrap();
    let evaluation = revalidate_human_approval(&store, "approval-1", &content).unwrap();

    assert_eq!(evaluation.decision, AuthorityDecision::Allow);
    assert_eq!(evaluation.reason, ApprovalReason::ExactContentMatch);
    assert_eq!(evaluation.human_action, HumanAction::None);
    assert_eq!(evaluation.approved_digest, stored.content_digest);
    assert_eq!(evaluation.current_digest, stored.content_digest);
}

#[test]
fn approval_audit_is_durable_append_only_and_outside_repo_content() {
    let (environment, store, content) = seeded_store("durable-audit");
    let stored = record_human_approval(&store, "approval-1", &content, 40).unwrap();
    let database_path = environment.state_root().join("winds.db");
    let repo_root = PathBuf::from(environment.canonical_repo_root());

    assert!(database_path.is_file());
    assert!(!database_path.starts_with(&repo_root));
    assert_eq!(fs::read_dir(&repo_root).unwrap().count(), 0);

    assert!(
        store
            .connection
            .execute(
                "UPDATE agentic_delegation_approvals SET content_digest = ?2 WHERE approval_id = ?1",
                rusqlite::params!["approval-1", "f".repeat(64)],
            )
            .is_err()
    );
    assert!(
        store
            .connection
            .execute(
                "DELETE FROM agentic_delegation_approvals WHERE approval_id = ?1",
                rusqlite::params!["approval-1"],
            )
            .is_err()
    );

    drop(store);
    let reopened = Store::open(environment.state_root()).unwrap();
    let loaded = load_human_approval(&reopened, "approval-1").unwrap();
    assert_eq!(loaded, stored);
}

#[test]
fn stored_audit_digest_and_identity_must_self_validate() {
    let (_environment, store, content) = seeded_store("stored-integrity");
    ensure_approval_schema(&store).unwrap();
    let (canonical_json, _digest) = approval_json_and_digest(&content).unwrap();

    store
        .connection
        .execute(
            "INSERT INTO agentic_delegation_approvals(
                approval_id, workstream_id, session_id, workspace_id,
                content_digest, canonical_content_json, approved_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "approval-bad-digest",
                content.workstream_id,
                content.session_id,
                content.workspace_id,
                "f".repeat(64),
                canonical_json,
                40_i64,
            ],
        )
        .unwrap();
    assert!(load_human_approval(&store, "approval-bad-digest").is_err());

    let mut mismatched_content = content.clone();
    mismatched_content.session_id = "session-json-other".to_owned();
    let (mismatched_json, mismatched_digest) =
        approval_json_and_digest(&mismatched_content).unwrap();
    store
        .connection
        .execute(
            "INSERT INTO agentic_delegation_approvals(
                approval_id, workstream_id, session_id, workspace_id,
                content_digest, canonical_content_json, approved_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "approval-bad-identity",
                content.workstream_id,
                content.session_id,
                content.workspace_id,
                mismatched_digest,
                mismatched_json,
                40_i64,
            ],
        )
        .unwrap();
    assert!(load_human_approval(&store, "approval-bad-identity").is_err());
}

#[test]
fn migration_is_idempotent_and_rejects_identity_aliasing() {
    let (_environment, store, content) = seeded_store("migration-identity");
    ensure_approval_schema(&store).unwrap();
    ensure_approval_schema(&store).unwrap();

    let mut mismatched = content.clone();
    mismatched.workstream_id = "other-workstream".to_owned();
    assert!(record_human_approval(&store, "approval-bad", &mismatched, 40).is_err());

    let count: i64 = store
        .connection
        .query_row(
            "SELECT COUNT(*) FROM agentic_delegation_approvals",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn canonical_approval_schema_has_no_auth_secret_environment_or_signing_fields() {
    let environment = TestEnvironment::new("narrow-schema");
    let content = base_content(&environment.canonical_repo_root());
    let (json, _) = approval_json_and_digest(&content).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let object = value.as_object().unwrap();

    for forbidden in [
        "credential",
        "credentials",
        "api_key",
        "auth_token",
        "access_token",
        "environment",
        "env",
        "signature",
        "private_key",
        "public_key",
    ] {
        assert!(
            !object.contains_key(forbidden),
            "forbidden field={forbidden}"
        );
    }
    assert!(object.contains_key("context_digest"));
    assert!(object.contains_key("budgets"));
    assert!(object.contains_key("candidate_oid"));
}

#[test]
fn malformed_or_ambiguous_content_fails_closed_before_approval() {
    let environment = TestEnvironment::new("validation");
    let repo_root = environment.canonical_repo_root();

    let mut bad_context = base_content(&repo_root);
    bad_context.context_digest = "not-a-digest".to_owned();
    assert!(approval_json_and_digest(&bad_context).is_err());

    let mut bad_candidate = base_content(&repo_root);
    bad_candidate.candidate_oid = "abc".to_owned();
    assert!(approval_json_and_digest(&bad_candidate).is_err());

    let mut bad_topology = base_content(&repo_root);
    bad_topology.worker_parent_planner_id = "another-planner".to_owned();
    assert!(approval_json_and_digest(&bad_topology).is_err());

    let mut no_scope = base_content(&repo_root);
    no_scope.path_scopes.clear();
    assert!(approval_json_and_digest(&no_scope).is_err());

    let mut nul_scope = base_content(&repo_root);
    nul_scope.path_scopes = vec!["src\0outside".to_owned()];
    assert!(approval_json_and_digest(&nul_scope).is_err());
}

#[test]
fn approval_time_cannot_precede_canonical_session_creation() {
    let (_environment, store, content) = seeded_store("approval-time");
    assert!(record_human_approval(&store, "approval-early", &content, 29).is_err());
    assert!(record_human_approval(&store, "approval-valid", &content, 30).is_ok());
}
