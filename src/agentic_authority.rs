use crate::store::{Result as StoreResult, Store};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "t076_agentic_approval_tests.rs"]
mod t076_agentic_approval_tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityDecision {
    Deny,
    Ask,
    Allow,
}

impl AuthorityDecision {
    fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "DENY",
            Self::Ask => "ASK",
            Self::Allow => "ALLOW",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AuthorityTarget {
    pub capability: String,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthorityPlane {
    pub default_decision: AuthorityDecision,
    pub rules: BTreeMap<AuthorityTarget, AuthorityDecision>,
}

impl AuthorityPlane {
    fn decision_for(&self, target: &AuthorityTarget) -> AuthorityDecision {
        self.rules
            .get(target)
            .copied()
            .unwrap_or(self.default_decision)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkerGrant {
    pub worker_id: String,
    pub parent_planner_id: String,
    pub authority: AuthorityPlane,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnforcementQuality {
    WindsEnforced,
    OsSandboxEnforced,
    AgentNativeEnforced,
    BestEffortTripwire,
    ObservationOnly,
    Unavailable,
}

impl EnforcementQuality {
    fn as_str(self) -> &'static str {
        match self {
            Self::WindsEnforced => "WINDS_ENFORCED",
            Self::OsSandboxEnforced => "OS_SANDBOX_ENFORCED",
            Self::AgentNativeEnforced => "AGENT_NATIVE_ENFORCED",
            Self::BestEffortTripwire => "BEST_EFFORT_TRIPWIRE",
            Self::ObservationOnly => "OBSERVATION_ONLY",
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EnforcementEvidence {
    pub claimed_quality: EnforcementQuality,
    pub winds_mediation_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DelegationContract {
    pub planner_id: String,
    pub planner_direct_authority: AuthorityPlane,
    pub planner_delegation_ceiling: AuthorityPlane,
    pub team_policy: AuthorityPlane,
    pub human_ceiling: AuthorityPlane,
    pub workers: Vec<WorkerGrant>,
    pub enforcement: EnforcementEvidence,
    pub untrusted_authority_text: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthorityRequest {
    pub worker_id: String,
    pub target: AuthorityTarget,
    pub resource_visible_to_runtime: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthoritySource {
    WorkerGrant,
    PlannerDelegationCeiling,
    TeamPolicy,
    HumanCeiling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityReason {
    InvalidTopology,
    UnknownWorker,
    ExplicitDeny,
    ApprovalRequired,
    EnforcementUnproven,
    AllCeilingsAllow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HumanAction {
    ReduceToSingleWorker,
    SelectAuthorizedWorker,
    ChangeProtectedPolicy,
    ApproveRequest,
    EstablishEnforcementEvidence,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VisibilityAssessment {
    NotVisible,
    VisibleNotAuthorization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthorityEvaluation {
    pub decision: AuthorityDecision,
    pub reason: AuthorityReason,
    pub human_action: HumanAction,
    pub blocking_sources: Vec<AuthoritySource>,
    pub planner_direct_decision: AuthorityDecision,
    pub planner_delegation_decision: AuthorityDecision,
    pub effective_enforcement: EnforcementQuality,
    pub visibility: VisibilityAssessment,
    pub ignored_untrusted_text_count: usize,
}

pub(crate) fn evaluate_delegation(
    contract: &DelegationContract,
    request: &AuthorityRequest,
) -> AuthorityEvaluation {
    let planner_direct_decision = contract
        .planner_direct_authority
        .decision_for(&request.target);
    let planner_delegation_decision = contract
        .planner_delegation_ceiling
        .decision_for(&request.target);
    let effective_enforcement = truthful_enforcement(contract.enforcement);
    let visibility = if request.resource_visible_to_runtime {
        VisibilityAssessment::VisibleNotAuthorization
    } else {
        VisibilityAssessment::NotVisible
    };
    let ignored_untrusted_text_count = contract.untrusted_authority_text.len();

    let make_evaluation = |decision, reason, human_action, blocking_sources| AuthorityEvaluation {
        decision,
        reason,
        human_action,
        blocking_sources,
        planner_direct_decision,
        planner_delegation_decision,
        effective_enforcement,
        visibility,
        ignored_untrusted_text_count,
    };

    if contract.workers.len() != 1 {
        return make_evaluation(
            AuthorityDecision::Deny,
            AuthorityReason::InvalidTopology,
            HumanAction::ReduceToSingleWorker,
            Vec::new(),
        );
    }

    let worker = &contract.workers[0];
    if worker.parent_planner_id != contract.planner_id || worker.worker_id == contract.planner_id {
        return make_evaluation(
            AuthorityDecision::Deny,
            AuthorityReason::InvalidTopology,
            HumanAction::ReduceToSingleWorker,
            Vec::new(),
        );
    }

    if worker.worker_id != request.worker_id {
        return make_evaluation(
            AuthorityDecision::Deny,
            AuthorityReason::UnknownWorker,
            HumanAction::SelectAuthorizedWorker,
            Vec::new(),
        );
    }

    let source_decisions = [
        (
            AuthoritySource::WorkerGrant,
            worker.authority.decision_for(&request.target),
        ),
        (
            AuthoritySource::PlannerDelegationCeiling,
            planner_delegation_decision,
        ),
        (
            AuthoritySource::TeamPolicy,
            contract.team_policy.decision_for(&request.target),
        ),
        (
            AuthoritySource::HumanCeiling,
            contract.human_ceiling.decision_for(&request.target),
        ),
    ];

    let denied_by = source_decisions
        .iter()
        .filter_map(|(source, decision)| (*decision == AuthorityDecision::Deny).then_some(*source))
        .collect::<Vec<_>>();
    if !denied_by.is_empty() {
        return make_evaluation(
            AuthorityDecision::Deny,
            AuthorityReason::ExplicitDeny,
            HumanAction::ChangeProtectedPolicy,
            denied_by,
        );
    }

    let asked_by = source_decisions
        .iter()
        .filter_map(|(source, decision)| (*decision == AuthorityDecision::Ask).then_some(*source))
        .collect::<Vec<_>>();
    if !asked_by.is_empty() {
        return make_evaluation(
            AuthorityDecision::Ask,
            AuthorityReason::ApprovalRequired,
            HumanAction::ApproveRequest,
            asked_by,
        );
    }

    if effective_enforcement == EnforcementQuality::Unavailable {
        return make_evaluation(
            AuthorityDecision::Ask,
            AuthorityReason::EnforcementUnproven,
            HumanAction::EstablishEnforcementEvidence,
            Vec::new(),
        );
    }

    make_evaluation(
        AuthorityDecision::Allow,
        AuthorityReason::AllCeilingsAllow,
        HumanAction::None,
        Vec::new(),
    )
}

fn truthful_enforcement(evidence: EnforcementEvidence) -> EnforcementQuality {
    if evidence.claimed_quality == EnforcementQuality::WindsEnforced
        && !evidence.winds_mediation_complete
    {
        EnforcementQuality::Unavailable
    } else {
        evidence.claimed_quality
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalContent {
    pub workstream_id: String,
    pub session_id: String,
    pub planner_id: String,
    pub worker_id: String,
    pub worker_parent_planner_id: String,
    pub worker_role: String,
    pub runtime_kind: String,
    pub workspace_id: String,
    pub canonical_worktree_root: String,
    pub authority_root: String,
    pub target: AuthorityTarget,
    pub path_scopes: Vec<String>,
    pub context_digest: String,
    pub planner_delegation_ceiling: AuthorityPlane,
    pub worker_grant: AuthorityPlane,
    pub team_policy: AuthorityPlane,
    pub human_ceiling: AuthorityPlane,
    pub enforcement: EnforcementEvidence,
    pub budgets: BTreeMap<String, u64>,
    pub base_oid: String,
    pub candidate_oid: String,
    pub candidate_tree: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StoredApproval {
    pub approval_id: String,
    pub workstream_id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub content_digest: String,
    pub canonical_content_json: String,
    pub approved_unix_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApprovalReason {
    ExactContentMatch,
    MaterialContentChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ApprovalEvaluation {
    pub decision: AuthorityDecision,
    pub reason: ApprovalReason,
    pub human_action: HumanAction,
    pub approved_digest: String,
    pub current_digest: String,
}

#[derive(Debug, Clone, Serialize)]
struct CanonicalAuthorityRule {
    capability: String,
    resource: String,
    decision: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct CanonicalAuthorityPlane {
    default_decision: &'static str,
    rules: Vec<CanonicalAuthorityRule>,
}

#[derive(Debug, Clone, Serialize)]
struct CanonicalBudget {
    name: String,
    limit: u64,
}

#[derive(Debug, Clone, Serialize)]
struct CanonicalApprovalContent {
    schema_version: u32,
    workstream_id: String,
    session_id: String,
    planner_id: String,
    worker_id: String,
    worker_parent_planner_id: String,
    worker_role: String,
    runtime_kind: String,
    workspace_id: String,
    canonical_worktree_root: String,
    authority_root: String,
    target_capability: String,
    target_resource: String,
    path_scopes: Vec<String>,
    context_digest: String,
    planner_delegation_ceiling: CanonicalAuthorityPlane,
    worker_grant: CanonicalAuthorityPlane,
    team_policy: CanonicalAuthorityPlane,
    human_ceiling: CanonicalAuthorityPlane,
    enforcement_quality: &'static str,
    winds_mediation_complete: bool,
    budgets: Vec<CanonicalBudget>,
    base_oid: String,
    candidate_oid: String,
    candidate_tree: String,
}

pub(crate) fn approval_json_and_digest(content: &ApprovalContent) -> StoreResult<(String, String)> {
    let canonical = canonicalize_approval(content)?;
    let json = serde_json::to_string(&canonical)?;
    let digest = sha256_hex(json.as_bytes());
    Ok((json, digest))
}

pub(crate) fn record_human_approval(
    store: &Store,
    approval_id: &str,
    content: &ApprovalContent,
    approved_unix_ms: i64,
) -> StoreResult<StoredApproval> {
    let approval_id = normalize_label(approval_id, "approval id")?;
    if approved_unix_ms < 0 {
        return Err("approval time must not be negative".into());
    }
    let canonical = canonicalize_approval(content)?;
    let canonical_json = serde_json::to_string(&canonical)?;
    let content_digest = sha256_hex(canonical_json.as_bytes());

    let identity = store
        .connection
        .query_row(
            "SELECT workspace.canonical_worktree_root, session.created_unix_ms
             FROM winds_sessions session
             INNER JOIN workstreams workstream
                ON workstream.workstream_id = session.workstream_id
             INNER JOIN workspaces workspace
                ON workspace.workspace_id = workstream.workspace_id
             WHERE session.session_id = ?1
               AND session.workstream_id = ?2
               AND workstream.workspace_id = ?3",
            params![
                canonical.session_id,
                canonical.workstream_id,
                canonical.workspace_id
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()?
        .ok_or(
            "approval identity does not match canonical Winds session/workstream/workspace truth",
        )?;
    if identity.0 != canonical.canonical_worktree_root {
        return Err("approval worktree root does not match canonical Winds workspace truth".into());
    }
    if approved_unix_ms < identity.1 {
        return Err("approval time cannot precede Winds session creation".into());
    }

    ensure_approval_schema(store)?;
    store.connection.execute(
        "INSERT INTO agentic_delegation_approvals(
            approval_id, workstream_id, session_id, workspace_id,
            content_digest, canonical_content_json, approved_unix_ms
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            approval_id,
            canonical.workstream_id,
            canonical.session_id,
            canonical.workspace_id,
            content_digest,
            canonical_json,
            approved_unix_ms,
        ],
    )?;

    Ok(StoredApproval {
        approval_id,
        workstream_id: canonical.workstream_id,
        session_id: canonical.session_id,
        workspace_id: canonical.workspace_id,
        content_digest,
        canonical_content_json: canonical_json,
        approved_unix_ms,
    })
}

pub(crate) fn load_human_approval(store: &Store, approval_id: &str) -> StoreResult<StoredApproval> {
    let approval_id = normalize_label(approval_id, "approval id")?;
    ensure_approval_schema(store)?;
    let stored = store
        .connection
        .query_row(
            "SELECT approval_id, workstream_id, session_id, workspace_id,
                    content_digest, canonical_content_json, approved_unix_ms
             FROM agentic_delegation_approvals
             WHERE approval_id = ?1",
            params![approval_id],
            |row| {
                Ok(StoredApproval {
                    approval_id: row.get(0)?,
                    workstream_id: row.get(1)?,
                    session_id: row.get(2)?,
                    workspace_id: row.get(3)?,
                    content_digest: row.get(4)?,
                    canonical_content_json: row.get(5)?,
                    approved_unix_ms: row.get(6)?,
                })
            },
        )
        .optional()?
        .ok_or_else(|| format!("unknown human approval: {approval_id}"))?;
    validate_stored_approval(&stored)?;
    Ok(stored)
}

pub(crate) fn revalidate_human_approval(
    store: &Store,
    approval_id: &str,
    current_content: &ApprovalContent,
) -> StoreResult<ApprovalEvaluation> {
    let stored = load_human_approval(store, approval_id)?;
    let (_, current_digest) = approval_json_and_digest(current_content)?;
    if current_digest == stored.content_digest {
        Ok(ApprovalEvaluation {
            decision: AuthorityDecision::Allow,
            reason: ApprovalReason::ExactContentMatch,
            human_action: HumanAction::None,
            approved_digest: stored.content_digest,
            current_digest,
        })
    } else {
        Ok(ApprovalEvaluation {
            decision: AuthorityDecision::Ask,
            reason: ApprovalReason::MaterialContentChanged,
            human_action: HumanAction::ApproveRequest,
            approved_digest: stored.content_digest,
            current_digest,
        })
    }
}

fn ensure_approval_schema(store: &Store) -> StoreResult<()> {
    let complete_objects = store.connection.query_row(
        "SELECT COUNT(*)
         FROM sqlite_master
         WHERE name IN (
             'agentic_delegation_approvals',
             'idx_agentic_delegation_approvals_session_time',
             'trg_agentic_delegation_approval_identity_insert',
             'trg_agentic_delegation_approval_no_update',
             'trg_agentic_delegation_approval_no_delete'
         )",
        [],
        |row| row.get::<_, i64>(0),
    )?;
    if complete_objects != 5 {
        store.connection.execute_batch(include_str!(
            "../migrations/0009_agentic_delegation_audit.sql"
        ))?;
    }
    Ok(())
}

fn validate_stored_approval(stored: &StoredApproval) -> StoreResult<()> {
    let observed_digest = sha256_hex(stored.canonical_content_json.as_bytes());
    if observed_digest != stored.content_digest {
        return Err("stored human approval digest does not match canonical content".into());
    }

    let value: serde_json::Value = serde_json::from_str(&stored.canonical_content_json)?;
    let object = value
        .as_object()
        .ok_or("stored human approval canonical content must be a JSON object")?;
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Err("stored human approval has unsupported canonical schema version".into());
    }
    for (field, expected) in [
        ("workstream_id", stored.workstream_id.as_str()),
        ("session_id", stored.session_id.as_str()),
        ("workspace_id", stored.workspace_id.as_str()),
    ] {
        if object.get(field).and_then(serde_json::Value::as_str) != Some(expected) {
            return Err(
                format!("stored human approval {field} does not match its audit identity").into(),
            );
        }
    }
    Ok(())
}

fn canonicalize_approval(content: &ApprovalContent) -> StoreResult<CanonicalApprovalContent> {
    let workstream_id = validate_exact_text(&content.workstream_id, "workstream id")?;
    let session_id = validate_exact_text(&content.session_id, "session id")?;
    let planner_id = validate_exact_text(&content.planner_id, "planner id")?;
    let worker_id = validate_exact_text(&content.worker_id, "worker id")?;
    let worker_parent_planner_id =
        validate_exact_text(&content.worker_parent_planner_id, "worker parent planner id")?;
    if worker_parent_planner_id != planner_id || worker_id == planner_id {
        return Err("approval requires one Worker directly delegated by a distinct Planner".into());
    }
    let worker_role = normalize_label(&content.worker_role, "worker role")?;
    let runtime_kind = normalize_label(&content.runtime_kind, "runtime kind")?;
    if !matches!(runtime_kind.as_str(), "CODEX" | "CLAUDE") {
        return Err("approval runtime kind must be CODEX or CLAUDE".into());
    }
    let workspace_id = validate_exact_text(&content.workspace_id, "workspace id")?;
    let canonical_worktree_root =
        validate_exact_text(&content.canonical_worktree_root, "canonical worktree root")?;
    let authority_root = validate_exact_text(&content.authority_root, "authority root")?;
    let target_capability =
        validate_exact_text(&content.target.capability, "target capability")?;
    let target_resource = validate_exact_text(&content.target.resource, "target resource")?;

    let mut path_scopes = content
        .path_scopes
        .iter()
        .map(|path| validate_exact_text(path, "path scope"))
        .collect::<StoreResult<Vec<_>>>()?;
    path_scopes.sort();
    path_scopes.dedup();
    if path_scopes.is_empty() {
        return Err("approval requires at least one explicit path scope".into());
    }

    let context_digest = normalize_hex(&content.context_digest, 64, "context digest")?;
    let base_oid = normalize_hex(&content.base_oid, 40, "base oid")?;
    let candidate_oid = normalize_hex(&content.candidate_oid, 40, "candidate oid")?;
    let candidate_tree = normalize_hex(&content.candidate_tree, 40, "candidate tree")?;

    let mut budgets = content
        .budgets
        .iter()
        .map(|(name, limit)| {
            Ok(CanonicalBudget {
                name: normalize_label(name, "budget name")?,
                limit: *limit,
            })
        })
        .collect::<StoreResult<Vec<_>>>()?;
    if budgets.is_empty() {
        return Err("approval requires at least one explicit budget".into());
    }
    budgets.sort_by(|left, right| left.name.cmp(&right.name));
    if budgets.windows(2).any(|pair| pair[0].name == pair[1].name) {
        return Err("approval budget names collide after normalization".into());
    }

    Ok(CanonicalApprovalContent {
        schema_version: 1,
        workstream_id,
        session_id,
        planner_id,
        worker_id,
        worker_parent_planner_id,
        worker_role,
        runtime_kind,
        workspace_id,
        canonical_worktree_root,
        authority_root,
        target_capability,
        target_resource,
        path_scopes,
        context_digest,
        planner_delegation_ceiling: canonicalize_plane(&content.planner_delegation_ceiling)?,
        worker_grant: canonicalize_plane(&content.worker_grant)?,
        team_policy: canonicalize_plane(&content.team_policy)?,
        human_ceiling: canonicalize_plane(&content.human_ceiling)?,
        enforcement_quality: truthful_enforcement(content.enforcement).as_str(),
        winds_mediation_complete: content.enforcement.winds_mediation_complete,
        budgets,
        base_oid,
        candidate_oid,
        candidate_tree,
    })
}

fn canonicalize_plane(plane: &AuthorityPlane) -> StoreResult<CanonicalAuthorityPlane> {
    let mut rules = plane
        .rules
        .iter()
        .map(|(target, decision)| {
            Ok(CanonicalAuthorityRule {
                capability: validate_exact_text(&target.capability, "authority capability")?,
                resource: validate_exact_text(&target.resource, "authority resource")?,
                decision: decision.as_str(),
            })
        })
        .collect::<StoreResult<Vec<_>>>()?;
    rules.sort_by(|left, right| {
        (&left.capability, &left.resource).cmp(&(&right.capability, &right.resource))
    });
    Ok(CanonicalAuthorityPlane {
        default_decision: plane.default_decision.as_str(),
        rules,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn normalize_label(value: &str, label: &str) -> StoreResult<String> {
    if value.contains('\0') {
        return Err(format!("{label} must not contain NUL").into());
    }
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} must not be empty").into());
    }
    Ok(value.to_owned())
}

fn validate_exact_text(value: &str, label: &str) -> StoreResult<String> {
    if value.trim().is_empty() {
        return Err(format!("{label} must not be blank").into());
    }
    if value.contains('\0') {
        return Err(format!("{label} must not contain NUL").into());
    }
    Ok(value.to_owned())
}

fn normalize_hex(value: &str, expected_len: usize, label: &str) -> StoreResult<String> {
    let value = value.trim();
    if value.len() != expected_len || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(
            format!("{label} must be exactly {expected_len} hexadecimal characters").into(),
        );
    }
    Ok(value.to_ascii_lowercase())
}
