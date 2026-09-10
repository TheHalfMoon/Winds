use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowError(String);

impl WorkflowError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for WorkflowError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for WorkflowError {}

pub(crate) type WorkflowResult<T> = std::result::Result<T, WorkflowError>;

fn required_identity(value: &str, field: &str) -> WorkflowResult<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(WorkflowError::new(format!("{field} must not be empty")));
    }
    Ok(normalized.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WorkflowRunIdentity {
    pub workflow_run_id: String,
    pub workspace_id: String,
    pub workstream_id: String,
}

impl WorkflowRunIdentity {
    pub(crate) fn new(
        workflow_run_id: &str,
        workspace_id: &str,
        workstream_id: &str,
    ) -> WorkflowResult<Self> {
        Ok(Self {
            workflow_run_id: required_identity(workflow_run_id, "workflow run id")?,
            workspace_id: required_identity(workspace_id, "workspace id")?,
            workstream_id: required_identity(workstream_id, "workstream id")?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StageRunIdentity {
    pub stage_run_id: String,
    pub workflow_run_id: String,
    pub stage_key: String,
    pub attempt_ordinal: u32,
    pub predecessor_stage_run_id: Option<String>,
}

impl StageRunIdentity {
    pub(crate) fn new(
        stage_run_id: &str,
        workflow_run_id: &str,
        stage_key: &str,
        attempt_ordinal: u32,
        predecessor_stage_run_id: Option<&str>,
    ) -> WorkflowResult<Self> {
        if attempt_ordinal == 0 {
            return Err(WorkflowError::new(
                "stage attempt ordinal must be greater than zero",
            ));
        }
        let predecessor_stage_run_id = predecessor_stage_run_id
            .map(|value| required_identity(value, "predecessor stage run id"))
            .transpose()?;
        Ok(Self {
            stage_run_id: required_identity(stage_run_id, "stage run id")?,
            workflow_run_id: required_identity(workflow_run_id, "workflow run id")?,
            stage_key: required_identity(stage_key, "stage key")?,
            attempt_ordinal,
            predecessor_stage_run_id,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageAttemptRelation {
    Retry,
    Reconstruction,
    Reassignment,
    Recovery,
}

impl StageAttemptRelation {
    pub(crate) fn as_db_str(self) -> &'static str {
        match self {
            Self::Retry => "RETRY_OF",
            Self::Reconstruction => "RECONSTRUCTION_OF",
            Self::Reassignment => "REASSIGNMENT_OF",
            Self::Recovery => "RECOVERY_OF",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "RETRY_OF" => Some(Self::Retry),
            "RECONSTRUCTION_OF" => Some(Self::Reconstruction),
            "REASSIGNMENT_OF" => Some(Self::Reassignment),
            "RECOVERY_OF" => Some(Self::Recovery),
            _ => None,
        }
    }
}

pub(crate) fn validate_successor_attempt(
    predecessor: &StageRunIdentity,
    successor: &StageRunIdentity,
    _relation: StageAttemptRelation,
) -> WorkflowResult<()> {
    if successor.stage_run_id == predecessor.stage_run_id {
        return Err(WorkflowError::new(
            "material new work requires a distinct stage run id",
        ));
    }
    if successor.workflow_run_id != predecessor.workflow_run_id {
        return Err(WorkflowError::new(
            "successor attempt must remain in the same workflow",
        ));
    }
    if successor.stage_key != predecessor.stage_key {
        return Err(WorkflowError::new(
            "successor attempt must remain in the same logical stage",
        ));
    }
    let expected_ordinal = predecessor
        .attempt_ordinal
        .checked_add(1)
        .ok_or_else(|| WorkflowError::new("stage attempt ordinal exhausted"))?;
    if successor.attempt_ordinal != expected_ordinal {
        return Err(WorkflowError::new(
            "successor attempt ordinal must increment by exactly one",
        ));
    }
    if successor.predecessor_stage_run_id.as_deref() != Some(predecessor.stage_run_id.as_str()) {
        return Err(WorkflowError::new(
            "successor attempt must bind the exact predecessor stage run",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StageLifecycleState {
    Prepared,
    Active,
    WaitingApproval,
    WaitingExternal,
    Blocked,
    Failed,
    Stale,
    Cancelled,
    Completed,
    RecoveryRequired,
}

impl StageLifecycleState {
    pub(crate) const ALL: [Self; 10] = [
        Self::Prepared,
        Self::Active,
        Self::WaitingApproval,
        Self::WaitingExternal,
        Self::Blocked,
        Self::Failed,
        Self::Stale,
        Self::Cancelled,
        Self::Completed,
        Self::RecoveryRequired,
    ];

    pub(crate) fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Failed | Self::Stale | Self::Cancelled | Self::Completed | Self::RecoveryRequired
        )
    }

    pub(crate) fn as_db_str(self) -> &'static str {
        match self {
            Self::Prepared => "PREPARED",
            Self::Active => "ACTIVE",
            Self::WaitingApproval => "WAITING_APPROVAL",
            Self::WaitingExternal => "WAITING_EXTERNAL",
            Self::Blocked => "BLOCKED",
            Self::Failed => "FAILED",
            Self::Stale => "STALE",
            Self::Cancelled => "CANCELLED",
            Self::Completed => "COMPLETED",
            Self::RecoveryRequired => "RECOVERY_REQUIRED",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|state| state.as_db_str() == value)
    }
}

pub(crate) fn is_legal_stage_transition(
    current: StageLifecycleState,
    requested: StageLifecycleState,
) -> bool {
    use StageLifecycleState::{
        Active, Blocked, Cancelled, Completed, Failed, Prepared, RecoveryRequired, Stale,
        WaitingApproval, WaitingExternal,
    };
    matches!(
        (current, requested),
        (Prepared, Active | Stale | Cancelled | RecoveryRequired)
            | (
                Active,
                WaitingApproval
                    | WaitingExternal
                    | Blocked
                    | Failed
                    | Stale
                    | Cancelled
                    | Completed
                    | RecoveryRequired
            )
            | (
                WaitingApproval,
                Active | Blocked | Failed | Stale | Cancelled | RecoveryRequired
            )
            | (
                WaitingExternal,
                Active | Blocked | Failed | Stale | Cancelled | RecoveryRequired
            )
            | (
                Blocked,
                Active | Failed | Stale | Cancelled | RecoveryRequired
            )
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TruthSource {
    AgentReported,
    WindsObserved,
    HumanDecided,
}

impl TruthSource {
    pub(crate) fn as_db_str(self) -> &'static str {
        match self {
            Self::AgentReported => "AGENT_REPORTED",
            Self::WindsObserved => "WINDS_OBSERVED",
            Self::HumanDecided => "HUMAN_DECIDED",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "AGENT_REPORTED" => Some(Self::AgentReported),
            "WINDS_OBSERVED" => Some(Self::WindsObserved),
            "HUMAN_DECIDED" => Some(Self::HumanDecided),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StageTransitionAuthority {
    None,
    WindsPolicy,
    HumanDecision,
}

impl StageTransitionAuthority {
    fn can_authorize_terminal_resolution(self) -> bool {
        matches!(self, Self::WindsPolicy | Self::HumanDecision)
    }

    pub(crate) fn as_db_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::WindsPolicy => "WINDS_POLICY",
            Self::HumanDecision => "HUMAN_DECISION",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "NONE" => Some(Self::None),
            "WINDS_POLICY" => Some(Self::WindsPolicy),
            "HUMAN_DECISION" => Some(Self::HumanDecision),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StageTransitionRequest {
    pub operation_id: String,
    pub expected_current: StageLifecycleState,
    pub requested: StageLifecycleState,
    pub source: TruthSource,
    pub authority: StageTransitionAuthority,
}

impl StageTransitionRequest {
    pub(crate) fn new(
        operation_id: &str,
        expected_current: StageLifecycleState,
        requested: StageLifecycleState,
        source: TruthSource,
        authority: StageTransitionAuthority,
    ) -> WorkflowResult<Self> {
        Ok(Self {
            operation_id: required_identity(operation_id, "stage transition operation id")?,
            expected_current,
            requested,
            source,
            authority,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppliedStageTransition {
    pub operation_id: String,
    pub from: StageLifecycleState,
    pub to: StageLifecycleState,
    pub source: TruthSource,
    pub authority: StageTransitionAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StageTransitionOutcome {
    Apply(AppliedStageTransition),
    IdempotentNoChange,
}

pub(crate) fn evaluate_stage_transition(
    current: StageLifecycleState,
    previous: Option<&AppliedStageTransition>,
    request: &StageTransitionRequest,
) -> WorkflowResult<StageTransitionOutcome> {
    if request.expected_current != current {
        if let Some(previous) = previous
            && previous.operation_id == request.operation_id
            && previous.from == request.expected_current
            && previous.to == request.requested
            && previous.source == request.source
            && previous.authority == request.authority
            && previous.to == current
        {
            return Ok(StageTransitionOutcome::IdempotentNoChange);
        }
        return Err(WorkflowError::new(
            "stage transition expected-current precondition is stale",
        ));
    }

    if request.requested == current {
        return Err(WorkflowError::new(
            "same-state transition is not idempotent without an exact prior operation",
        ));
    }
    if !is_legal_stage_transition(current, request.requested) {
        return Err(WorkflowError::new("illegal stage lifecycle transition"));
    }
    if matches!(
        request.requested,
        StageLifecycleState::Cancelled | StageLifecycleState::Completed
    ) {
        if request.source == TruthSource::AgentReported {
            return Err(WorkflowError::new(
                "agent-reported input cannot authorize stage cancellation or completion",
            ));
        }
        if !request.authority.can_authorize_terminal_resolution() {
            return Err(WorkflowError::new(
                "stage cancellation or completion requires explicit canonical authority",
            ));
        }
    }

    Ok(StageTransitionOutcome::Apply(AppliedStageTransition {
        operation_id: request.operation_id.clone(),
        from: current,
        to: request.requested,
        source: request.source,
        authority: request.authority,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LatestStageTruth {
    pub state: StageLifecycleState,
    pub source: TruthSource,
    pub authority: StageTransitionAuthority,
}

pub(crate) fn workflow_completion_eligible(latest_stages: &[LatestStageTruth]) -> bool {
    !latest_stages.is_empty()
        && latest_stages.iter().all(|stage| match stage.state {
            StageLifecycleState::Completed | StageLifecycleState::Cancelled => {
                stage.source != TruthSource::AgentReported
                    && stage.authority.can_authorize_terminal_resolution()
            }
            StageLifecycleState::Prepared
            | StageLifecycleState::Active
            | StageLifecycleState::WaitingApproval
            | StageLifecycleState::WaitingExternal
            | StageLifecycleState::Blocked
            | StageLifecycleState::Failed
            | StageLifecycleState::Stale
            | StageLifecycleState::RecoveryRequired => false,
        })
}
pub(crate) const MAX_CONSECUTIVE_NO_PROGRESS_RETRIES: u8 = 2;
pub(crate) const RETRY_OUTCOME_FAILURE_RECORDED: &str = "FAILURE_RECORDED";
pub(crate) const RETRY_OUTCOME_NO_PROGRESS: &str = "NO_PROGRESS_RETRY";
pub(crate) const RETRY_OUTCOME_BUDGET_EXHAUSTED: &str = "RETRY_BUDGET_EXHAUSTED";
pub(crate) const RETRY_OUTCOME_AMBIGUOUS_EFFECT: &str = "AMBIGUOUS_NON_IDEMPOTENT_EFFECT";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SideEffectTruth {
    SafeOrIdempotent,
    AmbiguousNonIdempotent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RetryFailureObservation {
    pub(crate) failure_class: String,
    pub(crate) checkpoint_identity: Option<String>,
    pub(crate) material_progress_basis: String,
    pub(crate) side_effect_truth: SideEffectTruth,
}

impl RetryFailureObservation {
    pub(crate) fn new(
        failure_class: &str,
        checkpoint_identity: Option<&str>,
        material_progress_basis: &str,
        side_effect_truth: SideEffectTruth,
    ) -> WorkflowResult<Self> {
        let failure_class = normalize_failure_class(failure_class)?;
        let checkpoint_identity = checkpoint_identity
            .map(|value| required_retry_reference(value, "retry checkpoint identity", 512))
            .transpose()?;
        let material_progress_basis = required_retry_reference(
            material_progress_basis,
            "retry material-progress basis",
            2048,
        )?;
        Ok(Self {
            failure_class,
            checkpoint_identity,
            material_progress_basis,
            side_effect_truth,
        })
    }
}

fn normalize_failure_class(value: &str) -> WorkflowResult<String> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty()
        || normalized.len() > 128
        || !normalized
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':' | '/'))
    {
        return Err(WorkflowError::new(
            "retry failure class must be a bounded canonical token",
        ));
    }
    Ok(normalized)
}

fn required_retry_reference(value: &str, field: &str, max_bytes: usize) -> WorkflowResult<String> {
    let normalized = value.trim();
    if normalized.is_empty()
        || normalized.len() > max_bytes
        || normalized.chars().any(char::is_control)
    {
        return Err(WorkflowError::new(format!(
            "{field} must be non-empty bounded printable canonical metadata"
        )));
    }
    Ok(normalized.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetryProgressComparison {
    MaterialProgress,
    NoProgress,
}

pub(crate) fn compare_retry_progress(
    previous: &RetryFailureObservation,
    current: &RetryFailureObservation,
) -> RetryProgressComparison {
    if previous.failure_class != current.failure_class {
        return RetryProgressComparison::MaterialProgress;
    }
    if previous.checkpoint_identity != current.checkpoint_identity
        && current.checkpoint_identity.is_some()
        && previous.material_progress_basis != current.material_progress_basis
    {
        return RetryProgressComparison::MaterialProgress;
    }
    RetryProgressComparison::NoProgress
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RetryFailureResolution {
    pub(crate) terminal_state: StageLifecycleState,
    pub(crate) comparison: Option<RetryProgressComparison>,
    pub(crate) consecutive_no_progress_retries: u8,
    pub(crate) outcome_reason: &'static str,
}

pub(crate) fn evaluate_retry_failure(
    is_retry_attempt: bool,
    predecessor: Option<(&RetryFailureObservation, &str)>,
    current: &RetryFailureObservation,
) -> WorkflowResult<RetryFailureResolution> {
    if current.side_effect_truth == SideEffectTruth::AmbiguousNonIdempotent {
        return Ok(RetryFailureResolution {
            terminal_state: StageLifecycleState::RecoveryRequired,
            comparison: None,
            consecutive_no_progress_retries: 0,
            outcome_reason: RETRY_OUTCOME_AMBIGUOUS_EFFECT,
        });
    }
    if !is_retry_attempt {
        return Ok(RetryFailureResolution {
            terminal_state: StageLifecycleState::Failed,
            comparison: None,
            consecutive_no_progress_retries: 0,
            outcome_reason: RETRY_OUTCOME_FAILURE_RECORDED,
        });
    }
    let (previous, previous_outcome) = predecessor.ok_or_else(|| {
        WorkflowError::new("retry attempt requires normalized predecessor failure truth")
    })?;
    if previous_outcome == RETRY_OUTCOME_BUDGET_EXHAUSTED {
        return Err(WorkflowError::new(
            "retry budget was already exhausted for this no-progress lineage",
        ));
    }
    let comparison = compare_retry_progress(previous, current);
    if comparison == RetryProgressComparison::MaterialProgress {
        return Ok(RetryFailureResolution {
            terminal_state: StageLifecycleState::Failed,
            comparison: Some(comparison),
            consecutive_no_progress_retries: 0,
            outcome_reason: RETRY_OUTCOME_FAILURE_RECORDED,
        });
    }
    let prior_no_progress = if previous_outcome == RETRY_OUTCOME_NO_PROGRESS {
        1
    } else {
        0
    };
    let consecutive = prior_no_progress + 1;
    let outcome_reason = if consecutive >= MAX_CONSECUTIVE_NO_PROGRESS_RETRIES {
        RETRY_OUTCOME_BUDGET_EXHAUSTED
    } else {
        RETRY_OUTCOME_NO_PROGRESS
    };
    Ok(RetryFailureResolution {
        terminal_state: StageLifecycleState::Failed,
        comparison: Some(comparison),
        consecutive_no_progress_retries: consecutive,
        outcome_reason,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ArtifactBaselineKind {
    ExactGitCandidate,
    WindsVerificationEvidence,
    PriorStageOutput,
    CanonicalDecision,
    BoundedBlobArtifact,
}

impl ArtifactBaselineKind {
    pub(crate) fn as_db_str(self) -> &'static str {
        match self {
            Self::ExactGitCandidate => "EXACT_GIT_CANDIDATE",
            Self::WindsVerificationEvidence => "WINDS_VERIFICATION_EVIDENCE",
            Self::PriorStageOutput => "PRIOR_STAGE_OUTPUT",
            Self::CanonicalDecision => "CANONICAL_DECISION",
            Self::BoundedBlobArtifact => "BOUNDED_BLOB_ARTIFACT",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "EXACT_GIT_CANDIDATE" => Some(Self::ExactGitCandidate),
            "WINDS_VERIFICATION_EVIDENCE" => Some(Self::WindsVerificationEvidence),
            "PRIOR_STAGE_OUTPUT" => Some(Self::PriorStageOutput),
            "CANONICAL_DECISION" => Some(Self::CanonicalDecision),
            "BOUNDED_BLOB_ARTIFACT" => Some(Self::BoundedBlobArtifact),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CandidateBaselineIdentity {
    pub(crate) oid: String,
    pub(crate) tree: String,
}

impl CandidateBaselineIdentity {
    pub(crate) fn new(oid: &str, tree: &str) -> WorkflowResult<Self> {
        Ok(Self {
            oid: required_lower_hex_git_oid(oid, "baseline candidate OID")?,
            tree: required_lower_hex_git_oid(tree, "baseline candidate tree")?,
        })
    }
}

fn required_lower_hex_git_oid(value: &str, field: &str) -> WorkflowResult<String> {
    if !matches!(value.len(), 40 | 64)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WorkflowError::new(format!(
            "{field} must be a lowercase 40- or 64-hex Git object id"
        )));
    }
    Ok(value.to_owned())
}

fn required_lower_hex_sha256_reference(value: &str) -> WorkflowResult<String> {
    let digest = value
        .strip_prefix("sha256:")
        .ok_or_else(|| WorkflowError::new("bounded blob baseline must use a sha256: reference"))?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WorkflowError::new(
            "bounded blob baseline sha256 reference must contain 64 lowercase hex characters",
        ));
    }
    Ok(value.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactBaselineIdentity {
    pub(crate) baseline_id: String,
    pub(crate) stage_run_id: String,
    pub(crate) kind: ArtifactBaselineKind,
    pub(crate) stable_reference: String,
    pub(crate) candidate: Option<CandidateBaselineIdentity>,
}

impl ArtifactBaselineIdentity {
    pub(crate) fn new(
        baseline_id: &str,
        stage_run_id: &str,
        kind: ArtifactBaselineKind,
        stable_reference: &str,
        candidate: Option<CandidateBaselineIdentity>,
    ) -> WorkflowResult<Self> {
        let stable_reference = match kind {
            ArtifactBaselineKind::BoundedBlobArtifact => {
                required_lower_hex_sha256_reference(stable_reference)?
            }
            _ => required_identity(stable_reference, "artifact baseline stable reference")?,
        };
        if matches!(
            kind,
            ArtifactBaselineKind::ExactGitCandidate
                | ArtifactBaselineKind::WindsVerificationEvidence
        ) && candidate.is_none()
        {
            return Err(WorkflowError::new(
                "candidate-bound artifact baseline requires exact candidate OID and tree",
            ));
        }
        Ok(Self {
            baseline_id: required_identity(baseline_id, "artifact baseline id")?,
            stage_run_id: required_identity(stage_run_id, "artifact baseline stage run id")?,
            kind,
            stable_reference,
            candidate,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactBaselineRequirement {
    pub(crate) stage_run_id: String,
    pub(crate) kind: ArtifactBaselineKind,
    pub(crate) stable_reference: String,
    pub(crate) candidate: Option<CandidateBaselineIdentity>,
}

impl ArtifactBaselineRequirement {
    pub(crate) fn new(
        stage_run_id: &str,
        kind: ArtifactBaselineKind,
        stable_reference: &str,
        candidate: Option<CandidateBaselineIdentity>,
    ) -> WorkflowResult<Self> {
        let identity = ArtifactBaselineIdentity::new(
            "requirement",
            stage_run_id,
            kind,
            stable_reference,
            candidate,
        )?;
        Ok(Self {
            stage_run_id: identity.stage_run_id,
            kind: identity.kind,
            stable_reference: identity.stable_reference,
            candidate: identity.candidate,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BaselineFreshness {
    Applicable,
    Stale,
    Missing,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BaselineEvaluation {
    pub(crate) requirement: ArtifactBaselineRequirement,
    pub(crate) freshness: BaselineFreshness,
    pub(crate) matching_baseline_ids: Vec<String>,
}

pub(crate) fn evaluate_artifact_baseline_requirement(
    requirement: &ArtifactBaselineRequirement,
    observed: &[ArtifactBaselineIdentity],
) -> BaselineEvaluation {
    let relevant = observed
        .iter()
        .filter(|baseline| {
            baseline.stage_run_id == requirement.stage_run_id
                && baseline.kind == requirement.kind
                && baseline.stable_reference == requirement.stable_reference
        })
        .collect::<Vec<_>>();
    let mut matching_baseline_ids = relevant
        .iter()
        .map(|baseline| baseline.baseline_id.clone())
        .collect::<Vec<_>>();
    matching_baseline_ids.sort();
    let freshness = match relevant.as_slice() {
        [] => BaselineFreshness::Missing,
        [baseline] if baseline.candidate == requirement.candidate => BaselineFreshness::Applicable,
        [_] => BaselineFreshness::Stale,
        _ => BaselineFreshness::Ambiguous,
    };
    BaselineEvaluation {
        requirement: requirement.clone(),
        freshness,
        matching_baseline_ids,
    }
}

pub(crate) const RECONSTRUCTION_REPORT_SCHEMA_VERSION: u32 = 1;
pub(crate) const MAX_RECONSTRUCTION_SOURCE_REFERENCE_BYTES: usize = 1024;
pub(crate) const MAX_RECONSTRUCTION_REPORT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkflowContinuationClass {
    Resumed,
    Reconstructed,
    OwnershipLost,
    Unavailable,
    Unproven,
}

impl WorkflowContinuationClass {
    pub(crate) fn as_db_str(self) -> &'static str {
        match self {
            Self::Resumed => "RESUMED",
            Self::Reconstructed => "RECONSTRUCTED",
            Self::OwnershipLost => "OWNERSHIP_LOST",
            Self::Unavailable => "UNAVAILABLE",
            Self::Unproven => "UNPROVEN",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "RESUMED" => Some(Self::Resumed),
            "RECONSTRUCTED" => Some(Self::Reconstructed),
            "OWNERSHIP_LOST" => Some(Self::OwnershipLost),
            "UNAVAILABLE" => Some(Self::Unavailable),
            "UNPROVEN" => Some(Self::Unproven),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ReconstructionCategory {
    CanonicalWorkContext,
    ObjectiveConstraints,
    Decisions,
    CandidateEvidence,
    PriorStageOutputs,
    RuntimeNativeContext,
    ProviderPrivateState,
}

impl ReconstructionCategory {
    pub(crate) const ALL: [Self; 7] = [
        Self::CanonicalWorkContext,
        Self::ObjectiveConstraints,
        Self::Decisions,
        Self::CandidateEvidence,
        Self::PriorStageOutputs,
        Self::RuntimeNativeContext,
        Self::ProviderPrivateState,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ReconstructionTransferState {
    PreservedReference,
    Reconstructed,
    Derived,
    Omitted,
    Unavailable,
    NoLongerTransferable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ReconstructionSourceClass {
    WindsObserved,
    HumanDecided,
    StoredCanonicalReference,
    DerivedReconstruction,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ReconstructionContentState {
    Full,
    Redacted,
    Omitted,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReconstructionItem {
    pub(crate) category: ReconstructionCategory,
    pub(crate) source_class: ReconstructionSourceClass,
    pub(crate) source_reference: String,
    pub(crate) transfer_state: ReconstructionTransferState,
    pub(crate) content_state: ReconstructionContentState,
}

impl ReconstructionItem {
    pub(crate) fn new(
        category: ReconstructionCategory,
        source_class: ReconstructionSourceClass,
        source_reference: &str,
        transfer_state: ReconstructionTransferState,
        content_state: ReconstructionContentState,
    ) -> WorkflowResult<Self> {
        let source_reference = required_bounded_reference(
            source_reference,
            "reconstruction source reference",
            MAX_RECONSTRUCTION_SOURCE_REFERENCE_BYTES,
        )?;
        let item = Self {
            category,
            source_class,
            source_reference,
            transfer_state,
            content_state,
        };
        validate_reconstruction_item(&item)?;
        Ok(item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReconstructionReport {
    pub(crate) schema_version: u32,
    pub(crate) binding_id: String,
    pub(crate) stage_run_id: String,
    pub(crate) items: Vec<ReconstructionItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReconstructionPreview {
    pub(crate) report: ReconstructionReport,
    pub(crate) canonical_json: String,
}

pub(crate) fn build_reconstruction_preview(
    binding_id: &str,
    stage_run_id: &str,
    items: &[ReconstructionItem],
) -> WorkflowResult<ReconstructionPreview> {
    let mut report = ReconstructionReport {
        schema_version: RECONSTRUCTION_REPORT_SCHEMA_VERSION,
        binding_id: required_identity(binding_id, "reconstruction binding id")?,
        stage_run_id: required_identity(stage_run_id, "reconstruction stage run id")?,
        items: items.to_vec(),
    };
    report.items.sort_by_key(|item| item.category);
    validate_reconstruction_report(&report)?;
    let canonical_json = serde_json::to_string(&report).map_err(|error| {
        WorkflowError::new(format!(
            "reconstruction report serialization failed: {error}"
        ))
    })?;
    if canonical_json.len() > MAX_RECONSTRUCTION_REPORT_BYTES {
        return Err(WorkflowError::new(
            "reconstruction report exceeds the bounded persistence limit",
        ));
    }
    Ok(ReconstructionPreview {
        report,
        canonical_json,
    })
}

pub(crate) fn parse_reconstruction_report_json(
    canonical_json: &str,
) -> WorkflowResult<ReconstructionReport> {
    if canonical_json.is_empty() || canonical_json.len() > MAX_RECONSTRUCTION_REPORT_BYTES {
        return Err(WorkflowError::new(
            "reconstruction report JSON is empty or exceeds the bounded persistence limit",
        ));
    }
    let report: ReconstructionReport = serde_json::from_str(canonical_json).map_err(|error| {
        WorkflowError::new(format!("invalid reconstruction report JSON: {error}"))
    })?;
    validate_reconstruction_report(&report)?;
    let normalized =
        build_reconstruction_preview(&report.binding_id, &report.stage_run_id, &report.items)?;
    if normalized.canonical_json != canonical_json {
        return Err(WorkflowError::new(
            "reconstruction report JSON is not in canonical deterministic form",
        ));
    }
    Ok(report)
}

fn validate_reconstruction_report(report: &ReconstructionReport) -> WorkflowResult<()> {
    if report.schema_version != RECONSTRUCTION_REPORT_SCHEMA_VERSION {
        return Err(WorkflowError::new(
            "unsupported reconstruction report schema version",
        ));
    }
    required_identity(&report.binding_id, "reconstruction binding id")?;
    required_identity(&report.stage_run_id, "reconstruction stage run id")?;
    if report.items.len() != ReconstructionCategory::ALL.len() {
        return Err(WorkflowError::new(
            "reconstruction report must contain every first-slice category exactly once",
        ));
    }
    let mut categories = BTreeSet::new();
    for item in &report.items {
        validate_reconstruction_item(item)?;
        if !categories.insert(item.category) {
            return Err(WorkflowError::new(
                "reconstruction report contains a duplicate material-context category",
            ));
        }
    }
    if categories != ReconstructionCategory::ALL.into_iter().collect() {
        return Err(WorkflowError::new(
            "reconstruction report is missing a required material-context category",
        ));
    }
    Ok(())
}

fn validate_reconstruction_item(item: &ReconstructionItem) -> WorkflowResult<()> {
    required_bounded_reference(
        &item.source_reference,
        "reconstruction source reference",
        MAX_RECONSTRUCTION_SOURCE_REFERENCE_BYTES,
    )?;
    match item.transfer_state {
        ReconstructionTransferState::Omitted
            if item.content_state != ReconstructionContentState::Omitted =>
        {
            return Err(WorkflowError::new(
                "omitted reconstruction context must retain an OMITTED completeness marker",
            ));
        }
        ReconstructionTransferState::Unavailable
        | ReconstructionTransferState::NoLongerTransferable
            if item.content_state != ReconstructionContentState::Unavailable =>
        {
            return Err(WorkflowError::new(
                "unavailable reconstruction context must retain an UNAVAILABLE completeness marker",
            ));
        }
        ReconstructionTransferState::PreservedReference
        | ReconstructionTransferState::Reconstructed
        | ReconstructionTransferState::Derived
            if matches!(
                item.content_state,
                ReconstructionContentState::Omitted | ReconstructionContentState::Unavailable
            ) =>
        {
            return Err(WorkflowError::new(
                "present reconstruction context cannot be marked omitted or unavailable",
            ));
        }
        _ => {}
    }
    if item.category == ReconstructionCategory::ProviderPrivateState {
        let expected_reference = match item.transfer_state {
            ReconstructionTransferState::Unavailable => "provider-private-state:unavailable",
            ReconstructionTransferState::NoLongerTransferable => {
                "provider-private-state:no-longer-transferable"
            }
            _ => "",
        };
        if expected_reference.is_empty()
            || item.source_class != ReconstructionSourceClass::Unavailable
            || item.content_state != ReconstructionContentState::Unavailable
            || item.source_reference != expected_reference
        {
            return Err(WorkflowError::new(
                "provider-private state must use only bounded unavailable metadata and must never persist private payload",
            ));
        }
    }
    Ok(())
}

fn required_bounded_reference(
    value: &str,
    field: &str,
    max_bytes: usize,
) -> WorkflowResult<String> {
    let normalized = value.trim();
    if normalized.is_empty()
        || normalized.len() > max_bytes
        || normalized.chars().any(char::is_control)
    {
        return Err(WorkflowError::new(format!(
            "{field} must be non-empty bounded printable reference metadata"
        )));
    }
    Ok(normalized.to_owned())
}
