use crate::agentic_authority::{
    ApprovalContent, AuthorityEvaluation, AuthorityRequest, DelegationContract,
    approval_json_and_digest, evaluate_delegation,
};
use crate::agentic_claude::ClaudeEvidenceClass;
use crate::agentic_runtime::{RuntimeKind, RuntimeSessionBinding};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[cfg(test)]
#[path = "t081_cross_runtime_handoff_tests.rs"]
mod t081_cross_runtime_handoff_tests;
#[cfg(test)]
#[path = "t082_worker_worktree_tests.rs"]
mod t082_worker_worktree_tests;

const CONTEXT_CAPSULE_VERSION: &str = "winds.context.v1";
const CONTEXT_POLICY_VERSION: &str = "winds.context.policy.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextCapsuleError(String);

impl Display for ContextCapsuleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ContextCapsuleError {}

type ContextResult<T> = Result<T, ContextCapsuleError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ContextFactKind {
    Objective,
    Constraint,
    Decision,
}

impl ContextFactKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Objective => "OBJECTIVE",
            Self::Constraint => "CONSTRAINT",
            Self::Decision => "DECISION",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ContextProvenance {
    WindsObserved,
    HumanDecided,
    ImportedHistory,
    DerivedReconstructed,
}

impl ContextProvenance {
    fn authority_rank(self) -> u8 {
        match self {
            Self::WindsObserved | Self::HumanDecided => 3,
            Self::ImportedHistory => 2,
            Self::DerivedReconstructed => 1,
        }
    }

    fn deterministic_rank(self) -> u8 {
        match self {
            Self::WindsObserved => 0,
            Self::HumanDecided => 1,
            Self::ImportedHistory => 2,
            Self::DerivedReconstructed => 3,
        }
    }

    fn is_protected(self) -> bool {
        matches!(self, Self::WindsObserved | Self::HumanDecided)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextFactInput {
    pub kind: ContextFactKind,
    pub key: String,
    pub value: String,
    pub provenance: ContextProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextReferenceInput {
    pub reference_id: String,
    pub exact_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextUnavailableInput {
    pub item_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextCapsuleInput {
    pub workspace_id: String,
    pub workstream_id: String,
    pub session_id: String,
    pub facts: Vec<ContextFactInput>,
    pub candidate_references: Vec<ContextReferenceInput>,
    pub evidence_references: Vec<ContextReferenceInput>,
    pub unavailable: Vec<ContextUnavailableInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CanonicalContextFact {
    pub kind: ContextFactKind,
    pub key: String,
    pub value: String,
    pub provenance: ContextProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CanonicalContextReference {
    pub reference_id: String,
    pub exact_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HiddenStateAvailability {
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct HiddenStateBoundary {
    pub state: HiddenStateAvailability,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContextCapsulePayload {
    pub version: String,
    pub policy_version: String,
    pub workspace_id: String,
    pub workstream_id: String,
    pub session_id: String,
    pub facts: Vec<CanonicalContextFact>,
    pub candidate_references: Vec<CanonicalContextReference>,
    pub evidence_references: Vec<CanonicalContextReference>,
    pub private_hidden_state: HiddenStateBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum TransferDisposition {
    Transferred,
    DerivedReconstructed,
    Omitted,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TransferReportEntry {
    pub item_type: String,
    pub item_id: String,
    pub disposition: TransferDisposition,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ContextTransferReport {
    pub entries: Vec<TransferReportEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextCapsule {
    pub payload: ContextCapsulePayload,
    pub canonical_json: String,
    pub sha256: String,
    pub transfer_report: ContextTransferReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CompactedContextView {
    pub source_capsule_sha256: String,
    pub facts: Vec<CanonicalContextFact>,
    pub candidate_references: Vec<CanonicalContextReference>,
    pub evidence_references: Vec<CanonicalContextReference>,
    pub transfer_report: ContextTransferReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CrossRuntimeTransferReport {
    pub source_runtime: RuntimeKind,
    pub destination_runtime: RuntimeKind,
    pub source_session_id: String,
    pub destination_session_id: String,
    pub canonical_workstream_id: String,
    pub context: ContextTransferReport,
}

pub(crate) struct CrossRuntimeHandoffInput<'a> {
    pub capsule: &'a ContextCapsule,
    pub source_binding: &'a RuntimeSessionBinding,
    pub destination_binding: &'a RuntimeSessionBinding,
    pub planner_worker_proposal: &'a str,
    pub approval_content: &'a ApprovalContent,
    pub delegation_contract: &'a DelegationContract,
    pub authority_request: &'a AuthorityRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CrossRuntimeHandoffContract {
    pub workspace_id: String,
    pub workstream_id: String,
    pub planner_worker_proposal: String,
    pub proposal_evidence: ClaudeEvidenceClass,
    pub transfer_report: CrossRuntimeTransferReport,
    pub normalized_contract_json: String,
    pub normalized_contract_sha256: String,
    pub authority_evaluation: AuthorityEvaluation,
    pub human_approval_required: bool,
    pub worker_execution_authorized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandoffContractMatch {
    Exact,
    Changed,
}

pub(crate) fn build_cross_runtime_handoff(
    input: CrossRuntimeHandoffInput<'_>,
) -> ContextResult<CrossRuntimeHandoffContract> {
    if input.source_binding.runtime != RuntimeKind::Claude
        || input.destination_binding.runtime != RuntimeKind::Codex
    {
        return Err(ContextCapsuleError(
            "T081 requires the exact Claude-Planner to Codex-Worker runtime direction".to_owned(),
        ));
    }
    if input.source_binding.session_id != input.capsule.payload.session_id {
        return Err(ContextCapsuleError(
            "T081 source runtime binding does not match the canonical Planner session".to_owned(),
        ));
    }

    let planner_worker_proposal =
        normalize_required(input.planner_worker_proposal, "Planner Worker proposal")?;
    let approval = input.approval_content;
    let contract = input.delegation_contract;
    let request = input.authority_request;

    if input.destination_binding.session_id != approval.session_id {
        return Err(ContextCapsuleError(
            "T081 destination runtime binding does not match the approved Worker session"
                .to_owned(),
        ));
    }

    let (normalized_contract_json, normalized_contract_sha256) = approval_json_and_digest(approval)
        .map_err(|error| {
            ContextCapsuleError(format!("T081 approval contract is invalid: {error}"))
        })?;

    if approval.workspace_id != input.capsule.payload.workspace_id {
        return Err(ContextCapsuleError(
            "T081 approval workspace does not match the canonical context capsule".to_owned(),
        ));
    }
    if approval.workstream_id != input.capsule.payload.workstream_id {
        return Err(ContextCapsuleError(
            "T081 approval workstream does not match the canonical context capsule".to_owned(),
        ));
    }
    if approval.context_digest.trim().to_ascii_lowercase() != input.capsule.sha256 {
        return Err(ContextCapsuleError(
            "T081 approval context digest does not match the exact canonical capsule".to_owned(),
        ));
    }
    if approval.runtime_kind.trim() != "CODEX" {
        return Err(ContextCapsuleError(
            "T081 destination approval must bind the Codex runtime".to_owned(),
        ));
    }
    if contract.planner_id != approval.planner_id || contract.workers.len() != 1 {
        return Err(ContextCapsuleError(
            "T081 requires exactly one Planner and one directly delegated Worker".to_owned(),
        ));
    }

    let worker = &contract.workers[0];
    if worker.worker_id != approval.worker_id
        || worker.parent_planner_id != approval.worker_parent_planner_id
        || worker.parent_planner_id != contract.planner_id
        || worker.worker_id == contract.planner_id
    {
        return Err(ContextCapsuleError(
            "T081 Worker topology does not match the exact approval contract".to_owned(),
        ));
    }
    if request.worker_id != approval.worker_id || request.target != approval.target {
        return Err(ContextCapsuleError(
            "T081 authority request does not match the exact approved Worker target".to_owned(),
        ));
    }
    if contract.planner_delegation_ceiling != approval.planner_delegation_ceiling
        || worker.authority != approval.worker_grant
        || contract.team_policy != approval.team_policy
        || contract.human_ceiling != approval.human_ceiling
        || contract.enforcement != approval.enforcement
    {
        return Err(ContextCapsuleError(
            "T081 delegation policy differs from the normalized approval contract".to_owned(),
        ));
    }

    let authority_evaluation = evaluate_delegation(contract, request);
    Ok(CrossRuntimeHandoffContract {
        workspace_id: input.capsule.payload.workspace_id.clone(),
        workstream_id: input.capsule.payload.workstream_id.clone(),
        planner_worker_proposal,
        proposal_evidence: ClaudeEvidenceClass::AgentReported,
        transfer_report: CrossRuntimeTransferReport {
            source_runtime: input.source_binding.runtime,
            destination_runtime: input.destination_binding.runtime,
            source_session_id: input.source_binding.session_id.clone(),
            destination_session_id: input.destination_binding.session_id.clone(),
            canonical_workstream_id: input.capsule.payload.workstream_id.clone(),
            context: input.capsule.transfer_report.clone(),
        },
        normalized_contract_json,
        normalized_contract_sha256,
        authority_evaluation,
        human_approval_required: true,
        worker_execution_authorized: false,
    })
}

pub(crate) fn revalidate_handoff_content(
    handoff: &CrossRuntimeHandoffContract,
    current_content: &ApprovalContent,
) -> ContextResult<HandoffContractMatch> {
    let (_, current_digest) = approval_json_and_digest(current_content).map_err(|error| {
        ContextCapsuleError(format!(
            "T081 current approval contract is invalid: {error}"
        ))
    })?;
    Ok(if current_digest == handoff.normalized_contract_sha256 {
        HandoffContractMatch::Exact
    } else {
        HandoffContractMatch::Changed
    })
}

pub(crate) fn build_context_capsule(input: ContextCapsuleInput) -> ContextResult<ContextCapsule> {
    let workspace_id = normalize_required(&input.workspace_id, "workspace id")?;
    let workstream_id = normalize_required(&input.workstream_id, "workstream id")?;
    let session_id = normalize_required(&input.session_id, "session id")?;

    let mut report_entries = Vec::new();
    let facts = canonicalize_facts(input.facts, &mut report_entries)?;
    let candidate_references = canonicalize_references(
        "candidate_reference",
        input.candidate_references,
        &mut report_entries,
    )?;
    let evidence_references = canonicalize_references(
        "evidence_reference",
        input.evidence_references,
        &mut report_entries,
    )?;

    for unavailable in input.unavailable {
        report_entries.push(TransferReportEntry {
            item_type: "unavailable".to_owned(),
            item_id: normalize_required(&unavailable.item_id, "unavailable item id")?,
            disposition: TransferDisposition::Unavailable,
            detail: normalize_required(&unavailable.reason, "unavailable item reason")?,
        });
    }
    report_entries.push(TransferReportEntry {
        item_type: "private_hidden_state".to_owned(),
        item_id: "private_hidden_state_reasoning".to_owned(),
        disposition: TransferDisposition::Unavailable,
        detail:
            "Private hidden state/reasoning is not transferred or reconstructed as canonical truth."
                .to_owned(),
    });
    sort_transfer_entries(&mut report_entries);

    let payload = ContextCapsulePayload {
        version: CONTEXT_CAPSULE_VERSION.to_owned(),
        policy_version: CONTEXT_POLICY_VERSION.to_owned(),
        workspace_id,
        workstream_id,
        session_id,
        facts,
        candidate_references,
        evidence_references,
        private_hidden_state: HiddenStateBoundary {
            state: HiddenStateAvailability::Unavailable,
            statement:
                "Private hidden state/reasoning is unavailable and is never represented as transferred truth."
                    .to_owned(),
        },
    };
    let canonical_bytes = serde_json::to_vec(&payload)
        .map_err(|error| ContextCapsuleError(format!("context serialization failed: {error}")))?;
    let sha256 = format!("{:x}", Sha256::digest(&canonical_bytes));
    let canonical_json = String::from_utf8(canonical_bytes).map_err(|error| {
        ContextCapsuleError(format!("context serialization was not UTF-8: {error}"))
    })?;

    Ok(ContextCapsule {
        payload,
        canonical_json,
        sha256,
        transfer_report: ContextTransferReport {
            entries: report_entries,
        },
    })
}

pub(crate) fn compact_context_view(
    capsule: &ContextCapsule,
    max_facts: usize,
) -> CompactedContextView {
    let mut ranked = capsule.payload.facts.clone();
    ranked.sort_by(|left, right| {
        right
            .provenance
            .authority_rank()
            .cmp(&left.provenance.authority_rank())
            .then_with(|| {
                left.provenance
                    .deterministic_rank()
                    .cmp(&right.provenance.deterministic_rank())
            })
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.key.cmp(&right.key))
            .then_with(|| left.value.cmp(&right.value))
    });

    let retained_count = max_facts.min(ranked.len());
    let mut retained = ranked[..retained_count].to_vec();
    retained.sort_by(canonical_fact_order);

    let mut report_entries = capsule.transfer_report.entries.clone();
    for omitted in &ranked[retained_count..] {
        report_entries.push(TransferReportEntry {
            item_type: "fact".to_owned(),
            item_id: context_fact_report_id(omitted.kind, &omitted.key),
            disposition: TransferDisposition::Omitted,
            detail: "Omitted from compacted view only; canonical capsule truth is unchanged."
                .to_owned(),
        });
    }
    sort_transfer_entries(&mut report_entries);

    CompactedContextView {
        source_capsule_sha256: capsule.sha256.clone(),
        facts: retained,
        candidate_references: capsule.payload.candidate_references.clone(),
        evidence_references: capsule.payload.evidence_references.clone(),
        transfer_report: ContextTransferReport {
            entries: report_entries,
        },
    }
}

fn canonicalize_facts(
    facts: Vec<ContextFactInput>,
    report_entries: &mut Vec<TransferReportEntry>,
) -> ContextResult<Vec<CanonicalContextFact>> {
    let mut normalized = facts
        .into_iter()
        .map(|fact| {
            Ok(CanonicalContextFact {
                kind: fact.kind,
                key: normalize_required(&fact.key, "context fact key")?,
                value: normalize_required(&fact.value, "context fact value")?,
                provenance: fact.provenance,
            })
        })
        .collect::<ContextResult<Vec<_>>>()?;
    normalized.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.key.cmp(&right.key))
            .then_with(|| {
                right
                    .provenance
                    .authority_rank()
                    .cmp(&left.provenance.authority_rank())
            })
            .then_with(|| {
                left.provenance
                    .deterministic_rank()
                    .cmp(&right.provenance.deterministic_rank())
            })
            .then_with(|| left.value.cmp(&right.value))
    });

    let mut selected: BTreeMap<(ContextFactKind, String), CanonicalContextFact> = BTreeMap::new();
    for fact in normalized {
        let identity = (fact.kind, fact.key.clone());
        let Some(existing) = selected.get(&identity) else {
            selected.insert(identity, fact);
            continue;
        };

        if existing.value == fact.value {
            report_entries.push(TransferReportEntry {
                item_type: "fact".to_owned(),
                item_id: context_fact_report_id(fact.kind, &fact.key),
                disposition: TransferDisposition::Omitted,
                detail: "Duplicate fact omitted after deterministic normalization.".to_owned(),
            });
            continue;
        }

        if existing.provenance.is_protected() && fact.provenance.is_protected() {
            return Err(ContextCapsuleError(format!(
                "conflicting protected context fact: {:?}:{}",
                fact.kind, fact.key
            )));
        }
        if existing.provenance.authority_rank() == fact.provenance.authority_rank() {
            return Err(ContextCapsuleError(format!(
                "ambiguous context fact at equal authority: {:?}:{}",
                fact.kind, fact.key
            )));
        }

        report_entries.push(TransferReportEntry {
            item_type: "fact".to_owned(),
            item_id: context_fact_report_id(fact.kind, &fact.key),
            disposition: TransferDisposition::Omitted,
            detail: format!(
                "Lower-authority {:?} fact cannot overwrite selected {:?} fact.",
                fact.provenance, existing.provenance
            ),
        });
    }

    let mut canonical = selected.into_values().collect::<Vec<_>>();
    canonical.sort_by(canonical_fact_order);
    for fact in &canonical {
        report_entries.push(TransferReportEntry {
            item_type: "fact".to_owned(),
            item_id: context_fact_report_id(fact.kind, &fact.key),
            disposition: if fact.provenance == ContextProvenance::DerivedReconstructed {
                TransferDisposition::DerivedReconstructed
            } else {
                TransferDisposition::Transferred
            },
            detail: format!(
                "Canonical fact retained with {:?} provenance.",
                fact.provenance
            ),
        });
    }
    Ok(canonical)
}

fn canonicalize_references(
    item_type: &str,
    references: Vec<ContextReferenceInput>,
    report_entries: &mut Vec<TransferReportEntry>,
) -> ContextResult<Vec<CanonicalContextReference>> {
    let mut by_id = BTreeMap::<String, String>::new();
    for reference in references {
        let reference_id = normalize_required(&reference.reference_id, "context reference id")?;
        let exact_identity =
            normalize_required(&reference.exact_identity, "context reference identity")?;
        if let Some(existing) = by_id.get(&reference_id) {
            if existing != &exact_identity {
                return Err(ContextCapsuleError(format!(
                    "ambiguous exact identity for context reference: {reference_id}"
                )));
            }
            report_entries.push(TransferReportEntry {
                item_type: item_type.to_owned(),
                item_id: reference_id,
                disposition: TransferDisposition::Omitted,
                detail: "Duplicate exact reference omitted.".to_owned(),
            });
            continue;
        }
        by_id.insert(reference_id, exact_identity);
    }

    let canonical = by_id
        .into_iter()
        .map(|(reference_id, exact_identity)| CanonicalContextReference {
            reference_id,
            exact_identity,
        })
        .collect::<Vec<_>>();
    for reference in &canonical {
        report_entries.push(TransferReportEntry {
            item_type: item_type.to_owned(),
            item_id: reference.reference_id.clone(),
            disposition: TransferDisposition::Transferred,
            detail: "Exact reference retained.".to_owned(),
        });
    }
    Ok(canonical)
}

fn canonical_fact_order(
    left: &CanonicalContextFact,
    right: &CanonicalContextFact,
) -> std::cmp::Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.key.cmp(&right.key))
        .then_with(|| left.provenance.cmp(&right.provenance))
        .then_with(|| left.value.cmp(&right.value))
}

fn context_fact_report_id(kind: ContextFactKind, key: &str) -> String {
    format!("{}:{key}", kind.as_str())
}

fn sort_transfer_entries(entries: &mut [TransferReportEntry]) {
    entries.sort_by(|left, right| {
        left.item_type
            .cmp(&right.item_type)
            .then_with(|| left.item_id.cmp(&right.item_id))
            .then_with(|| left.disposition.cmp(&right.disposition))
            .then_with(|| left.detail.cmp(&right.detail))
    });
}

fn normalize_required(value: &str, label: &str) -> ContextResult<String> {
    if value.contains('\0') {
        return Err(ContextCapsuleError(format!("{label} must not contain NUL")));
    }
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    let normalized = normalized.trim().to_owned();
    if normalized.is_empty() {
        return Err(ContextCapsuleError(format!("{label} must not be empty")));
    }
    Ok(normalized)
}
