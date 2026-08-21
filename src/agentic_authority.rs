use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthorityDecision {
    Deny,
    Ask,
    Allow,
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
        .filter_map(|(source, decision)| {
            (*decision == AuthorityDecision::Deny).then_some(*source)
        })
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
        .filter_map(|(source, decision)| {
            (*decision == AuthorityDecision::Ask).then_some(*source)
        })
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
