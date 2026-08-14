use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckStatus {
    Pass,
    Fail,
    Timeout,
    Error,
    NotRun,
}

impl CheckStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Timeout => "TIMEOUT",
            Self::Error => "ERROR",
            Self::NotRun => "NOT_RUN",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobEvidence {
    pub relative_path: String,
    pub sha256: String,
    pub captured_bytes: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckEvidence {
    pub authority: &'static str,
    pub command: String,
    pub status: CheckStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub stdout: BlobEvidence,
    pub stderr: BlobEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceReport {
    pub schema_version: u32,
    pub run_id: String,
    pub authority: &'static str,
    pub repo_path: String,
    pub base_oid: String,
    pub candidate_ref: String,
    pub candidate_oid: String,
    pub candidate_tree: String,
    pub run_branch: String,
    pub worktree_path: String,
    pub check: CheckEvidence,
    pub eligibility: Eligibility,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StoredRun {
    pub run_id: String,
    pub repo_path: String,
    pub base_oid: String,
    pub candidate_ref: String,
    pub candidate_oid: String,
    pub candidate_tree: String,
    pub run_branch: String,
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
