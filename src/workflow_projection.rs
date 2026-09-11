#![allow(
    dead_code,
    reason = "Spec 008 T107 pure workflow projections; presentation callers land in later tasks"
)]

use crate::domain::workflow::{
    BaselineEvaluation, BaselineFreshness, CandidateBaselineIdentity, DecisionApplicability,
    RETRY_OUTCOME_AMBIGUOUS_EFFECT, RETRY_OUTCOME_BUDGET_EXHAUSTED, RETRY_OUTCOME_FAILURE_RECORDED,
    RETRY_OUTCOME_NO_PROGRESS, ReconstructionReport, RetryFailureObservation, StageLifecycleState,
    StageRunIdentity, StageTransitionAuthority, TruthSource, WorkflowContinuationClass,
    WorkflowDecisionRecord, WorkflowRunIdentity,
};

#[cfg(test)]
#[path = "t107_workflow_projection_tests.rs"]
mod t107_workflow_projection_tests;

type ProjectionResult<T> = Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExternalGateState {
    Unknown,
    Pending,
    Satisfied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExternalGateTruth {
    pub(crate) state: ExternalGateState,
    pub(crate) source: Option<TruthSource>,
    pub(crate) reference: Option<String>,
}

impl ExternalGateTruth {
    pub(crate) fn unknown() -> Self {
        Self {
            state: ExternalGateState::Unknown,
            source: None,
            reference: None,
        }
    }

    pub(crate) fn pending(source: Option<TruthSource>) -> Self {
        Self {
            state: ExternalGateState::Pending,
            source,
            reference: None,
        }
    }

    pub(crate) fn satisfied(source: TruthSource, reference: &str) -> ProjectionResult<Self> {
        let reference = required_reference(reference, "external gate reference")?;
        Ok(Self {
            state: ExternalGateState::Satisfied,
            source: Some(source),
            reference: Some(reference),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ActorRoleTruth {
    Known { role: String, source: TruthSource },
    Unknown,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthorityCeilingTruth {
    Known {
        source: TruthSource,
        authority: StageTransitionAuthority,
        reference: Option<String>,
    },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActorBindingProjectionInput {
    pub(crate) binding_id: String,
    pub(crate) stage_run_id: String,
    pub(crate) winds_session_id: String,
    pub(crate) runtime_binding_id: Option<String>,
    pub(crate) continuation: WorkflowContinuationClass,
    pub(crate) role: ActorRoleTruth,
    pub(crate) reconstruction_report: Option<ReconstructionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecisionProjectionInput {
    pub(crate) record: WorkflowDecisionRecord,
    pub(crate) applicability: DecisionApplicability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowProjectionInput {
    pub(crate) expected_workspace_id: String,
    pub(crate) expected_workstream_id: String,
    pub(crate) workflow: WorkflowRunIdentity,
    pub(crate) stage: StageRunIdentity,
    pub(crate) lifecycle_state: StageLifecycleState,
    pub(crate) lifecycle_source: TruthSource,
    pub(crate) lifecycle_authority: StageTransitionAuthority,
    pub(crate) actor: Option<ActorBindingProjectionInput>,
    pub(crate) current_candidate: Option<CandidateBaselineIdentity>,
    pub(crate) baselines: Vec<BaselineEvaluation>,
    pub(crate) decisions: Vec<DecisionProjectionInput>,
    pub(crate) retry_failure: Option<RetryFailureObservation>,
    pub(crate) retry_outcome_reason: Option<String>,
    pub(crate) verification: ExternalGateTruth,
    pub(crate) human_acceptance: ExternalGateTruth,
    pub(crate) authority_ceiling: AuthorityCeilingTruth,
    pub(crate) prospective_reconstruction: Option<ReconstructionReport>,
    pub(crate) reassignment_proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActorBindingProjection {
    pub(crate) binding_id: String,
    pub(crate) stage_run_id: String,
    pub(crate) winds_session_id: String,
    pub(crate) runtime_binding_id: Option<String>,
    pub(crate) continuation: WorkflowContinuationClass,
    pub(crate) role: ActorRoleTruth,
    pub(crate) reconstruction: Option<ReconstructionReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecisionProjection {
    pub(crate) decision_id: String,
    pub(crate) workflow_run_id: String,
    pub(crate) stage_run_id: Option<String>,
    pub(crate) source: TruthSource,
    pub(crate) authority: StageTransitionAuthority,
    pub(crate) decision_type: String,
    pub(crate) decision_result: String,
    pub(crate) candidate: Option<CandidateBaselineIdentity>,
    pub(crate) evidence_reference: Option<String>,
    pub(crate) content_state: crate::domain::workflow::DecisionContentState,
    pub(crate) created_unix_ms: i64,
    pub(crate) applicability: DecisionApplicability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowStatusProjection {
    pub(crate) workflow: WorkflowRunIdentity,
    pub(crate) stage: StageRunIdentity,
    pub(crate) lifecycle_state: StageLifecycleState,
    pub(crate) lifecycle_source: TruthSource,
    pub(crate) lifecycle_authority: StageTransitionAuthority,
    pub(crate) actor: Option<ActorBindingProjection>,
    pub(crate) current_candidate: Option<CandidateBaselineIdentity>,
    pub(crate) baselines: Vec<BaselineEvaluation>,
    pub(crate) decisions: Vec<DecisionProjection>,
    pub(crate) retry_failure: Option<RetryFailureObservation>,
    pub(crate) retry_outcome_reason: Option<String>,
    pub(crate) verification: ExternalGateTruth,
    pub(crate) human_acceptance: ExternalGateTruth,
    pub(crate) authority_ceiling: AuthorityCeilingTruth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResumeDisposition {
    ExactNativeResume,
    WindsReconstruction,
    Reassignment,
    Blocked,
    Unavailable,
    OwnershipLostHandling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResumePreviewProjection {
    pub(crate) workflow: WorkflowRunIdentity,
    pub(crate) stage: StageRunIdentity,
    pub(crate) disposition: ResumeDisposition,
    pub(crate) continuation: Option<WorkflowContinuationClass>,
    pub(crate) reconstruction: Option<ReconstructionReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockerTruth {
    None,
    WaitingApproval,
    WaitingExternal,
    RetryRequired,
    RecoveryRequired,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequiredAction {
    None,
    ExplicitApproval,
    ExternalCondition,
    ExplicitRetry,
    Recovery,
    RefreshCanonicalBasis,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WhyBlockedProjection {
    pub(crate) workflow: WorkflowRunIdentity,
    pub(crate) stage: StageRunIdentity,
    pub(crate) blocker: BlockerTruth,
    pub(crate) required_action: RequiredAction,
    pub(crate) source: Option<TruthSource>,
    pub(crate) detail_reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewerHandoffProjection {
    pub(crate) workflow: WorkflowRunIdentity,
    pub(crate) stage: StageRunIdentity,
    pub(crate) lifecycle_state: StageLifecycleState,
    pub(crate) lifecycle_source: TruthSource,
    pub(crate) lifecycle_authority: StageTransitionAuthority,
    pub(crate) actor: Option<ActorBindingProjection>,
    pub(crate) current_candidate: Option<CandidateBaselineIdentity>,
    pub(crate) baselines: Vec<BaselineEvaluation>,
    pub(crate) decisions: Vec<DecisionProjection>,
    pub(crate) verification: ExternalGateTruth,
    pub(crate) human_acceptance: ExternalGateTruth,
    pub(crate) authority_ceiling: AuthorityCeilingTruth,
    pub(crate) blocker: WhyBlockedProjection,
}

pub(crate) fn project_workflow_status(
    input: &WorkflowProjectionInput,
) -> ProjectionResult<WorkflowStatusProjection> {
    validate_input(input)?;
    Ok(WorkflowStatusProjection {
        workflow: input.workflow.clone(),
        stage: input.stage.clone(),
        lifecycle_state: input.lifecycle_state,
        lifecycle_source: input.lifecycle_source,
        lifecycle_authority: input.lifecycle_authority,
        actor: project_actor(input.actor.as_ref()),
        current_candidate: input.current_candidate.clone(),
        baselines: sorted_baselines(&input.baselines),
        decisions: sorted_decisions(&input.decisions),
        retry_failure: input.retry_failure.clone(),
        retry_outcome_reason: input.retry_outcome_reason.clone(),
        verification: input.verification.clone(),
        human_acceptance: input.human_acceptance.clone(),
        authority_ceiling: input.authority_ceiling.clone(),
    })
}

pub(crate) fn project_resume_preview(
    input: &WorkflowProjectionInput,
) -> ProjectionResult<ResumePreviewProjection> {
    validate_input(input)?;
    let continuation = input.actor.as_ref().map(|actor| actor.continuation);
    let blocked = matches!(
        input.lifecycle_state,
        StageLifecycleState::WaitingApproval
            | StageLifecycleState::WaitingExternal
            | StageLifecycleState::Blocked
            | StageLifecycleState::Failed
            | StageLifecycleState::Stale
            | StageLifecycleState::RecoveryRequired
    );
    let terminal = matches!(
        input.lifecycle_state,
        StageLifecycleState::Cancelled | StageLifecycleState::Completed
    );
    let (disposition, reconstruction) = if blocked {
        (ResumeDisposition::Blocked, None)
    } else if terminal {
        (ResumeDisposition::Unavailable, None)
    } else {
        match continuation {
            Some(WorkflowContinuationClass::Resumed) => {
                (ResumeDisposition::ExactNativeResume, None)
            }
            Some(WorkflowContinuationClass::Reconstructed) => (
                ResumeDisposition::WindsReconstruction,
                input
                    .actor
                    .as_ref()
                    .and_then(|actor| actor.reconstruction_report.clone()),
            ),
            Some(WorkflowContinuationClass::OwnershipLost) => {
                (ResumeDisposition::OwnershipLostHandling, None)
            }
            Some(WorkflowContinuationClass::Unavailable | WorkflowContinuationClass::Unproven)
            | None => {
                if let Some(report) = input.prospective_reconstruction.clone() {
                    (ResumeDisposition::WindsReconstruction, Some(report))
                } else if input.reassignment_proven {
                    (ResumeDisposition::Reassignment, None)
                } else {
                    (ResumeDisposition::Unavailable, None)
                }
            }
        }
    };
    Ok(ResumePreviewProjection {
        workflow: input.workflow.clone(),
        stage: input.stage.clone(),
        disposition,
        continuation,
        reconstruction,
    })
}

pub(crate) fn project_why_blocked(
    input: &WorkflowProjectionInput,
) -> ProjectionResult<WhyBlockedProjection> {
    validate_input(input)?;
    let (blocker, required_action, source, detail_reference) = match input.lifecycle_state {
        StageLifecycleState::WaitingApproval => (
            BlockerTruth::WaitingApproval,
            RequiredAction::ExplicitApproval,
            Some(input.lifecycle_source),
            None,
        ),
        StageLifecycleState::WaitingExternal => (
            BlockerTruth::WaitingExternal,
            RequiredAction::ExternalCondition,
            Some(input.lifecycle_source),
            None,
        ),
        StageLifecycleState::RecoveryRequired => (
            BlockerTruth::RecoveryRequired,
            RequiredAction::Recovery,
            Some(input.lifecycle_source),
            input.retry_outcome_reason.clone(),
        ),
        StageLifecycleState::Failed => match input.retry_outcome_reason.as_deref() {
            Some(RETRY_OUTCOME_BUDGET_EXHAUSTED | RETRY_OUTCOME_AMBIGUOUS_EFFECT) => (
                BlockerTruth::RecoveryRequired,
                RequiredAction::Recovery,
                Some(input.lifecycle_source),
                input.retry_outcome_reason.clone(),
            ),
            Some(RETRY_OUTCOME_FAILURE_RECORDED | RETRY_OUTCOME_NO_PROGRESS) => (
                BlockerTruth::RetryRequired,
                RequiredAction::ExplicitRetry,
                Some(input.lifecycle_source),
                input.retry_outcome_reason.clone(),
            ),
            _ => (BlockerTruth::Unknown, RequiredAction::Unknown, None, None),
        },
        StageLifecycleState::Stale => (
            BlockerTruth::Stale,
            RequiredAction::RefreshCanonicalBasis,
            Some(input.lifecycle_source),
            None,
        ),
        StageLifecycleState::Blocked => {
            (BlockerTruth::Unknown, RequiredAction::Unknown, None, None)
        }
        StageLifecycleState::Prepared
        | StageLifecycleState::Active
        | StageLifecycleState::Cancelled
        | StageLifecycleState::Completed => (BlockerTruth::None, RequiredAction::None, None, None),
    };
    Ok(WhyBlockedProjection {
        workflow: input.workflow.clone(),
        stage: input.stage.clone(),
        blocker,
        required_action,
        source,
        detail_reference,
    })
}

pub(crate) fn project_reviewer_handoff(
    input: &WorkflowProjectionInput,
) -> ProjectionResult<ReviewerHandoffProjection> {
    validate_input(input)?;
    Ok(ReviewerHandoffProjection {
        workflow: input.workflow.clone(),
        stage: input.stage.clone(),
        lifecycle_state: input.lifecycle_state,
        lifecycle_source: input.lifecycle_source,
        lifecycle_authority: input.lifecycle_authority,
        actor: project_actor(input.actor.as_ref()),
        current_candidate: input.current_candidate.clone(),
        baselines: sorted_baselines(&input.baselines),
        decisions: sorted_decisions(&input.decisions),
        verification: input.verification.clone(),
        human_acceptance: input.human_acceptance.clone(),
        authority_ceiling: input.authority_ceiling.clone(),
        blocker: project_why_blocked(input)?,
    })
}

fn validate_input(input: &WorkflowProjectionInput) -> ProjectionResult<()> {
    if input.expected_workspace_id != input.workflow.workspace_id
        || input.expected_workstream_id != input.workflow.workstream_id
    {
        return Err(
            "workflow projection canonical workspace/workstream binding is stale or mismatched"
                .to_owned(),
        );
    }
    if input.stage.workflow_run_id != input.workflow.workflow_run_id {
        return Err("workflow projection stage does not belong to the exact workflow".to_owned());
    }
    validate_lifecycle_authority(
        input.lifecycle_state,
        input.lifecycle_source,
        input.lifecycle_authority,
    )?;
    validate_actor(input)?;
    validate_baselines(input)?;
    validate_decisions(input)?;
    validate_retry_truth(input)?;
    validate_gate(&input.verification, GateKind::Verification)?;
    validate_gate_freshness(&input.verification, &input.decisions)?;
    validate_gate(&input.human_acceptance, GateKind::HumanAcceptance)?;
    validate_gate_freshness(&input.human_acceptance, &input.decisions)?;
    validate_authority_ceiling(&input.authority_ceiling)?;
    validate_prospective_reconstruction(input)?;
    Ok(())
}

fn validate_lifecycle_authority(
    state: StageLifecycleState,
    source: TruthSource,
    authority: StageTransitionAuthority,
) -> ProjectionResult<()> {
    validate_source_authority_pair(source, authority, "lifecycle truth")?;
    if source == TruthSource::AgentReported
        && matches!(
            state,
            StageLifecycleState::Cancelled | StageLifecycleState::Completed
        )
    {
        return Err("agent-reported lifecycle truth cannot project terminal authority".to_owned());
    }
    Ok(())
}

fn validate_source_authority_pair(
    source: TruthSource,
    authority: StageTransitionAuthority,
    context: &str,
) -> ProjectionResult<()> {
    match (source, authority) {
        (TruthSource::AgentReported, StageTransitionAuthority::None)
        | (TruthSource::WindsObserved, StageTransitionAuthority::None)
        | (TruthSource::WindsObserved, StageTransitionAuthority::WindsPolicy)
        | (TruthSource::HumanDecided, StageTransitionAuthority::HumanDecision) => Ok(()),
        _ => Err(format!("{context} source/authority pair is not canonical")),
    }
}

fn validate_actor(input: &WorkflowProjectionInput) -> ProjectionResult<()> {
    let Some(actor) = input.actor.as_ref() else {
        return Ok(());
    };
    required_reference(&actor.binding_id, "actor binding id")?;
    required_reference(&actor.winds_session_id, "Winds session id")?;
    if actor.stage_run_id != input.stage.stage_run_id {
        return Err("actor binding does not belong to the exact stage attempt".to_owned());
    }
    if let Some(runtime_binding_id) = actor.runtime_binding_id.as_deref() {
        required_reference(runtime_binding_id, "runtime binding id")?;
    }
    match &actor.role {
        ActorRoleTruth::Known { role, .. } => {
            required_reference(role, "actor role")?;
        }
        ActorRoleTruth::Unknown | ActorRoleTruth::Unavailable => {}
    }
    match actor.continuation {
        WorkflowContinuationClass::Reconstructed => {
            let report = actor.reconstruction_report.as_ref().ok_or_else(|| {
                "reconstructed actor projection requires its canonical reconstruction report"
                    .to_owned()
            })?;
            validate_reconstruction_binding(report, &actor.binding_id, &input.stage.stage_run_id)?;
        }
        _ if actor.reconstruction_report.is_some() => {
            return Err(
                "non-reconstructed actor projection cannot carry reconstruction truth".to_owned(),
            );
        }
        _ => {}
    }
    Ok(())
}

fn validate_baselines(input: &WorkflowProjectionInput) -> ProjectionResult<()> {
    for baseline in &input.baselines {
        if baseline.requirement.stage_run_id != input.stage.stage_run_id {
            return Err("artifact baseline requirement crosses stage identity".to_owned());
        }
    }
    Ok(())
}

fn validate_decisions(input: &WorkflowProjectionInput) -> ProjectionResult<()> {
    for decision in &input.decisions {
        if decision.record.workflow_run_id != input.workflow.workflow_run_id {
            return Err("workflow decision projection crosses workflow identity".to_owned());
        }
        if let Some(stage_run_id) = decision.record.stage_run_id.as_deref()
            && stage_run_id != input.stage.stage_run_id
        {
            return Err("workflow decision projection crosses stage identity".to_owned());
        }
        if decision.applicability == DecisionApplicability::Applicable
            && decision.record.candidate.is_some()
            && decision.record.candidate != input.current_candidate
        {
            return Err(
                "candidate-bound workflow decision cannot project as applicable after candidate movement"
                    .to_owned(),
            );
        }
    }
    Ok(())
}

fn validate_retry_truth(input: &WorkflowProjectionInput) -> ProjectionResult<()> {
    match (&input.retry_failure, input.retry_outcome_reason.as_deref()) {
        (None, None) => Ok(()),
        (
            Some(_),
            Some(
                RETRY_OUTCOME_FAILURE_RECORDED
                | RETRY_OUTCOME_NO_PROGRESS
                | RETRY_OUTCOME_BUDGET_EXHAUSTED
                | RETRY_OUTCOME_AMBIGUOUS_EFFECT,
            ),
        ) => Ok(()),
        (Some(_), None) => Err("retry projection lacks explicit outcome reason".to_owned()),
        (None, Some(_)) => {
            Err("retry projection outcome lacks normalized failure truth".to_owned())
        }
        (Some(_), Some(_)) => Err("retry projection contains an unknown outcome reason".to_owned()),
    }
}

#[derive(Debug, Clone, Copy)]
enum GateKind {
    Verification,
    HumanAcceptance,
}

fn validate_gate(gate: &ExternalGateTruth, kind: GateKind) -> ProjectionResult<()> {
    match gate.state {
        ExternalGateState::Unknown => {
            if gate.source.is_some() || gate.reference.is_some() {
                return Err("UNKNOWN external gate cannot imply source or evidence".to_owned());
            }
        }
        ExternalGateState::Pending => {
            if gate.reference.is_some() {
                return Err("PENDING external gate cannot carry satisfied evidence".to_owned());
            }
        }
        ExternalGateState::Satisfied => {
            let source = gate
                .source
                .ok_or_else(|| "SATISFIED external gate requires an explicit source".to_owned())?;
            let reference = gate.reference.as_deref().ok_or_else(|| {
                "SATISFIED external gate requires an exact evidence/decision reference".to_owned()
            })?;
            required_reference(reference, "satisfied external gate reference")?;
            match kind {
                GateKind::Verification if source != TruthSource::WindsObserved => {
                    return Err(
                        "candidate verification must remain Winds-observed evidence truth"
                            .to_owned(),
                    );
                }
                GateKind::HumanAcceptance if source != TruthSource::HumanDecided => {
                    return Err(
                        "human acceptance must remain explicitly HUMAN_DECIDED truth".to_owned(),
                    );
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn validate_gate_freshness(
    gate: &ExternalGateTruth,
    decisions: &[DecisionProjectionInput],
) -> ProjectionResult<()> {
    if gate.state != ExternalGateState::Satisfied {
        return Ok(());
    }
    let Some(reference) = gate.reference.as_deref() else {
        return Ok(());
    };
    if decisions.iter().any(|decision| {
        decision.applicability == DecisionApplicability::Stale
            && (decision.record.decision_id == reference
                || decision.record.evidence_reference.as_deref() == Some(reference))
    }) {
        return Err(
            "SATISFIED external gate cannot reference stale decision/evidence truth".to_owned(),
        );
    }
    Ok(())
}

fn validate_authority_ceiling(authority: &AuthorityCeilingTruth) -> ProjectionResult<()> {
    let AuthorityCeilingTruth::Known {
        source,
        authority,
        reference,
    } = authority
    else {
        return Ok(());
    };
    if let Some(reference) = reference.as_deref() {
        required_reference(reference, "authority ceiling reference")?;
    }
    validate_source_authority_pair(*source, *authority, "authority ceiling")
}

fn validate_prospective_reconstruction(input: &WorkflowProjectionInput) -> ProjectionResult<()> {
    let Some(report) = input.prospective_reconstruction.as_ref() else {
        return Ok(());
    };
    if report.stage_run_id != input.stage.stage_run_id {
        return Err("prospective reconstruction crosses stage identity".to_owned());
    }
    required_reference(&report.binding_id, "prospective reconstruction binding id")?;
    Ok(())
}

fn validate_reconstruction_binding(
    report: &ReconstructionReport,
    binding_id: &str,
    stage_run_id: &str,
) -> ProjectionResult<()> {
    if report.binding_id != binding_id || report.stage_run_id != stage_run_id {
        return Err(
            "reconstruction report does not match the exact actor/stage binding".to_owned(),
        );
    }
    Ok(())
}

fn project_actor(actor: Option<&ActorBindingProjectionInput>) -> Option<ActorBindingProjection> {
    actor.map(|actor| ActorBindingProjection {
        binding_id: actor.binding_id.clone(),
        stage_run_id: actor.stage_run_id.clone(),
        winds_session_id: actor.winds_session_id.clone(),
        runtime_binding_id: actor.runtime_binding_id.clone(),
        continuation: actor.continuation,
        role: actor.role.clone(),
        reconstruction: actor.reconstruction_report.clone(),
    })
}

fn sorted_baselines(baselines: &[BaselineEvaluation]) -> Vec<BaselineEvaluation> {
    let mut projected = baselines.to_vec();
    projected.sort_by(|left, right| {
        left.requirement
            .stage_run_id
            .cmp(&right.requirement.stage_run_id)
            .then(left.requirement.kind.cmp(&right.requirement.kind))
            .then(
                left.requirement
                    .stable_reference
                    .cmp(&right.requirement.stable_reference),
            )
            .then(
                baseline_freshness_rank(left.freshness)
                    .cmp(&baseline_freshness_rank(right.freshness)),
            )
            .then(left.matching_baseline_ids.cmp(&right.matching_baseline_ids))
    });
    projected
}

fn sorted_decisions(decisions: &[DecisionProjectionInput]) -> Vec<DecisionProjection> {
    let mut projected = decisions
        .iter()
        .map(|decision| DecisionProjection {
            decision_id: decision.record.decision_id.clone(),
            workflow_run_id: decision.record.workflow_run_id.clone(),
            stage_run_id: decision.record.stage_run_id.clone(),
            source: decision.record.source,
            authority: decision.record.authority,
            decision_type: decision.record.decision_type.clone(),
            decision_result: decision.record.decision_result.clone(),
            candidate: decision.record.candidate.clone(),
            evidence_reference: decision.record.evidence_reference.clone(),
            content_state: decision.record.content_state,
            created_unix_ms: decision.record.created_unix_ms,
            applicability: decision.applicability,
        })
        .collect::<Vec<_>>();
    projected.sort_by(|left, right| {
        left.created_unix_ms
            .cmp(&right.created_unix_ms)
            .then(left.decision_id.cmp(&right.decision_id))
    });
    projected
}

fn baseline_freshness_rank(freshness: BaselineFreshness) -> u8 {
    match freshness {
        BaselineFreshness::Applicable => 0,
        BaselineFreshness::Stale => 1,
        BaselineFreshness::Missing => 2,
        BaselineFreshness::Ambiguous => 3,
    }
}

fn required_reference(value: &str, field: &str) -> ProjectionResult<String> {
    let normalized = value.trim();
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        return Err(format!(
            "{field} must be non-empty printable canonical metadata"
        ));
    }
    Ok(normalized.to_owned())
}
