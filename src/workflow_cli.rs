use crate::domain::workflow::projection::{
    ActorBindingProjectionInput, ActorRoleTruth, AuthorityCeilingTruth, BlockerTruth,
    DecisionProjectionInput, ExternalGateState, ExternalGateTruth, RequiredAction,
    ResumeDisposition, WorkflowProjectionInput, project_resume_preview, project_reviewer_handoff,
    project_why_blocked, project_workflow_status,
};
use crate::domain::workflow::{
    ArtifactBaselineKind, ArtifactBaselineRequirement, BaselineEvaluation, BaselineFreshness,
    CandidateBaselineIdentity, DecisionApplicability, DecisionApplicabilityContext,
    DecisionContentState, RETRY_OUTCOME_BUDGET_EXHAUSTED, RetryFailureObservation, SideEffectTruth,
    StageAttemptRelation, StageLifecycleState, StageRunIdentity, StageTransitionAuthority,
    StageTransitionOutcome, StageTransitionRequest, TruthSource, WorkflowContinuationClass,
    WorkflowDecisionInput, WorkflowDecisionRecord, WorkflowRunIdentity,
    evaluate_artifact_baseline_requirement, evaluate_decision_applicability,
};
use crate::store::{Store, StoredStageRun, StoredWorkflowActorBinding, StoredWorkflowRun};
use crate::{Result, ensure_allowed_flags, required, unix_ms};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "t109_workflow_cli_tests.rs"]
mod t109_workflow_cli_tests;

const MAX_CLI_ITEMS: usize = 256;

pub(crate) fn dispatch(flags: HashMap<String, String>) -> Result<()> {
    let output = execute(flags)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn execute(flags: HashMap<String, String>) -> Result<Value> {
    let action = required(&flags, "action")?.to_owned();
    match action.as_str() {
        "create" => create_workflow(flags),
        "open" => open_workflow(flags),
        "prepare-stage" => prepare_stage(flags),
        "start-stage" => start_stage(flags),
        "transition-stage" => transition_stage(flags),
        "record-failure" => record_failure(flags),
        "retry" => retry_stage(flags),
        "recover" => recover_stage(flags),
        "decision-record" => record_decision(flags),
        "decision-list" => list_decisions(flags),
        "status" => inspect_projection(flags, ProjectionKind::Status),
        "resume-preview" => inspect_projection(flags, ProjectionKind::ResumePreview),
        "why-blocked" => inspect_projection(flags, ProjectionKind::WhyBlocked),
        "reviewer-handoff" => inspect_projection(flags, ProjectionKind::ReviewerHandoff),
        _ => Err(format!("unknown workflow action: {action}").into()),
    }
}

fn create_workflow(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &[
            "action",
            "home",
            "workspace-id",
            "workstream-id",
            "workflow-id",
        ],
    )?;
    let store = open_store(&flags)?;
    let identity = workflow_identity_from_flags(&flags)?;
    store.create_workflow_run(&identity, unix_ms()?)?;
    let stored = store.load_workflow_run(&identity.workflow_run_id)?;
    Ok(json!({"action":"create","workflow":workflow_json(&stored)}))
}

fn open_workflow(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &[
            "action",
            "home",
            "workspace-id",
            "workstream-id",
            "workflow-id",
        ],
    )?;
    let store = open_store(&flags)?;
    let workflow = require_workflow(&store, &flags)?;
    Ok(json!({"action":"open","workflow":workflow_json(&workflow)}))
}

fn prepare_stage(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &[
            "action",
            "home",
            "workspace-id",
            "workstream-id",
            "workflow-id",
            "stage-id",
            "stage-key",
        ],
    )?;
    let store = open_store(&flags)?;
    let workflow = require_workflow(&store, &flags)?;
    let stage = StageRunIdentity::new(
        required(&flags, "stage-id")?,
        &workflow.identity.workflow_run_id,
        required(&flags, "stage-key")?,
        1,
        None,
    )?;
    store.create_stage_run(&stage, None, unix_ms()?)?;
    Ok(
        json!({"action":"prepare-stage","stage":stage_json(&store.load_stage_run(&stage.stage_run_id)?)}),
    )
}

fn start_stage(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &stage_operation_flags(&["operation-id"]))?;
    let store = open_store(&flags)?;
    let stage = require_stage(&store, &flags)?;
    let request = StageTransitionRequest::new(
        required(&flags, "operation-id")?,
        StageLifecycleState::Prepared,
        StageLifecycleState::Active,
        TruthSource::WindsObserved,
        StageTransitionAuthority::None,
    )?;
    let outcome = store.transition_stage_run(&stage.identity.stage_run_id, &request, unix_ms()?)?;
    Ok(json!({
        "action":"start-stage",
        "outcome":transition_outcome_json(&outcome),
        "stage":stage_json(&store.load_stage_run(&stage.identity.stage_run_id)?),
    }))
}
fn transition_stage(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &stage_operation_flags(&["operation-id", "to"]))?;
    let store = open_store(&flags)?;
    let stage = require_stage(&store, &flags)?;
    let requested = StageLifecycleState::from_db(required(&flags, "to")?).ok_or_else(|| {
        format!(
            "unknown workflow lifecycle target: {}",
            required(&flags, "to").unwrap_or("")
        )
    })?;
    if !matches!(
        requested,
        StageLifecycleState::WaitingApproval
            | StageLifecycleState::WaitingExternal
            | StageLifecycleState::Blocked
    ) {
        return Err("workflow CLI transition-stage permits only WAITING_APPROVAL, WAITING_EXTERNAL, or BLOCKED; terminal resolution, staleness, and failure/recovery require dedicated qualified paths".into());
    }
    let request = StageTransitionRequest::new(
        required(&flags, "operation-id")?,
        stage.lifecycle_state,
        requested,
        TruthSource::WindsObserved,
        StageTransitionAuthority::None,
    )?;
    let outcome = store.transition_stage_run(&stage.identity.stage_run_id, &request, unix_ms()?)?;
    Ok(json!({
        "action":"transition-stage",
        "outcome":transition_outcome_json(&outcome),
        "stage":stage_json(&store.load_stage_run(&stage.identity.stage_run_id)?),
    }))
}

fn record_failure(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &stage_operation_flags(&[
            "operation-id",
            "failure-class",
            "checkpoint",
            "progress-basis",
            "side-effect",
        ]),
    )?;
    let store = open_store(&flags)?;
    let stage = require_stage(&store, &flags)?;
    let side_effect = match required(&flags, "side-effect")? {
        "SAFE_OR_IDEMPOTENT" => SideEffectTruth::SafeOrIdempotent,
        "AMBIGUOUS_NON_IDEMPOTENT" => SideEffectTruth::AmbiguousNonIdempotent,
        other => return Err(format!("unknown side-effect truth: {other}").into()),
    };
    let observation = RetryFailureObservation::new(
        required(&flags, "failure-class")?,
        flags.get("checkpoint").map(String::as_str),
        required(&flags, "progress-basis")?,
        side_effect,
    )?;
    let resolution = store.record_stage_failure(
        &stage.identity.stage_run_id,
        required(&flags, "operation-id")?,
        &observation,
        unix_ms()?,
    )?;
    Ok(json!({
        "action":"record-failure",
        "failure":{
            "failure_class":observation.failure_class,
            "checkpoint_identity":observation.checkpoint_identity,
            "material_progress_basis":observation.material_progress_basis,
            "side_effect":side_effect_str(side_effect),
        },
        "resolution":{
            "terminal_state":resolution.terminal_state.as_db_str(),
            "comparison":resolution.comparison.map(retry_comparison_str),
            "consecutive_no_progress_retries":resolution.consecutive_no_progress_retries,
            "outcome_reason":resolution.outcome_reason,
        },
        "stage":stage_json(&store.load_stage_run(&stage.identity.stage_run_id)?),
    }))
}

fn retry_stage(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &stage_operation_flags(&["new-stage-id"]))?;
    let store = open_store(&flags)?;
    let predecessor = require_stage(&store, &flags)?;
    let ordinal = predecessor
        .identity
        .attempt_ordinal
        .checked_add(1)
        .ok_or("workflow stage attempt ordinal exhausted")?;
    let successor = StageRunIdentity::new(
        required(&flags, "new-stage-id")?,
        &predecessor.identity.workflow_run_id,
        &predecessor.identity.stage_key,
        ordinal,
        Some(&predecessor.identity.stage_run_id),
    )?;
    store.create_explicit_retry_stage_run(&successor, unix_ms()?)?;
    Ok(json!({
        "action":"retry",
        "relation":"RETRY_OF",
        "predecessor_stage_run_id":predecessor.identity.stage_run_id,
        "stage":stage_json(&store.load_stage_run(&successor.stage_run_id)?),
    }))
}

fn recover_stage(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &stage_operation_flags(&["new-stage-id"]))?;
    let store = open_store(&flags)?;
    let predecessor = require_stage(&store, &flags)?;
    let recoverable = predecessor.lifecycle_state == StageLifecycleState::RecoveryRequired
        || (predecessor.lifecycle_state == StageLifecycleState::Failed
            && predecessor.outcome_reason.as_deref() == Some(RETRY_OUTCOME_BUDGET_EXHAUSTED));
    if !recoverable {
        return Err("explicit recovery requires RECOVERY_REQUIRED or retry-budget-exhausted predecessor truth".into());
    }
    let ordinal = predecessor
        .identity
        .attempt_ordinal
        .checked_add(1)
        .ok_or("workflow stage attempt ordinal exhausted")?;
    let successor = StageRunIdentity::new(
        required(&flags, "new-stage-id")?,
        &predecessor.identity.workflow_run_id,
        &predecessor.identity.stage_key,
        ordinal,
        Some(&predecessor.identity.stage_run_id),
    )?;
    store.create_stage_run(&successor, Some(StageAttemptRelation::Recovery), unix_ms()?)?;
    Ok(json!({
        "action":"recover",
        "relation":"RECOVERY_OF",
        "predecessor_stage_run_id":predecessor.identity.stage_run_id,
        "stage":stage_json(&store.load_stage_run(&successor.stage_run_id)?),
    }))
}

fn record_decision(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(
        &flags,
        &decision_flags(&[
            "decision-id",
            "decision-type",
            "decision-result",
            "predecessor-decision-id",
            "source",
            "authority",
            "content-state",
            "safe-rationale",
        ]),
    )?;
    let store = open_store(&flags)?;
    let workflow = require_workflow(&store, &flags)?;
    let stage_run_id = optional_exact_stage_id(&store, &flags, &workflow)?;
    let source = parse_decision_source(required(&flags, "source")?)?;
    let authority = parse_decision_authority(required(&flags, "authority")?)?;
    let candidate = optional_candidate(&flags)?;
    let content_state = DecisionContentState::from_db(required(&flags, "content-state")?)
        .ok_or_else(|| {
            format!(
                "unknown decision content state: {}",
                required(&flags, "content-state").unwrap_or("")
            )
        })?;
    let decision = WorkflowDecisionRecord::new(WorkflowDecisionInput {
        decision_id: required(&flags, "decision-id")?,
        workflow_run_id: &workflow.identity.workflow_run_id,
        stage_run_id: stage_run_id.as_deref(),
        source,
        authority,
        decision_type: required(&flags, "decision-type")?,
        decision_result: required(&flags, "decision-result")?,
        predecessor_decision_id: flags.get("predecessor-decision-id").map(String::as_str),
        candidate,
        evidence_reference: flags.get("evidence-reference").map(String::as_str),
        content_state,
        safe_rationale: flags.get("safe-rationale").map(String::as_str),
        created_unix_ms: unix_ms()?,
    })?;
    let outcome = store.append_workflow_decision(&decision)?;
    Ok(json!({
        "action":"decision-record",
        "append_outcome":match outcome {
            crate::domain::workflow::DecisionAppendOutcome::Inserted => "INSERTED",
            crate::domain::workflow::DecisionAppendOutcome::IdempotentNoChange => "IDEMPOTENT_NO_CHANGE",
        },
        "decision":decision_record_json(&store.load_workflow_decision(&decision.decision_id)?),
    }))
}
fn list_decisions(flags: HashMap<String, String>) -> Result<Value> {
    ensure_allowed_flags(&flags, &decision_flags(&[]))?;
    let store = open_store(&flags)?;
    let workflow = require_workflow(&store, &flags)?;
    let stage_run_id = optional_exact_stage_id(&store, &flags, &workflow)?;
    let candidate = optional_candidate(&flags)?;
    let evidence = evidence_references(&flags)?;
    let context = DecisionApplicabilityContext::new(candidate, &evidence)?;
    let mut decisions = store.list_workflow_decisions(&workflow.identity.workflow_run_id)?;
    if let Some(stage_run_id) = stage_run_id.as_deref() {
        decisions.retain(|decision| {
            decision.stage_run_id.is_none()
                || decision.stage_run_id.as_deref() == Some(stage_run_id)
        });
    }
    require_bounded_items(decisions.len(), "workflow decision query")?;
    let values = decisions
        .iter()
        .map(|decision| {
            let mut value = decision_record_json(decision);
            value["applicability"] = json!(decision_applicability_str(
                evaluate_decision_applicability(decision, &context),
            ));
            value
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "action":"decision-list",
        "workflow":workflow_json(&workflow),
        "stage_run_id":stage_run_id,
        "decisions":values,
    }))
}

#[derive(Debug, Clone, Copy)]
enum ProjectionKind {
    Status,
    ResumePreview,
    WhyBlocked,
    ReviewerHandoff,
}

fn inspect_projection(flags: HashMap<String, String>, kind: ProjectionKind) -> Result<Value> {
    ensure_allowed_flags(&flags, &projection_flags())?;
    let store = open_store(&flags)?;
    let input = build_projection_input(&store, &flags)?;
    match kind {
        ProjectionKind::Status => {
            let projected = project_workflow_status(&input).map_err(workflow_cli_error)?;
            Ok(status_json(&projected))
        }
        ProjectionKind::ResumePreview => {
            let projected = project_resume_preview(&input).map_err(workflow_cli_error)?;
            Ok(json!({
                "kind":"RESUME_PREVIEW",
                "workflow":workflow_identity_json(&projected.workflow),
                "stage":stage_identity_json(&projected.stage),
                "disposition":resume_disposition_str(projected.disposition),
                "continuation":projected.continuation.map(WorkflowContinuationClass::as_db_str),
                "reconstruction":projected.reconstruction,
            }))
        }
        ProjectionKind::WhyBlocked => {
            let projected = project_why_blocked(&input).map_err(workflow_cli_error)?;
            Ok(why_blocked_json(&projected))
        }
        ProjectionKind::ReviewerHandoff => {
            let projected = project_reviewer_handoff(&input).map_err(workflow_cli_error)?;
            Ok(reviewer_handoff_json(&projected))
        }
    }
}

fn build_projection_input(
    store: &Store,
    flags: &HashMap<String, String>,
) -> Result<WorkflowProjectionInput> {
    let workflow = require_workflow(store, flags)?;
    let stage = require_stage(store, flags)?;
    let current_candidate = optional_candidate(flags)?;
    let evidence = evidence_references(flags)?;
    let applicability_context =
        DecisionApplicabilityContext::new(current_candidate.clone(), &evidence)?;

    let observed_baselines =
        store.list_artifact_baselines_for_stage(&stage.identity.stage_run_id)?;
    require_bounded_items(observed_baselines.len(), "workflow baseline projection")?;
    let baseline_identities = observed_baselines
        .iter()
        .map(|stored| stored.identity.clone())
        .collect::<Vec<_>>();
    let baselines = baseline_identities
        .iter()
        .map(|baseline| {
            let expected_candidate = match baseline.kind {
                ArtifactBaselineKind::ExactGitCandidate
                | ArtifactBaselineKind::WindsVerificationEvidence => current_candidate.clone(),
                ArtifactBaselineKind::PriorStageOutput
                | ArtifactBaselineKind::CanonicalDecision
                | ArtifactBaselineKind::BoundedBlobArtifact => baseline.candidate.clone(),
            };
            let requirement = ArtifactBaselineRequirement::new(
                &stage.identity.stage_run_id,
                baseline.kind,
                &baseline.stable_reference,
                expected_candidate,
            )?;
            Ok::<BaselineEvaluation, Box<dyn std::error::Error + Send + Sync>>(
                evaluate_artifact_baseline_requirement(&requirement, &baseline_identities),
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let decisions = store
        .list_workflow_decisions(&workflow.identity.workflow_run_id)?
        .into_iter()
        .filter(|decision| {
            decision.stage_run_id.is_none()
                || decision.stage_run_id.as_deref() == Some(stage.identity.stage_run_id.as_str())
        })
        .map(|record| DecisionProjectionInput {
            applicability: evaluate_decision_applicability(&record, &applicability_context),
            record,
        })
        .collect::<Vec<_>>();
    require_bounded_items(decisions.len(), "workflow decision projection")?;

    let actor = flags
        .get("binding-id")
        .map(|binding_id| store.load_workflow_actor_binding(binding_id))
        .transpose()?
        .map(|binding| actor_projection_input(binding, &stage.identity.stage_run_id))
        .transpose()?;

    let (lifecycle_source, lifecycle_authority) = stage
        .last_transition
        .as_ref()
        .map(|transition| (transition.source, transition.authority))
        .unwrap_or((TruthSource::WindsObserved, StageTransitionAuthority::None));

    Ok(WorkflowProjectionInput {
        expected_workspace_id: workflow.identity.workspace_id.clone(),
        expected_workstream_id: workflow.identity.workstream_id.clone(),
        workflow: workflow.identity,
        stage: stage.identity,
        lifecycle_state: stage.lifecycle_state,
        lifecycle_source,
        lifecycle_authority,
        actor,
        current_candidate,
        baselines,
        decisions,
        retry_failure: stage.failure_observation,
        retry_outcome_reason: stage.outcome_reason,
        verification: ExternalGateTruth::unknown(),
        human_acceptance: ExternalGateTruth::unknown(),
        authority_ceiling: AuthorityCeilingTruth::Unknown,
        prospective_reconstruction: None,
        reassignment_proven: false,
    })
}

fn actor_projection_input(
    binding: StoredWorkflowActorBinding,
    exact_stage_id: &str,
) -> Result<ActorBindingProjectionInput> {
    if binding.stage_run_id != exact_stage_id {
        return Err("workflow actor binding does not belong to the exact requested stage".into());
    }
    Ok(ActorBindingProjectionInput {
        binding_id: binding.binding_id,
        stage_run_id: binding.stage_run_id,
        winds_session_id: binding.winds_session_id,
        runtime_binding_id: binding.runtime_binding_id,
        continuation: binding.continuation,
        role: ActorRoleTruth::Unknown,
        reconstruction_report: binding.reconstruction_report.map(|stored| stored.report),
    })
}
fn open_store(flags: &HashMap<String, String>) -> Result<Store> {
    let home = PathBuf::from(required(flags, "home")?);
    if !home.is_absolute() {
        return Err("workflow --home must be an absolute path".into());
    }
    Store::open(Path::new(&home))
}

fn workflow_identity_from_flags(flags: &HashMap<String, String>) -> Result<WorkflowRunIdentity> {
    Ok(WorkflowRunIdentity::new(
        required(flags, "workflow-id")?,
        required(flags, "workspace-id")?,
        required(flags, "workstream-id")?,
    )?)
}

fn require_workflow(store: &Store, flags: &HashMap<String, String>) -> Result<StoredWorkflowRun> {
    let expected = workflow_identity_from_flags(flags)?;
    let stored = store.load_workflow_run(&expected.workflow_run_id)?;
    if stored.identity != expected {
        return Err("workflow CLI exact workspace/workstream/workflow identity mismatch".into());
    }
    Ok(stored)
}

fn require_stage(store: &Store, flags: &HashMap<String, String>) -> Result<StoredStageRun> {
    let workflow = require_workflow(store, flags)?;
    let stage_id = required(flags, "stage-id")?;
    let stage = store.load_stage_run(stage_id)?;
    if stage.identity.workflow_run_id != workflow.identity.workflow_run_id {
        return Err("workflow CLI stage does not belong to the exact requested workflow".into());
    }
    Ok(stage)
}

fn optional_exact_stage_id(
    store: &Store,
    flags: &HashMap<String, String>,
    workflow: &StoredWorkflowRun,
) -> Result<Option<String>> {
    let Some(stage_id) = flags.get("stage-id") else {
        return Ok(None);
    };
    let stage = store.load_stage_run(stage_id)?;
    if stage.identity.workflow_run_id != workflow.identity.workflow_run_id {
        return Err(
            "workflow decision stage does not belong to the exact requested workflow".into(),
        );
    }
    Ok(Some(stage.identity.stage_run_id))
}

fn optional_candidate(
    flags: &HashMap<String, String>,
) -> Result<Option<CandidateBaselineIdentity>> {
    match (flags.get("candidate-oid"), flags.get("candidate-tree")) {
        (None, None) => Ok(None),
        (Some(oid), Some(tree)) => Ok(Some(CandidateBaselineIdentity::new(oid, tree)?)),
        _ => Err("candidate identity requires both --candidate-oid and --candidate-tree".into()),
    }
}

fn evidence_references(flags: &HashMap<String, String>) -> Result<Vec<String>> {
    let Some(raw) = flags.get("evidence-json") else {
        return Ok(Vec::new());
    };
    let values: Vec<String> = serde_json::from_str(raw)?;
    require_bounded_items(values.len(), "workflow evidence applicability input")?;
    DecisionApplicabilityContext::new(None, &values)?;
    Ok(values)
}

fn parse_decision_source(value: &str) -> Result<TruthSource> {
    let source = TruthSource::from_db(value)
        .ok_or_else(|| format!("unknown workflow decision source: {value}"))?;
    if source == TruthSource::HumanDecided {
        return Err("workflow CLI cannot create HUMAN_DECIDED truth from a string flag".into());
    }
    Ok(source)
}

fn parse_decision_authority(value: &str) -> Result<StageTransitionAuthority> {
    let authority = StageTransitionAuthority::from_db(value)
        .ok_or_else(|| format!("unknown workflow decision authority: {value}"))?;
    if authority == StageTransitionAuthority::HumanDecision {
        return Err(
            "workflow CLI cannot create HUMAN_DECISION authority from a string flag".into(),
        );
    }
    Ok(authority)
}

fn stage_operation_flags(extra: &[&str]) -> Vec<&'static str> {
    let mut flags = vec![
        "action",
        "home",
        "workspace-id",
        "workstream-id",
        "workflow-id",
        "stage-id",
    ];
    for value in extra {
        flags.push(match *value {
            "operation-id" => "operation-id",
            "to" => "to",
            "failure-class" => "failure-class",
            "checkpoint" => "checkpoint",
            "progress-basis" => "progress-basis",
            "side-effect" => "side-effect",
            "new-stage-id" => "new-stage-id",
            _ => unreachable!("closed T109 stage flag set"),
        });
    }
    flags
}

fn decision_flags(extra: &[&str]) -> Vec<&'static str> {
    let mut flags = vec![
        "action",
        "home",
        "workspace-id",
        "workstream-id",
        "workflow-id",
        "stage-id",
        "candidate-oid",
        "candidate-tree",
        "evidence-reference",
        "evidence-json",
    ];
    for value in extra {
        flags.push(match *value {
            "decision-id" => "decision-id",
            "decision-type" => "decision-type",
            "decision-result" => "decision-result",
            "predecessor-decision-id" => "predecessor-decision-id",
            "source" => "source",
            "authority" => "authority",
            "content-state" => "content-state",
            "safe-rationale" => "safe-rationale",
            _ => unreachable!("closed T109 decision flag set"),
        });
    }
    flags
}

fn projection_flags() -> Vec<&'static str> {
    vec![
        "action",
        "home",
        "workspace-id",
        "workstream-id",
        "workflow-id",
        "stage-id",
        "binding-id",
        "candidate-oid",
        "candidate-tree",
        "evidence-json",
    ]
}

fn require_bounded_items(count: usize, context: &str) -> Result<()> {
    if count > MAX_CLI_ITEMS {
        return Err(format!("{context} exceeds the bounded {MAX_CLI_ITEMS}-item CLI limit").into());
    }
    Ok(())
}

fn workflow_cli_error(error: String) -> Box<dyn std::error::Error + Send + Sync> {
    error.into()
}
fn workflow_json(workflow: &StoredWorkflowRun) -> Value {
    json!({
        "workflow_run_id":workflow.identity.workflow_run_id,
        "workspace_id":workflow.identity.workspace_id,
        "workstream_id":workflow.identity.workstream_id,
        "schema_version":workflow.schema_version,
        "terminal_state":workflow.terminal_state,
        "created_unix_ms":workflow.created_unix_ms,
    })
}

fn workflow_identity_json(workflow: &WorkflowRunIdentity) -> Value {
    json!({
        "workflow_run_id":workflow.workflow_run_id,
        "workspace_id":workflow.workspace_id,
        "workstream_id":workflow.workstream_id,
    })
}

fn stage_json(stage: &StoredStageRun) -> Value {
    json!({
        "stage_run_id":stage.identity.stage_run_id,
        "workflow_run_id":stage.identity.workflow_run_id,
        "stage_key":stage.identity.stage_key,
        "attempt_ordinal":stage.identity.attempt_ordinal,
        "predecessor_stage_run_id":stage.identity.predecessor_stage_run_id,
        "relation":stage.relation.map(StageAttemptRelation::as_db_str),
        "lifecycle_state":stage.lifecycle_state.as_db_str(),
        "failure":stage.failure_observation.as_ref().map(|failure| json!({
            "failure_class":failure.failure_class,
            "checkpoint_identity":failure.checkpoint_identity,
            "material_progress_basis":failure.material_progress_basis,
            "side_effect":side_effect_str(failure.side_effect_truth),
        })),
        "outcome_reason":stage.outcome_reason,
        "last_transition":stage.last_transition.as_ref().map(|transition| json!({
            "operation_id":transition.operation_id,
            "from":transition.from.as_db_str(),
            "to":transition.to.as_db_str(),
            "source":transition.source.as_db_str(),
            "authority":transition.authority.as_db_str(),
        })),
        "created_unix_ms":stage.created_unix_ms,
        "updated_unix_ms":stage.updated_unix_ms,
    })
}

fn stage_identity_json(stage: &StageRunIdentity) -> Value {
    json!({
        "stage_run_id":stage.stage_run_id,
        "workflow_run_id":stage.workflow_run_id,
        "stage_key":stage.stage_key,
        "attempt_ordinal":stage.attempt_ordinal,
        "predecessor_stage_run_id":stage.predecessor_stage_run_id,
    })
}

fn transition_outcome_json(outcome: &StageTransitionOutcome) -> Value {
    match outcome {
        StageTransitionOutcome::Apply(applied) => json!({
            "kind":"APPLIED",
            "operation_id":applied.operation_id,
            "from":applied.from.as_db_str(),
            "to":applied.to.as_db_str(),
            "source":applied.source.as_db_str(),
            "authority":applied.authority.as_db_str(),
        }),
        StageTransitionOutcome::IdempotentNoChange => json!({"kind":"IDEMPOTENT_NO_CHANGE"}),
    }
}

fn decision_record_json(decision: &WorkflowDecisionRecord) -> Value {
    json!({
        "decision_id":decision.decision_id,
        "workflow_run_id":decision.workflow_run_id,
        "stage_run_id":decision.stage_run_id,
        "source":decision.source.as_db_str(),
        "authority":decision.authority.as_db_str(),
        "decision_type":decision.decision_type,
        "decision_result":decision.decision_result,
        "predecessor_decision_id":decision.predecessor_decision_id,
        "candidate":decision.candidate.as_ref().map(candidate_json),
        "evidence_reference":decision.evidence_reference,
        "content_state":decision.content_state.as_db_str(),
        "safe_rationale":decision.safe_rationale,
        "created_unix_ms":decision.created_unix_ms,
    })
}

fn candidate_json(candidate: &CandidateBaselineIdentity) -> Value {
    json!({"oid":candidate.oid,"tree":candidate.tree})
}

fn baseline_json(baseline: &BaselineEvaluation) -> Value {
    json!({
        "stage_run_id":baseline.requirement.stage_run_id,
        "kind":baseline.requirement.kind.as_db_str(),
        "stable_reference":baseline.requirement.stable_reference,
        "candidate":baseline.requirement.candidate.as_ref().map(candidate_json),
        "freshness":baseline_freshness_str(baseline.freshness),
        "matching_baseline_ids":baseline.matching_baseline_ids,
    })
}

fn projected_decision_json(
    decision: &crate::domain::workflow::projection::DecisionProjection,
) -> Value {
    json!({
        "decision_id":decision.decision_id,
        "workflow_run_id":decision.workflow_run_id,
        "stage_run_id":decision.stage_run_id,
        "source":decision.source.as_db_str(),
        "authority":decision.authority.as_db_str(),
        "decision_type":decision.decision_type,
        "decision_result":decision.decision_result,
        "candidate":decision.candidate.as_ref().map(candidate_json),
        "evidence_reference":decision.evidence_reference,
        "content_state":decision.content_state.as_db_str(),
        "created_unix_ms":decision.created_unix_ms,
        "applicability":decision_applicability_str(decision.applicability),
    })
}

fn actor_json(actor: &crate::domain::workflow::projection::ActorBindingProjection) -> Value {
    json!({
        "binding_id":actor.binding_id,
        "stage_run_id":actor.stage_run_id,
        "winds_session_id":actor.winds_session_id,
        "runtime_binding_id":actor.runtime_binding_id,
        "continuation":actor.continuation.as_db_str(),
        "role":match &actor.role {
            ActorRoleTruth::Known { role, source } => json!({"state":"KNOWN","role":role,"source":source.as_db_str()}),
            ActorRoleTruth::Unknown => json!({"state":"UNKNOWN"}),
            ActorRoleTruth::Unavailable => json!({"state":"UNAVAILABLE"}),
        },
        "reconstruction":actor.reconstruction,
    })
}

fn gate_json(gate: &ExternalGateTruth) -> Value {
    json!({
        "state":match gate.state {
            ExternalGateState::Unknown => "UNKNOWN",
            ExternalGateState::Pending => "PENDING",
            ExternalGateState::Satisfied => "SATISFIED",
        },
        "source":gate.source.map(TruthSource::as_db_str),
        "reference":gate.reference,
    })
}

fn authority_ceiling_json(authority: &AuthorityCeilingTruth) -> Value {
    match authority {
        AuthorityCeilingTruth::Unknown => json!({"state":"UNKNOWN"}),
        AuthorityCeilingTruth::Known {
            source,
            authority,
            reference,
        } => json!({
            "state":"KNOWN",
            "source":source.as_db_str(),
            "authority":authority.as_db_str(),
            "reference":reference,
        }),
    }
}
fn status_json(status: &crate::domain::workflow::projection::WorkflowStatusProjection) -> Value {
    json!({
        "kind":"WORKFLOW_STATUS",
        "workflow":workflow_identity_json(&status.workflow),
        "stage":stage_identity_json(&status.stage),
        "lifecycle":{
            "state":status.lifecycle_state.as_db_str(),
            "source":status.lifecycle_source.as_db_str(),
            "authority":status.lifecycle_authority.as_db_str(),
        },
        "actor":status.actor.as_ref().map(actor_json),
        "current_candidate":status.current_candidate.as_ref().map(candidate_json),
        "baselines":status.baselines.iter().map(baseline_json).collect::<Vec<_>>(),
        "decisions":status.decisions.iter().map(projected_decision_json).collect::<Vec<_>>(),
        "retry_failure":status.retry_failure.as_ref().map(|failure| json!({
            "failure_class":failure.failure_class,
            "checkpoint_identity":failure.checkpoint_identity,
            "material_progress_basis":failure.material_progress_basis,
            "side_effect":side_effect_str(failure.side_effect_truth),
        })),
        "retry_outcome_reason":status.retry_outcome_reason,
        "verification":gate_json(&status.verification),
        "human_acceptance":gate_json(&status.human_acceptance),
        "authority_ceiling":authority_ceiling_json(&status.authority_ceiling),
    })
}

fn why_blocked_json(blocked: &crate::domain::workflow::projection::WhyBlockedProjection) -> Value {
    json!({
        "kind":"WHY_BLOCKED",
        "workflow":workflow_identity_json(&blocked.workflow),
        "stage":stage_identity_json(&blocked.stage),
        "blocker":blocker_str(blocked.blocker),
        "required_action":required_action_str(blocked.required_action),
        "source":blocked.source.map(TruthSource::as_db_str),
        "detail_reference":blocked.detail_reference,
    })
}

fn reviewer_handoff_json(
    handoff: &crate::domain::workflow::projection::ReviewerHandoffProjection,
) -> Value {
    json!({
        "kind":"REVIEWER_HANDOFF",
        "workflow":workflow_identity_json(&handoff.workflow),
        "stage":stage_identity_json(&handoff.stage),
        "lifecycle":{
            "state":handoff.lifecycle_state.as_db_str(),
            "source":handoff.lifecycle_source.as_db_str(),
            "authority":handoff.lifecycle_authority.as_db_str(),
        },
        "actor":handoff.actor.as_ref().map(actor_json),
        "current_candidate":handoff.current_candidate.as_ref().map(candidate_json),
        "baselines":handoff.baselines.iter().map(baseline_json).collect::<Vec<_>>(),
        "decisions":handoff.decisions.iter().map(projected_decision_json).collect::<Vec<_>>(),
        "verification":gate_json(&handoff.verification),
        "human_acceptance":gate_json(&handoff.human_acceptance),
        "authority_ceiling":authority_ceiling_json(&handoff.authority_ceiling),
        "blocker":why_blocked_json(&handoff.blocker),
    })
}

fn side_effect_str(value: SideEffectTruth) -> &'static str {
    match value {
        SideEffectTruth::SafeOrIdempotent => "SAFE_OR_IDEMPOTENT",
        SideEffectTruth::AmbiguousNonIdempotent => "AMBIGUOUS_NON_IDEMPOTENT",
    }
}

fn retry_comparison_str(value: crate::domain::workflow::RetryProgressComparison) -> &'static str {
    match value {
        crate::domain::workflow::RetryProgressComparison::MaterialProgress => "MATERIAL_PROGRESS",
        crate::domain::workflow::RetryProgressComparison::NoProgress => "NO_PROGRESS",
    }
}

fn baseline_freshness_str(value: BaselineFreshness) -> &'static str {
    match value {
        BaselineFreshness::Applicable => "APPLICABLE",
        BaselineFreshness::Stale => "STALE",
        BaselineFreshness::Missing => "MISSING",
        BaselineFreshness::Ambiguous => "AMBIGUOUS",
    }
}

fn decision_applicability_str(value: DecisionApplicability) -> &'static str {
    match value {
        DecisionApplicability::Applicable => "APPLICABLE",
        DecisionApplicability::Stale => "STALE",
    }
}

fn resume_disposition_str(value: ResumeDisposition) -> &'static str {
    match value {
        ResumeDisposition::ExactNativeResume => "EXACT_NATIVE_RESUME",
        ResumeDisposition::WindsReconstruction => "WINDS_RECONSTRUCTION",
        ResumeDisposition::Reassignment => "REASSIGNMENT",
        ResumeDisposition::Blocked => "BLOCKED",
        ResumeDisposition::Unavailable => "UNAVAILABLE",
        ResumeDisposition::OwnershipLostHandling => "OWNERSHIP_LOST_HANDLING",
    }
}

fn blocker_str(value: BlockerTruth) -> &'static str {
    match value {
        BlockerTruth::None => "NONE",
        BlockerTruth::WaitingApproval => "WAITING_APPROVAL",
        BlockerTruth::WaitingExternal => "WAITING_EXTERNAL",
        BlockerTruth::RetryRequired => "RETRY_REQUIRED",
        BlockerTruth::RecoveryRequired => "RECOVERY_REQUIRED",
        BlockerTruth::Stale => "STALE",
        BlockerTruth::Unknown => "UNKNOWN",
    }
}

fn required_action_str(value: RequiredAction) -> &'static str {
    match value {
        RequiredAction::None => "NONE",
        RequiredAction::ExplicitApproval => "EXPLICIT_APPROVAL",
        RequiredAction::ExternalCondition => "EXTERNAL_CONDITION",
        RequiredAction::ExplicitRetry => "EXPLICIT_RETRY",
        RequiredAction::Recovery => "RECOVERY",
        RequiredAction::RefreshCanonicalBasis => "REFRESH_CANONICAL_BASIS",
        RequiredAction::Unknown => "UNKNOWN",
    }
}
