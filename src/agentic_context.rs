use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

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
            item_id: format!("{:?}:{}", omitted.kind, omitted.key),
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
                item_id: fact.key.clone(),
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
            item_id: fact.key.clone(),
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
            item_id: fact.key.clone(),
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
