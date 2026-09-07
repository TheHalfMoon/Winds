use crate::domain::{
    CandidateBindingStatus, CandidateIdentity, IndependentReviewContext,
    VerificationEvidenceReference,
};
use crate::git::Repo;
use crate::store::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentProgressProjection {
    NotReported,
    AgentReportedDone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerificationProjectionState {
    NotRun,
    Running,
    VerifiedForExactCandidate,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReviewProjectionState {
    NotAvailable,
    ApplicableToExactCandidate,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HumanAcceptanceProjectionState {
    NotAccepted,
    AcceptedForExactCandidate,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactCandidateProjection {
    pub(crate) oid: String,
    pub(crate) tree: String,
    binding: CandidateIdentity,
}

impl ExactCandidateProjection {
    fn observe(repo: &Repo, candidate_ref: &str) -> Result<Self, String> {
        let oid = repo
            .resolve_commit(candidate_ref)
            .map_err(|error| format!("T094 candidate observation failed: {error}"))?;
        let tree = repo
            .tree_oid(&oid)
            .map_err(|error| format!("T094 candidate tree observation failed: {error}"))?;
        let binding = CandidateIdentity::new(&oid, &tree)?;
        Ok(Self { oid, tree, binding })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadOnlyDiffProjection {
    pub(crate) base_oid: String,
    pub(crate) candidate_oid: String,
    pub(crate) candidate_tree: String,
    bytes: Vec<u8>,
}

impl ReadOnlyDiffProjection {
    fn from_existing_read(
        base_oid: String,
        candidate: &ExactCandidateProjection,
        bytes: &[u8],
    ) -> Self {
        Self {
            base_oid,
            candidate_oid: candidate.oid.clone(),
            candidate_tree: candidate.tree.clone(),
            bytes: bytes.to_vec(),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) const fn grants_file_authority(&self) -> bool {
        false
    }

    pub(crate) const fn grants_agent_authority(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VerificationProjection {
    pub(crate) state: VerificationProjectionState,
    pub(crate) applicable_evidence_count: usize,
    pub(crate) stale_evidence_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkbenchCandidateContext {
    pub(crate) candidate: ExactCandidateProjection,
    pub(crate) diff: Option<ReadOnlyDiffProjection>,
    pub(crate) agent_progress: AgentProgressProjection,
    pub(crate) verification: VerificationProjection,
    pub(crate) review: ReviewProjectionState,
    pub(crate) human_acceptance: HumanAcceptanceProjectionState,
}

impl WorkbenchCandidateContext {
    /// T094 is a read-only presentation projection. It cannot itself mutate
    /// evidence, human decisions, files, Agents, or Git state.
    pub(crate) const fn changes_canonical_authority(&self) -> bool {
        false
    }
}

pub(crate) struct WorkbenchContextInput<'a> {
    pub(crate) repo: &'a Repo,
    pub(crate) store: &'a Store,
    pub(crate) base_ref: &'a str,
    pub(crate) candidate_ref: &'a str,
    /// Bytes must come from an existing read-only diff/code inspection path.
    /// T094 stores only a presentation copy and never executes or writes them.
    pub(crate) diff_bytes: Option<&'a [u8]>,
    pub(crate) verification_run_ids: &'a [&'a str],
    pub(crate) verification_running: bool,
    pub(crate) agent_reported_done: bool,
    /// Existing review context only; T094 does not manufacture review authority.
    pub(crate) review: Option<&'a IndependentReviewContext>,
    /// Existing human-acceptance candidate identity only; this is presentation
    /// input and never creates or changes a canonical human decision.
    pub(crate) human_accepted_candidate: Option<&'a CandidateIdentity>,
}

#[allow(
    dead_code,
    reason = "Spec 007 T094 projection seam is exercised by focused tests before later UI composition"
)]
pub(crate) fn project_candidate_context(
    input: WorkbenchContextInput<'_>,
) -> Result<WorkbenchCandidateContext, String> {
    let candidate = ExactCandidateProjection::observe(input.repo, input.candidate_ref)?;
    let base_oid = input
        .repo
        .resolve_commit(input.base_ref)
        .map_err(|error| format!("T094 base observation failed: {error}"))?;

    let diff = input
        .diff_bytes
        .map(|bytes| ReadOnlyDiffProjection::from_existing_read(base_oid, &candidate, bytes));

    let mut applicable_evidence_count = 0usize;
    let mut stale_evidence_count = 0usize;
    for run_id in input.verification_run_ids {
        let evidence = VerificationEvidenceReference::from_store(input.store, run_id)?;
        match evidence.applicability(&candidate.binding) {
            CandidateBindingStatus::Current => {
                applicable_evidence_count = applicable_evidence_count.saturating_add(1);
            }
            CandidateBindingStatus::Stale => {
                stale_evidence_count = stale_evidence_count.saturating_add(1);
            }
        }
    }

    let verification_state = if applicable_evidence_count > 0 {
        VerificationProjectionState::VerifiedForExactCandidate
    } else if input.verification_running {
        VerificationProjectionState::Running
    } else if stale_evidence_count > 0 {
        VerificationProjectionState::Stale
    } else {
        VerificationProjectionState::NotRun
    };

    let review = match input.review {
        None => ReviewProjectionState::NotAvailable,
        Some(review)
            if review.applicability(&candidate.binding) == CandidateBindingStatus::Current =>
        {
            ReviewProjectionState::ApplicableToExactCandidate
        }
        Some(_) => ReviewProjectionState::Stale,
    };

    let human_acceptance = match input.human_accepted_candidate {
        None => HumanAcceptanceProjectionState::NotAccepted,
        Some(accepted) if accepted == &candidate.binding => {
            HumanAcceptanceProjectionState::AcceptedForExactCandidate
        }
        Some(_) => HumanAcceptanceProjectionState::Stale,
    };

    Ok(WorkbenchCandidateContext {
        candidate,
        diff,
        agent_progress: if input.agent_reported_done {
            AgentProgressProjection::AgentReportedDone
        } else {
            AgentProgressProjection::NotReported
        },
        verification: VerificationProjection {
            state: verification_state,
            applicable_evidence_count,
            stale_evidence_count,
        },
        review,
        human_acceptance,
    })
}

#[cfg(test)]
#[path = "t094_workbench_context_tests.rs"]
mod t094_workbench_context_tests;
