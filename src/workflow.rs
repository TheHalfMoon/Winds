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
