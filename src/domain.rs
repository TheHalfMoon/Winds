use serde::Serialize;

#[cfg(test)]
#[path = "t070_agentic_identity_tests.rs"]
mod t070_agentic_identity_tests;
#[cfg(test)]
#[path = "t083_agentic_candidate_evidence_tests.rs"]
mod t083_agentic_candidate_evidence_tests;

#[cfg_attr(
    not(unix),
    allow(
        dead_code,
        reason = "Spec 003 T051 keeps authoritative required-check execution Unix-only; Fail/Timeout remain cross-platform evidence values"
    )
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Pass,
    Fail,
    Timeout,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Eligibility {
    Eligible,
    Warning,
    Blocked,
}

impl Eligibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eligible => "ELIGIBLE",
            Self::Warning => "WARNING",
            Self::Blocked => "BLOCKED",
        }
    }
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionKind {
    Terminal,
    ShellCommand,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
impl ExecutionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Terminal => "TERMINAL",
            Self::ShellCommand => "SHELL_COMMAND",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "TERMINAL" => Some(Self::Terminal),
            "SHELL_COMMAND" => Some(Self::ShellCommand),
            _ => None,
        }
    }
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FactSource {
    CallerRequested,
    WindsObserved,
    ShellReported,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
impl FactSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CallerRequested => "CALLER_REQUESTED",
            Self::WindsObserved => "WINDS_OBSERVED",
            Self::ShellReported => "SHELL_REPORTED",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "CALLER_REQUESTED" => Some(Self::CallerRequested),
            "WINDS_OBSERVED" => Some(Self::WindsObserved),
            "SHELL_REPORTED" => Some(Self::ShellReported),
            _ => None,
        }
    }
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionStatus {
    Requested,
    Running,
    Exited,
    FailedToStart,
    Interrupted,
    OwnershipLost,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
impl ExecutionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "REQUESTED",
            Self::Running => "RUNNING",
            Self::Exited => "EXITED",
            Self::FailedToStart => "FAILED_TO_START",
            Self::Interrupted => "INTERRUPTED",
            Self::OwnershipLost => "OWNERSHIP_LOST",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "REQUESTED" => Some(Self::Requested),
            "RUNNING" => Some(Self::Running),
            "EXITED" => Some(Self::Exited),
            "FAILED_TO_START" => Some(Self::FailedToStart),
            "INTERRUPTED" => Some(Self::Interrupted),
            "OWNERSHIP_LOST" => Some(Self::OwnershipLost),
            _ => None,
        }
    }
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRecord {
    pub workspace_id: String,
    pub canonical_worktree_root: String,
    pub git_common_dir: String,
    pub created_unix_ms: i64,
    pub last_opened_unix_ms: i64,
}

#[allow(
    dead_code,
    reason = "Spec 006 T070 persistence substrate; product session semantics land in T071"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkstreamRecord {
    pub workstream_id: String,
    pub workspace_id: String,
    pub display_name: String,
    pub created_unix_ms: i64,
    pub updated_unix_ms: i64,
}

#[allow(
    dead_code,
    reason = "Spec 006 T070 persistence substrate; product session semantics land in T071"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindsSessionRecord {
    pub session_id: String,
    pub workstream_id: String,
    pub display_name: String,
    pub created_unix_ms: i64,
    pub updated_unix_ms: i64,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub workspace_id: String,
    pub kind: ExecutionKind,
    pub request_source: FactSource,
    pub execution_domain: String,
    pub status: ExecutionStatus,
    pub status_source: FactSource,
    pub requested_unix_ms: i64,
    pub started_unix_ms: Option<i64>,
    pub ended_unix_ms: Option<i64>,
    pub duration_ms: Option<u64>,
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEventRecord {
    pub event_id: i64,
    pub execution_id: String,
    pub kind: String,
    pub source: FactSource,
    pub created_unix_ms: i64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TerminalCloseReason {
    ProcessExited,
    FailedToStart,
    TerminatedByWinds,
    ClosedByWinds,
    StartPersistenceFailed,
    OwnershipLostProcessStateUnknown,
}

impl TerminalCloseReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProcessExited => "PROCESS_EXITED",
            Self::FailedToStart => "FAILED_TO_START",
            Self::TerminatedByWinds => "TERMINATED_BY_WINDS",
            Self::ClosedByWinds => "CLOSED_BY_WINDS",
            Self::StartPersistenceFailed => "START_PERSISTENCE_FAILED",
            Self::OwnershipLostProcessStateUnknown => "OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN",
        }
    }

    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "PROCESS_EXITED" => Some(Self::ProcessExited),
            "FAILED_TO_START" => Some(Self::FailedToStart),
            "TERMINATED_BY_WINDS" => Some(Self::TerminatedByWinds),
            "CLOSED_BY_WINDS" => Some(Self::ClosedByWinds),
            "START_PERSISTENCE_FAILED" => Some(Self::StartPersistenceFailed),
            "OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN" => Some(Self::OwnershipLostProcessStateUnknown),
            _ => None,
        }
    }
}

#[allow(
    dead_code,
    reason = "Spec 003 T044 persistence substrate; runtime callers land in later slices"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSessionRecord {
    pub execution_id: String,
    pub profile_id: String,
    pub shell_executable: String,
    pub shell_arguments: Vec<String>,
    pub requested_cwd: String,
    pub initial_cols: Option<u16>,
    pub initial_rows: Option<u16>,
    pub close_reason: Option<TerminalCloseReason>,
}

#[allow(
    dead_code,
    reason = "Spec 003 T054 command-record backend API; CLI/timeline caller lands in T057"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommandRecord {
    pub execution_id: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub command_source: FactSource,
    pub requested_cwd: String,
    pub cwd_source: FactSource,
    pub exit_code: Option<i32>,
    pub exit_source: Option<FactSource>,
    pub observed_end_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlobEvidence {
    pub relative_path: String,
    pub sha256: String,
    pub captured_bytes: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckEvidence {
    pub authority: &'static str,
    pub command: String,
    pub status: CheckStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout: BlobEvidence,
    pub stderr: BlobEvidence,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceReport {
    pub schema_version: u32,
    pub run_id: String,
    pub authority: &'static str,
    pub repo_path: String,
    pub base_oid: String,
    pub candidate_ref: String,
    pub candidate_oid: String,
    pub candidate_tree: String,
    pub worktree_path: String,
    pub check: CheckEvidence,
    pub eligibility: Eligibility,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StoredRun {
    pub run_id: String,
    pub repo_path: String,
    pub candidate_oid: String,
    pub candidate_tree: String,
    pub worktree_path: String,
    pub check_command: String,
    pub timeout_secs: u64,
    pub eligibility: Eligibility,
}

#[derive(Debug, Serialize)]
pub struct PromotionReport {
    pub run_id: String,
    pub authority: &'static str,
    pub branch: String,
    pub commit_oid: String,
    pub candidate_tree: String,
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 candidate identity seam precedes later product callers"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CandidateIdentity {
    oid: String,
    tree: String,
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 candidate identity seam precedes later product callers"
)]
impl CandidateIdentity {
    pub fn new(oid: &str, tree: &str) -> Result<Self, String> {
        Ok(Self {
            oid: normalize_git_object_id(oid, "candidate OID")?,
            tree: normalize_git_object_id(tree, "candidate tree")?,
        })
    }
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 candidate staleness seam precedes later product callers"
)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CandidateBindingStatus {
    Current,
    Stale,
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 verify-evidence reference seam precedes later product callers"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VerificationEvidenceReference {
    run_id: String,
    candidate: CandidateIdentity,
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 verify-evidence reference seam precedes later product callers"
)]
impl VerificationEvidenceReference {
    pub fn from_store(store: &crate::store::Store, run_id: &str) -> Result<Self, String> {
        let requested_run_id = normalize_nonempty(run_id, "verification run id")?;
        let run = store
            .load_run(&requested_run_id)
            .map_err(|error| format!("T083 could not load persisted winds verify run: {error}"))?;
        if run.eligibility != Eligibility::Eligible {
            return Err(
                "T083 accepts only persisted ELIGIBLE winds verify runs as verification evidence"
                    .to_owned(),
            );
        }
        let persisted_run_id = normalize_nonempty(&run.run_id, "persisted verification run id")?;
        if persisted_run_id != requested_run_id {
            return Err("T083 persisted verification run identity mismatch".to_owned());
        }
        Ok(Self {
            run_id: persisted_run_id,
            candidate: CandidateIdentity::new(&run.candidate_oid, &run.candidate_tree)?,
        })
    }

    pub fn applicability(&self, current: &CandidateIdentity) -> CandidateBindingStatus {
        candidate_binding_status(&self.candidate, current)
    }
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 independent-review context seam precedes later product callers"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IndependentReviewContext {
    pub base_oid: String,
    pub candidate: CandidateIdentity,
    pub diff_identity: String,
    pub acceptance_criteria: Vec<String>,
    pub canonical_constraints: Vec<String>,
    pub verification_evidence: Vec<VerificationEvidenceReference>,
    pub excluded_builder_persuasion_count: usize,
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 independent-review context seam precedes later product callers"
)]
pub struct IndependentReviewContextInput<'a> {
    pub base_oid: &'a str,
    pub candidate: CandidateIdentity,
    pub diff_identity: &'a str,
    pub acceptance_criteria: Vec<String>,
    pub canonical_constraints: Vec<String>,
    pub verification_evidence: Vec<VerificationEvidenceReference>,
    pub builder_persuasion: &'a [String],
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 independent-review context seam precedes later product callers"
)]
impl IndependentReviewContext {
    pub fn build(input: IndependentReviewContextInput<'_>) -> Result<Self, String> {
        let base_oid = normalize_git_object_id(input.base_oid, "review base OID")?;
        let diff_identity = normalize_nonempty(input.diff_identity, "review diff identity")?;
        let acceptance_criteria =
            normalize_review_items(input.acceptance_criteria, "acceptance criterion")?;
        let canonical_constraints =
            normalize_review_items(input.canonical_constraints, "canonical constraint")?;
        if input.verification_evidence.is_empty() {
            return Err("T083 review context requires winds verify evidence".to_owned());
        }

        for evidence in &input.verification_evidence {
            if evidence.applicability(&input.candidate) != CandidateBindingStatus::Current {
                return Err("T083 review context rejects verification evidence bound to a different candidate".to_owned());
            }
        }

        Ok(Self {
            base_oid,
            candidate: input.candidate,
            diff_identity,
            acceptance_criteria,
            canonical_constraints,
            verification_evidence: input.verification_evidence,
            excluded_builder_persuasion_count: input.builder_persuasion.len(),
        })
    }

    pub fn applicability(&self, current: &CandidateIdentity) -> CandidateBindingStatus {
        candidate_binding_status(&self.candidate, current)
    }
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 candidate staleness seam precedes later product callers"
)]
fn candidate_binding_status(
    bound: &CandidateIdentity,
    current: &CandidateIdentity,
) -> CandidateBindingStatus {
    if bound == current {
        CandidateBindingStatus::Current
    } else {
        CandidateBindingStatus::Stale
    }
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 normalization seam precedes later product callers"
)]
fn normalize_git_object_id(value: &str, label: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if !matches!(normalized.len(), 40 | 64)
        || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "{label} must be an exact 40- or 64-hex Git object id"
        ));
    }
    Ok(normalized)
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 normalization seam precedes later product callers"
)]
fn normalize_nonempty(value: &str, label: &str) -> Result<String, String> {
    if value.contains('\0') {
        return Err(format!("{label} must not contain NUL"));
    }
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    let normalized = normalized.trim().to_owned();
    if normalized.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    Ok(normalized)
}

#[allow(
    dead_code,
    reason = "Spec 006 T083 normalization seam precedes later product callers"
)]
fn normalize_review_items(values: Vec<String>, label: &str) -> Result<Vec<String>, String> {
    let mut normalized = values
        .into_iter()
        .map(|value| normalize_nonempty(&value, label))
        .collect::<Result<Vec<_>, _>>()?;
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(format!("T083 requires at least one {label}"));
    }
    Ok(normalized)
}
