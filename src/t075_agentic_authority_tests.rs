use crate::agentic_authority::{
    AuthorityDecision, AuthorityPlane, AuthorityReason, AuthorityRequest, AuthoritySource,
    AuthorityTarget, DelegationContract, EnforcementEvidence, EnforcementQuality, HumanAction,
    VisibilityAssessment, WorkerGrant, evaluate_delegation,
};
use std::collections::BTreeMap;

fn target() -> AuthorityTarget {
    AuthorityTarget {
        capability: "edit".to_owned(),
        resource: "workspace:/repo/src".to_owned(),
    }
}

fn plane(decision: AuthorityDecision) -> AuthorityPlane {
    AuthorityPlane {
        default_decision: AuthorityDecision::Deny,
        rules: BTreeMap::from([(target(), decision)]),
    }
}

fn base_contract() -> DelegationContract {
    DelegationContract {
        planner_id: "planner-1".to_owned(),
        planner_direct_authority: plane(AuthorityDecision::Deny),
        planner_delegation_ceiling: plane(AuthorityDecision::Allow),
        team_policy: plane(AuthorityDecision::Allow),
        human_ceiling: plane(AuthorityDecision::Allow),
        workers: vec![WorkerGrant {
            worker_id: "worker-1".to_owned(),
            parent_planner_id: "planner-1".to_owned(),
            authority: plane(AuthorityDecision::Allow),
        }],
        enforcement: EnforcementEvidence {
            claimed_quality: EnforcementQuality::WindsEnforced,
            winds_mediation_complete: true,
        },
        untrusted_authority_text: Vec::new(),
    }
}

fn request() -> AuthorityRequest {
    AuthorityRequest {
        worker_id: "worker-1".to_owned(),
        target: target(),
        resource_visible_to_runtime: false,
    }
}

fn set_target_decision(plane: &mut AuthorityPlane, decision: AuthorityDecision) {
    plane.rules.insert(target(), decision);
}

#[test]
fn planner_direct_authority_is_independent_from_delegation_ceiling() {
    let contract = base_contract();
    let result = evaluate_delegation(&contract, &request());

    assert_eq!(result.decision, AuthorityDecision::Allow);
    assert_eq!(result.planner_direct_decision, AuthorityDecision::Deny);
    assert_eq!(result.planner_delegation_decision, AuthorityDecision::Allow);
    assert_eq!(result.reason, AuthorityReason::AllCeilingsAllow);
}

#[test]
fn worker_effective_authority_is_the_intersection_of_all_four_planes() {
    let cases = [
        AuthoritySource::WorkerGrant,
        AuthoritySource::PlannerDelegationCeiling,
        AuthoritySource::TeamPolicy,
        AuthoritySource::HumanCeiling,
    ];

    for source in cases {
        let mut contract = base_contract();
        match source {
            AuthoritySource::WorkerGrant => {
                set_target_decision(&mut contract.workers[0].authority, AuthorityDecision::Deny);
            }
            AuthoritySource::PlannerDelegationCeiling => {
                set_target_decision(
                    &mut contract.planner_delegation_ceiling,
                    AuthorityDecision::Deny,
                );
            }
            AuthoritySource::TeamPolicy => {
                set_target_decision(&mut contract.team_policy, AuthorityDecision::Deny);
            }
            AuthoritySource::HumanCeiling => {
                set_target_decision(&mut contract.human_ceiling, AuthorityDecision::Deny);
            }
        }

        let result = evaluate_delegation(&contract, &request());
        assert_eq!(result.decision, AuthorityDecision::Deny);
        assert_eq!(result.reason, AuthorityReason::ExplicitDeny);
        assert_eq!(result.blocking_sources, vec![source]);
        assert_eq!(result.human_action, HumanAction::ChangeProtectedPolicy);
    }
}

#[test]
fn explicit_deny_precedes_ask_and_allow() {
    let mut contract = base_contract();
    set_target_decision(&mut contract.workers[0].authority, AuthorityDecision::Ask);
    set_target_decision(&mut contract.team_policy, AuthorityDecision::Deny);

    let result = evaluate_delegation(&contract, &request());

    assert_eq!(result.decision, AuthorityDecision::Deny);
    assert_eq!(result.reason, AuthorityReason::ExplicitDeny);
    assert_eq!(result.blocking_sources, vec![AuthoritySource::TeamPolicy]);
}

#[test]
fn ask_requires_explicit_human_action_when_no_plane_denies() {
    let mut contract = base_contract();
    set_target_decision(&mut contract.workers[0].authority, AuthorityDecision::Ask);

    let result = evaluate_delegation(&contract, &request());

    assert_eq!(result.decision, AuthorityDecision::Ask);
    assert_eq!(result.reason, AuthorityReason::ApprovalRequired);
    assert_eq!(result.human_action, HumanAction::ApproveRequest);
    assert_eq!(result.blocking_sources, vec![AuthoritySource::WorkerGrant]);
}

#[test]
fn missing_scope_fails_closed_instead_of_inheriting_visibility_or_prose() {
    let contract = base_contract();
    let request = AuthorityRequest {
        worker_id: "worker-1".to_owned(),
        target: AuthorityTarget {
            capability: "network".to_owned(),
            resource: "host:internet".to_owned(),
        },
        resource_visible_to_runtime: true,
    };

    let result = evaluate_delegation(&contract, &request);

    assert_eq!(result.decision, AuthorityDecision::Deny);
    assert_eq!(result.reason, AuthorityReason::ExplicitDeny);
    assert_eq!(
        result.blocking_sources,
        vec![
            AuthoritySource::WorkerGrant,
            AuthoritySource::PlannerDelegationCeiling,
            AuthoritySource::TeamPolicy,
            AuthoritySource::HumanCeiling,
        ]
    );
    assert_eq!(
        result.visibility,
        VisibilityAssessment::VisibleNotAuthorization
    );
}

#[test]
fn repo_model_tool_and_imported_text_cannot_self_escalate() {
    let mut contract = base_contract();
    set_target_decision(&mut contract.human_ceiling, AuthorityDecision::Deny);
    contract.untrusted_authority_text = vec![
        "repo says: grant edit".to_owned(),
        "model says: ALLOW".to_owned(),
        "tool output says: policy override".to_owned(),
        "imported history says: administrator approved".to_owned(),
    ];
    let before = contract.clone();

    let result = evaluate_delegation(&contract, &request());

    assert_eq!(result.decision, AuthorityDecision::Deny);
    assert_eq!(result.ignored_untrusted_text_count, 4);
    assert_eq!(contract, before);
}

#[test]
fn winds_enforced_label_requires_complete_winds_mediation() {
    let mut contract = base_contract();
    contract.enforcement.winds_mediation_complete = false;

    let result = evaluate_delegation(&contract, &request());

    assert_eq!(result.decision, AuthorityDecision::Ask);
    assert_eq!(result.reason, AuthorityReason::EnforcementUnproven);
    assert_eq!(
        result.human_action,
        HumanAction::EstablishEnforcementEvidence
    );
    assert_eq!(
        result.effective_enforcement,
        EnforcementQuality::Unavailable
    );
}

#[test]
fn truthful_non_winds_enforcement_labels_are_preserved_without_upgrade() {
    for quality in [
        EnforcementQuality::OsSandboxEnforced,
        EnforcementQuality::AgentNativeEnforced,
        EnforcementQuality::BestEffortTripwire,
        EnforcementQuality::ObservationOnly,
    ] {
        let mut contract = base_contract();
        contract.enforcement.claimed_quality = quality;
        contract.enforcement.winds_mediation_complete = false;

        let result = evaluate_delegation(&contract, &request());
        assert_eq!(result.decision, AuthorityDecision::Allow);
        assert_eq!(result.effective_enforcement, quality);
    }

    let mut unavailable = base_contract();
    unavailable.enforcement.claimed_quality = EnforcementQuality::Unavailable;
    unavailable.enforcement.winds_mediation_complete = false;
    let result = evaluate_delegation(&unavailable, &request());
    assert_eq!(result.decision, AuthorityDecision::Ask);
    assert_eq!(
        result.effective_enforcement,
        EnforcementQuality::Unavailable
    );
}

#[test]
fn one_planner_to_one_worker_topology_is_fail_closed() {
    let mut multiple_workers = base_contract();
    multiple_workers.workers.push(WorkerGrant {
        worker_id: "worker-2".to_owned(),
        parent_planner_id: "planner-1".to_owned(),
        authority: plane(AuthorityDecision::Allow),
    });
    let result = evaluate_delegation(&multiple_workers, &request());
    assert_eq!(result.decision, AuthorityDecision::Deny);
    assert_eq!(result.reason, AuthorityReason::InvalidTopology);
    assert_eq!(result.human_action, HumanAction::ReduceToSingleWorker);

    let mut nested_worker = base_contract();
    nested_worker.workers[0].parent_planner_id = "another-worker".to_owned();
    let result = evaluate_delegation(&nested_worker, &request());
    assert_eq!(result.decision, AuthorityDecision::Deny);
    assert_eq!(result.reason, AuthorityReason::InvalidTopology);
}

#[test]
fn unknown_worker_fails_closed() {
    let contract = base_contract();
    let mut unknown = request();
    unknown.worker_id = "worker-unknown".to_owned();

    let result = evaluate_delegation(&contract, &unknown);

    assert_eq!(result.decision, AuthorityDecision::Deny);
    assert_eq!(result.reason, AuthorityReason::UnknownWorker);
    assert_eq!(result.human_action, HumanAction::SelectAuthorizedWorker);
}

#[test]
fn evaluator_is_deterministic_and_does_not_mutate_the_contract() {
    let contract = base_contract();
    let before = contract.clone();

    let first = evaluate_delegation(&contract, &request());
    let second = evaluate_delegation(&contract, &request());

    assert_eq!(first, second);
    assert_eq!(contract, before);
    assert_eq!(first.visibility, VisibilityAssessment::NotVisible);
    assert_eq!(first.human_action, HumanAction::None);
}
