use crate::agentic_authority::{
    ApprovalContent, AuthorityDecision, AuthorityPlane, AuthorityRequest, AuthorityTarget,
    DelegationContract, EnforcementEvidence, EnforcementQuality, WorkerGrant, evaluate_delegation,
    record_human_approval, revalidate_human_approval,
};
use crate::git::{Repo, observe_worktree_state};
use crate::store::{NewWindsSession, NewWorkspace, NewWorkstream, Store};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    primary: PathBuf,
    state: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "winds-t082-worker-worktree-{}-{sequence}-{name}",
            std::process::id()
        ));
        let primary = root.join("primary");
        let state = root.join("state");
        fs::create_dir_all(&primary).unwrap();
        fs::create_dir_all(&state).unwrap();
        run_git(&primary, ["init"]).unwrap();
        run_git(&primary, ["config", "user.email", "winds@example.invalid"]).unwrap();
        run_git(&primary, ["config", "user.name", "Winds T082 Fixture"]).unwrap();
        fs::write(primary.join("README.md"), "base\n").unwrap();
        run_git(&primary, ["add", "README.md"]).unwrap();
        run_git(&primary, ["commit", "-m", "base"]).unwrap();
        Self {
            root,
            primary,
            state,
        }
    }

    fn worker(&self, name: &str) -> PathBuf {
        self.root.join("workers").join(name)
    }

    fn base_oid(&self) -> String {
        run_git(&self.primary, ["rev-parse", "HEAD"]).unwrap()
    }

    fn head_refs(&self) -> String {
        run_git(
            &self.primary,
            [
                "for-each-ref",
                "--format=%(refname):%(objectname)",
                "refs/heads",
            ],
        )
        .unwrap()
    }

    fn status(&self) -> String {
        run_git(
            &self.primary,
            ["status", "--porcelain=v1", "--untracked-files=all"],
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_git<I, S>(cwd: &Path, args: I) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .current_dir(cwd)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| error.to_string())
}

fn target(worker_root: &str, suffix: &str) -> AuthorityTarget {
    AuthorityTarget {
        capability: "edit".to_owned(),
        resource: format!("path:{worker_root}/{suffix}"),
    }
}

fn plane(target: &AuthorityTarget, decision: AuthorityDecision) -> AuthorityPlane {
    AuthorityPlane {
        default_decision: AuthorityDecision::Deny,
        rules: BTreeMap::from([(target.clone(), decision)]),
    }
}

fn approval_content(
    worker_root: &str,
    exact_base: &str,
    exact_tree: &str,
    approved_target: &AuthorityTarget,
) -> ApprovalContent {
    let allow = plane(approved_target, AuthorityDecision::Allow);
    ApprovalContent {
        workstream_id: "workstream-t082".to_owned(),
        session_id: "worker-session-t082".to_owned(),
        planner_id: "planner-t082".to_owned(),
        worker_id: "worker-t082".to_owned(),
        worker_parent_planner_id: "planner-t082".to_owned(),
        worker_role: "BUILDER".to_owned(),
        runtime_kind: "CODEX".to_owned(),
        workspace_id: "workspace-t082".to_owned(),
        canonical_worktree_root: worker_root.to_owned(),
        authority_root: worker_root.to_owned(),
        target: approved_target.clone(),
        path_scopes: vec!["src".to_owned()],
        context_digest: "a".repeat(64),
        planner_delegation_ceiling: allow.clone(),
        worker_grant: allow.clone(),
        team_policy: allow.clone(),
        human_ceiling: allow,
        enforcement: EnforcementEvidence {
            claimed_quality: EnforcementQuality::ObservationOnly,
            winds_mediation_complete: false,
        },
        budgets: BTreeMap::from([
            ("max_operations".to_owned(), 1),
            ("max_wall_seconds".to_owned(), 60),
        ]),
        base_oid: exact_base.to_owned(),
        candidate_oid: exact_base.to_owned(),
        candidate_tree: exact_tree.to_owned(),
    }
}

fn delegation(content: &ApprovalContent) -> DelegationContract {
    DelegationContract {
        planner_id: content.planner_id.clone(),
        planner_direct_authority: AuthorityPlane {
            default_decision: AuthorityDecision::Deny,
            rules: BTreeMap::new(),
        },
        planner_delegation_ceiling: content.planner_delegation_ceiling.clone(),
        team_policy: content.team_policy.clone(),
        human_ceiling: content.human_ceiling.clone(),
        workers: vec![WorkerGrant {
            worker_id: content.worker_id.clone(),
            parent_planner_id: content.worker_parent_planner_id.clone(),
            authority: content.worker_grant.clone(),
        }],
        enforcement: content.enforcement,
        untrusted_authority_text: Vec::new(),
    }
}

fn seed_approval_store(fixture: &Fixture, worker_root: &str, git_common_dir: &str) -> Store {
    let store = Store::open(&fixture.state).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-t082",
                canonical_worktree_root: worker_root,
                git_common_dir,
            },
            10,
        )
        .unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-t082",
                workspace_id: "workspace-t082",
                display_name: "T082 Worker Edit",
            },
            20,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "worker-session-t082",
                workstream_id: "workstream-t082",
                display_name: "Codex Worker",
            },
            30,
        )
        .unwrap();
    store
}

#[test]
fn exact_base_worker_worktree_is_detached_identity_bound_and_primary_safe() {
    let fixture = Fixture::new("exact-identity");
    let repo = Repo::open(&fixture.primary).unwrap();
    let exact_base = fixture.base_oid();
    let primary_status = fixture.status();
    let primary_refs = fixture.head_refs();
    let worker = fixture.worker("worker-1");

    repo.add_locked_worktree(
        &worker,
        &exact_base,
        "Winds T082 deterministic Worker fixture",
    )
    .unwrap();

    let canonical_worker = fs::canonicalize(&worker).unwrap();
    let worker_repo = Repo::open(&canonical_worker).unwrap();
    assert_eq!(worker_repo.root(), canonical_worker);
    assert_eq!(worker_repo.common_dir(), repo.common_dir());
    assert_eq!(repo.worktree_head(&canonical_worker).unwrap(), exact_base);
    assert!(repo.worktree_is_clean(&canonical_worker).unwrap());
    assert!(repo.worktree_paths().unwrap().iter().any(|path| {
        fs::canonicalize(path).ok().as_deref() == Some(canonical_worker.as_path())
    }));
    assert_eq!(fixture.status(), primary_status);
    assert_eq!(fixture.head_refs(), primary_refs);
}

#[test]
fn dirty_worker_and_agent_done_claim_remain_non_authoritative_git_observations() {
    let fixture = Fixture::new("dirty-preserved");
    let repo = Repo::open(&fixture.primary).unwrap();
    let exact_base = fixture.base_oid();
    let worker = fixture.worker("worker-1");
    repo.add_locked_worktree(&worker, &exact_base, "Winds T082 dirty-state fixture")
        .unwrap();
    let worker = fs::canonicalize(&worker).unwrap();

    fs::write(worker.join("README.md"), "worker edit not yet verified\n").unwrap();
    let agent_claim = "done; tests passed";
    assert!(!agent_claim.is_empty());

    let observation = observe_worktree_state(&worker, repo.common_dir()).unwrap();
    assert_eq!(observation.head_oid.as_deref(), Some(exact_base.as_str()));
    assert!(observation.dirty);
    assert_eq!(
        fs::read_to_string(worker.join("README.md")).unwrap(),
        "worker edit not yet verified\n"
    );
    assert_eq!(fixture.status(), "");
}

#[test]
fn moved_or_ambiguous_worker_state_is_retained_and_never_force_cleaned() {
    let fixture = Fixture::new("moved-preserved");
    let repo = Repo::open(&fixture.primary).unwrap();
    let exact_base = fixture.base_oid();
    let worker = fixture.worker("worker-1");
    repo.add_locked_worktree(&worker, &exact_base, "Winds T082 moved-state fixture")
        .unwrap();
    let worker = fs::canonicalize(&worker).unwrap();

    fs::write(worker.join("README.md"), "worker commit\n").unwrap();
    run_git(&worker, ["add", "README.md"]).unwrap();
    run_git(&worker, ["commit", "-m", "worker edit"]).unwrap();
    let moved_head = run_git(&worker, ["rev-parse", "HEAD"]).unwrap();
    assert_ne!(moved_head, exact_base);

    let observation = observe_worktree_state(&worker, repo.common_dir()).unwrap();
    assert_eq!(observation.head_oid.as_deref(), Some(moved_head.as_str()));
    assert!(!observation.dirty);

    let wrong_common = fixture.root.join("not-the-git-common-dir");
    fs::create_dir_all(&wrong_common).unwrap();
    assert!(observe_worktree_state(&worker, &wrong_common).is_err());
    assert!(worker.exists());
    assert_eq!(run_git(&worker, ["rev-parse", "HEAD"]).unwrap(), moved_head);
}

#[test]
fn human_approval_is_content_bound_and_operation_scope_cannot_self_expand() {
    let fixture = Fixture::new("human-approval");
    let repo = Repo::open(&fixture.primary).unwrap();
    let exact_base = fixture.base_oid();
    let exact_tree = repo.tree_oid(&exact_base).unwrap();
    let worker = fixture.worker("worker-1");
    repo.add_locked_worktree(&worker, &exact_base, "Winds T082 approval fixture")
        .unwrap();
    let worker = fs::canonicalize(&worker).unwrap();
    let worker_root = worker.to_string_lossy().into_owned();
    let git_common_dir = repo.common_dir().to_string_lossy().into_owned();
    let approved_target = target(&worker_root, "src");
    let content = approval_content(&worker_root, &exact_base, &exact_tree, &approved_target);
    let store = seed_approval_store(&fixture, &worker_root, &git_common_dir);

    assert!(revalidate_human_approval(&store, "approval-t082", &content).is_err());
    record_human_approval(&store, "approval-t082", &content, 40).unwrap();
    let exact = revalidate_human_approval(&store, "approval-t082", &content).unwrap();
    assert_eq!(exact.decision, AuthorityDecision::Allow);

    let mut changed = content.clone();
    changed.path_scopes.push("tests".to_owned());
    assert_eq!(
        revalidate_human_approval(&store, "approval-t082", &changed)
            .unwrap()
            .decision,
        AuthorityDecision::Ask
    );

    let contract = delegation(&content);
    let allowed = evaluate_delegation(
        &contract,
        &AuthorityRequest {
            worker_id: content.worker_id.clone(),
            target: approved_target,
            resource_visible_to_runtime: true,
        },
    );
    assert_eq!(allowed.decision, AuthorityDecision::Allow);

    let outside = evaluate_delegation(
        &contract,
        &AuthorityRequest {
            worker_id: content.worker_id.clone(),
            target: target(&worker_root, "tests"),
            resource_visible_to_runtime: true,
        },
    );
    assert_eq!(outside.decision, AuthorityDecision::Deny);
    assert!(repo.worktree_is_clean(&worker).unwrap());
}
