use crate::store::{Result as StoreResult, Store};
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
#[cfg(unix)]
use std::ffi::CString;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "t073_runtime_binding_tests.rs"]
mod t073_runtime_binding_tests;

const HASH_BUFFER_BYTES: usize = 64 * 1024;
pub(crate) const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_VERSION_BYTES: usize = 256;

pub(crate) type RuntimeDiscoveryResult<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RuntimeKind {
    Codex,
    Claude,
}

impl RuntimeKind {
    /// Returns the stable persistence representation for the concrete runtime kind.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "CODEX",
            Self::Claude => "CLAUDE",
        }
    }

    /// Parses only the concrete runtime kinds currently authorized by Spec 006.
    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "CODEX" => Some(Self::Codex),
            "CLAUDE" => Some(Self::Claude),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RuntimeCapability {
    StructuredControl,
    NativeContinuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CapabilitySupport {
    Supported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DeclarationSource {
    Vendor,
    Catalog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenceSource {
    WindsLocallyObserved,
    VendorDeclared,
    CatalogDeclared,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclaredCapability {
    pub capability: RuntimeCapability,
    pub support: CapabilitySupport,
    pub source: DeclarationSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalCapabilityObservation {
    pub capability: RuntimeCapability,
    pub support: CapabilitySupport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeCapabilityEvidence {
    pub capability: RuntimeCapability,
    pub support: Option<CapabilitySupport>,
    pub source: EvidenceSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SafeVersionObservation {
    Observed(String),
    Unsupported(String),
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeVersionState {
    Observed,
    Unsupported,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeVersionEvidence {
    pub state: RuntimeVersionState,
    pub value: Option<String>,
    pub source: EvidenceSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthReadiness {
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthReadinessEvidence {
    pub readiness: AuthReadiness,
    pub source: EvidenceSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentExecutionObservation {
    NotPerformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeDiscoveryState {
    Unavailable,
    Present,
    UnsupportedVersion,
    VersionUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeExecutableIdentity {
    pub observed_path: PathBuf,
    pub canonical_path: PathBuf,
    pub byte_len: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeDiscovery {
    pub runtime: RuntimeKind,
    pub state: RuntimeDiscoveryState,
    pub executable: Option<RuntimeExecutableIdentity>,
    pub version: RuntimeVersionEvidence,
    pub capabilities: Vec<RuntimeCapabilityEvidence>,
    pub auth_readiness: AuthReadinessEvidence,
    pub agent_execution: AgentExecutionObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeIdentityRevalidation {
    Match,
    Changed,
    Unavailable,
}

/// Durable ownership truth for a runtime/native binding.
///
/// T073 deliberately has no persisted `LIVE` variant: live process ownership must be proven in
/// the current Winds lifetime by a later authorized runtime task. A durable native ID therefore
/// cannot recreate live ownership after restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeBindingOwnership {
    Unproven,
    OwnershipLost,
}

impl RuntimeBindingOwnership {
    /// Returns the stable persistence representation for durable ownership truth.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Unproven => "UNPROVEN",
            Self::OwnershipLost => "OWNERSHIP_LOST",
        }
    }

    /// Parses the bounded durable ownership vocabulary used by T073.
    pub(crate) fn from_db(value: &str) -> Option<Self> {
        match value {
            "UNPROVEN" => Some(Self::Unproven),
            "OWNERSHIP_LOST" => Some(Self::OwnershipLost),
            _ => None,
        }
    }
}

/// Exact persisted mapping from one canonical Winds session to one concrete runtime observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeSessionBinding {
    pub binding_id: String,
    pub session_id: String,
    pub runtime: RuntimeKind,
    pub executable: RuntimeExecutableIdentity,
    pub version: RuntimeVersionEvidence,
    pub native_session_id: Option<String>,
    pub ownership: RuntimeBindingOwnership,
    pub bound_unix_ms: i64,
    pub ownership_observed_unix_ms: Option<i64>,
}

/// Pre-execution continuity truth derived from durable mappings and a fresh safe discovery.
///
/// This intentionally has no `LIVE` or `RESUMED` state. T073 may identify an exact future resume
/// candidate, but only a later task that actually performs and proves resume may claim `RESUMED`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeResumeResolution {
    Unavailable,
    Stale,
    Candidate(RuntimeSessionBinding),
    Ambiguous(Vec<RuntimeSessionBinding>),
}

pub(crate) fn discover_runtime_from_safe_observations(
    runtime: RuntimeKind,
    executable_path: &Path,
    version: SafeVersionObservation,
    declarations: &[DeclaredCapability],
    local_observations: &[LocalCapabilityObservation],
) -> RuntimeDiscoveryResult<RuntimeDiscovery> {
    let executable = inspect_runtime_executable(executable_path)?;
    let capabilities = build_capability_evidence(declarations, local_observations)?;

    let Some(executable) = executable else {
        if !matches!(&version, SafeVersionObservation::Unavailable) {
            return Err("version evidence cannot be attached to an unavailable runtime".to_owned());
        }
        if !local_observations.is_empty() {
            return Err(
                "local capability evidence cannot be attached to an unavailable runtime".to_owned(),
            );
        }
        return Ok(RuntimeDiscovery {
            runtime,
            state: RuntimeDiscoveryState::Unavailable,
            executable: None,
            version: RuntimeVersionEvidence {
                state: RuntimeVersionState::Unavailable,
                value: None,
                source: EvidenceSource::Unavailable,
            },
            capabilities,
            auth_readiness: unknown_auth_readiness(),
            agent_execution: AgentExecutionObservation::NotPerformed,
        });
    };

    let version = build_version_evidence(version)?;
    let state = match version.state {
        RuntimeVersionState::Observed => RuntimeDiscoveryState::Present,
        RuntimeVersionState::Unsupported => RuntimeDiscoveryState::UnsupportedVersion,
        RuntimeVersionState::Unavailable => RuntimeDiscoveryState::VersionUnavailable,
    };

    Ok(RuntimeDiscovery {
        runtime,
        state,
        executable: Some(executable),
        version,
        capabilities,
        auth_readiness: unknown_auth_readiness(),
        agent_execution: AgentExecutionObservation::NotPerformed,
    })
}

pub(crate) fn revalidate_runtime_identity(
    expected: &RuntimeExecutableIdentity,
) -> RuntimeDiscoveryResult<RuntimeIdentityRevalidation> {
    match inspect_runtime_executable(&expected.observed_path)? {
        None => Ok(RuntimeIdentityRevalidation::Unavailable),
        Some(current) if current == *expected => Ok(RuntimeIdentityRevalidation::Match),
        Some(_) => Ok(RuntimeIdentityRevalidation::Changed),
    }
}

/// Returns true only when a persisted binding exactly matches a fresh supported runtime discovery.
///
/// Exact matching includes concrete runtime kind, observed/canonical executable paths, byte length,
/// SHA-256, version state/value, and provenance. Ownership/native IDs are intentionally excluded:
/// they do not prove executable/version applicability.
pub(crate) fn runtime_binding_matches_discovery(
    binding: &RuntimeSessionBinding,
    discovery: &RuntimeDiscovery,
) -> bool {
    discovery.runtime == binding.runtime
        && discovery.state == RuntimeDiscoveryState::Present
        && discovery.executable.as_ref() == Some(&binding.executable)
        && discovery.version == binding.version
}

#[derive(Debug)]
struct StoredRuntimeBindingRow {
    binding_id: String,
    session_id: String,
    runtime_kind: String,
    observed_executable_path: String,
    canonical_executable_path: String,
    executable_byte_len: i64,
    executable_sha256: String,
    runtime_version_state: String,
    runtime_version: String,
    runtime_version_source: String,
    native_session_id: Option<String>,
    ownership_state: String,
    bound_unix_ms: i64,
    ownership_observed_unix_ms: Option<i64>,
}

impl Store {
    /// Persists a fixture/runtime binding only from exact supported local discovery evidence.
    pub(crate) fn create_runtime_session_binding(
        &self,
        binding_id: &str,
        session_id: &str,
        discovery: &RuntimeDiscovery,
        native_session_id: Option<&str>,
        now_ms: i64,
    ) -> StoreResult<()> {
        validate_runtime_binding_text(binding_id, "runtime binding id")?;
        validate_runtime_binding_text(session_id, "runtime binding Winds session id")?;
        validate_runtime_binding_timestamp(now_ms, "runtime binding observation time")?;
        if let Some(native_session_id) = native_session_id {
            validate_runtime_binding_text(native_session_id, "native runtime session id")?;
        }

        let session = self.load_winds_session(session_id)?;
        if now_ms < session.created_unix_ms {
            return Err("runtime binding observation cannot precede Winds session creation".into());
        }
        if discovery.state != RuntimeDiscoveryState::Present {
            return Err("runtime binding requires a supported present runtime discovery".into());
        }
        let executable = discovery
            .executable
            .as_ref()
            .ok_or("present runtime discovery is missing executable identity")?;
        let version = &discovery.version;
        if version.state != RuntimeVersionState::Observed
            || version.source != EvidenceSource::WindsLocallyObserved
        {
            return Err("runtime binding requires locally observed exact version evidence".into());
        }
        let version_value = version
            .value
            .as_deref()
            .ok_or("observed runtime version is missing its value")?;
        let observed_path = runtime_binding_path_text(&executable.observed_path, "observed")?;
        let canonical_path = runtime_binding_path_text(&executable.canonical_path, "canonical")?;
        let executable_byte_len = i64::try_from(executable.byte_len)?;
        validate_runtime_binding_sha256(&executable.sha256)?;

        self.connection.execute(
            "INSERT INTO runtime_session_bindings(
                binding_id, session_id, runtime_kind,
                observed_executable_path, canonical_executable_path,
                executable_byte_len, executable_sha256,
                runtime_version_state, runtime_version, runtime_version_source,
                native_session_id, ownership_state, bound_unix_ms, ownership_observed_unix_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'OBSERVED', ?8,
                       'WINDS_LOCALLY_OBSERVED', ?9, ?10, ?11, NULL)",
            params![
                binding_id,
                session_id,
                discovery.runtime.as_str(),
                observed_path,
                canonical_path,
                executable_byte_len,
                executable.sha256,
                version_value,
                native_session_id,
                RuntimeBindingOwnership::Unproven.as_str(),
                now_ms,
            ],
        )?;
        Ok(())
    }

    /// Loads one exact durable runtime/native binding and rejects unknown persisted vocabulary.
    pub(crate) fn load_runtime_session_binding(
        &self,
        binding_id: &str,
    ) -> StoreResult<RuntimeSessionBinding> {
        validate_runtime_binding_text(binding_id, "runtime binding id")?;
        let row = self
            .connection
            .query_row(
                "SELECT binding_id, session_id, runtime_kind,
                        observed_executable_path, canonical_executable_path,
                        executable_byte_len, executable_sha256,
                        runtime_version_state, runtime_version, runtime_version_source,
                        native_session_id, ownership_state, bound_unix_ms,
                        ownership_observed_unix_ms
                 FROM runtime_session_bindings
                 WHERE binding_id = ?1",
                params![binding_id],
                |row| {
                    Ok(StoredRuntimeBindingRow {
                        binding_id: row.get(0)?,
                        session_id: row.get(1)?,
                        runtime_kind: row.get(2)?,
                        observed_executable_path: row.get(3)?,
                        canonical_executable_path: row.get(4)?,
                        executable_byte_len: row.get(5)?,
                        executable_sha256: row.get(6)?,
                        runtime_version_state: row.get(7)?,
                        runtime_version: row.get(8)?,
                        runtime_version_source: row.get(9)?,
                        native_session_id: row.get(10)?,
                        ownership_state: row.get(11)?,
                        bound_unix_ms: row.get(12)?,
                        ownership_observed_unix_ms: row.get(13)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| format!("unknown runtime session binding: {binding_id}"))?;
        runtime_binding_from_stored_row(row)
    }

    /// Lists deterministic binding history for one Winds session and one concrete runtime kind.
    pub(crate) fn list_runtime_session_bindings(
        &self,
        session_id: &str,
        runtime: RuntimeKind,
    ) -> StoreResult<Vec<RuntimeSessionBinding>> {
        self.load_winds_session(session_id)?;
        let binding_ids = {
            let mut statement = self.connection.prepare(
                "SELECT binding_id
                 FROM runtime_session_bindings
                 WHERE session_id = ?1 AND runtime_kind = ?2
                 ORDER BY bound_unix_ms, binding_id",
            )?;
            statement
                .query_map(params![session_id, runtime.as_str()], |row| {
                    row.get::<_, String>(0)
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        binding_ids
            .into_iter()
            .map(|binding_id| self.load_runtime_session_binding(&binding_id))
            .collect()
    }

    /// Records durable ownership loss without deleting the native ID or implying resume success.
    pub(crate) fn mark_runtime_binding_ownership_lost(
        &self,
        binding_id: &str,
        observed_unix_ms: i64,
    ) -> StoreResult<()> {
        validate_runtime_binding_timestamp(
            observed_unix_ms,
            "runtime binding ownership-loss observation time",
        )?;
        let binding = self.load_runtime_session_binding(binding_id)?;
        if observed_unix_ms < binding.bound_unix_ms {
            return Err("runtime ownership loss cannot precede binding observation".into());
        }
        if binding.ownership == RuntimeBindingOwnership::OwnershipLost {
            let previous = binding
                .ownership_observed_unix_ms
                .ok_or("ownership-lost binding is missing observation time")?;
            if observed_unix_ms < previous {
                return Err("runtime ownership-loss observations must be monotonic".into());
            }
            return Ok(());
        }
        let updated = self.connection.execute(
            "UPDATE runtime_session_bindings
             SET ownership_state = ?2, ownership_observed_unix_ms = ?3
             WHERE binding_id = ?1 AND ownership_state = ?4",
            params![
                binding_id,
                RuntimeBindingOwnership::OwnershipLost.as_str(),
                observed_unix_ms,
                RuntimeBindingOwnership::Unproven.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("runtime ownership-loss transition lost its unproven binding row".into());
        }
        Ok(())
    }

    /// Resolves only a future exact native-resume candidate; it never claims `LIVE` or `RESUMED`.
    pub(crate) fn resolve_runtime_resume_candidate(
        &self,
        session_id: &str,
        discovery: &RuntimeDiscovery,
    ) -> StoreResult<RuntimeResumeResolution> {
        self.load_winds_session(session_id)?;
        let bindings = self.list_runtime_session_bindings(session_id, discovery.runtime)?;
        if bindings.is_empty() {
            return Ok(RuntimeResumeResolution::Unavailable);
        }

        let exact = bindings
            .into_iter()
            .filter(|binding| runtime_binding_matches_discovery(binding, discovery))
            .collect::<Vec<_>>();
        if exact.is_empty() {
            return Ok(RuntimeResumeResolution::Stale);
        }

        let mut resumable = exact
            .into_iter()
            .filter(|binding| binding.native_session_id.is_some())
            .collect::<Vec<_>>();
        match resumable.len() {
            0 => Ok(RuntimeResumeResolution::Unavailable),
            1 => Ok(RuntimeResumeResolution::Candidate(
                resumable.pop().expect("one exact runtime resume candidate"),
            )),
            _ => Ok(RuntimeResumeResolution::Ambiguous(resumable)),
        }
    }
}

fn runtime_binding_from_stored_row(
    row: StoredRuntimeBindingRow,
) -> StoreResult<RuntimeSessionBinding> {
    let runtime = RuntimeKind::from_db(&row.runtime_kind)
        .ok_or_else(|| format!("unknown runtime kind in store: {}", row.runtime_kind))?;
    if row.runtime_version_state != "OBSERVED" {
        return Err(format!(
            "runtime binding has unsupported version state in store: {}",
            row.runtime_version_state
        )
        .into());
    }
    if row.runtime_version_source != "WINDS_LOCALLY_OBSERVED" {
        return Err(format!(
            "runtime binding has unsupported version source in store: {}",
            row.runtime_version_source
        )
        .into());
    }
    validate_runtime_binding_text(&row.runtime_version, "persisted runtime version")?;
    validate_runtime_binding_sha256(&row.executable_sha256)?;
    validate_runtime_binding_timestamp(row.bound_unix_ms, "persisted runtime binding time")?;
    if let Some(native_session_id) = row.native_session_id.as_deref() {
        validate_runtime_binding_text(native_session_id, "persisted native runtime session id")?;
    }

    let observed_path = PathBuf::from(row.observed_executable_path);
    let canonical_path = PathBuf::from(row.canonical_executable_path);
    if !observed_path.is_absolute() || !canonical_path.is_absolute() {
        return Err("persisted runtime executable identity must use absolute paths".into());
    }
    let ownership = RuntimeBindingOwnership::from_db(&row.ownership_state).ok_or_else(|| {
        format!(
            "unknown runtime binding ownership state in store: {}",
            row.ownership_state
        )
    })?;
    match (ownership, row.ownership_observed_unix_ms) {
        (RuntimeBindingOwnership::Unproven, None) => {}
        (RuntimeBindingOwnership::OwnershipLost, Some(observed))
            if observed >= row.bound_unix_ms => {}
        _ => return Err("persisted runtime binding ownership evidence is inconsistent".into()),
    }

    Ok(RuntimeSessionBinding {
        binding_id: row.binding_id,
        session_id: row.session_id,
        runtime,
        executable: RuntimeExecutableIdentity {
            observed_path,
            canonical_path,
            byte_len: u64::try_from(row.executable_byte_len)?,
            sha256: row.executable_sha256,
        },
        version: RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some(row.runtime_version),
            source: EvidenceSource::WindsLocallyObserved,
        },
        native_session_id: row.native_session_id,
        ownership,
        bound_unix_ms: row.bound_unix_ms,
        ownership_observed_unix_ms: row.ownership_observed_unix_ms,
    })
}

fn validate_runtime_binding_text(value: &str, label: &str) -> StoreResult<()> {
    if value.trim().is_empty() {
        return Err(format!("{label} must not be empty").into());
    }
    Ok(())
}

fn validate_runtime_binding_timestamp(value: i64, label: &str) -> StoreResult<()> {
    if value < 0 {
        return Err(format!("{label} must not be negative").into());
    }
    Ok(())
}

fn validate_runtime_binding_sha256(value: &str) -> StoreResult<()> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err("runtime executable SHA-256 must be 64 lowercase hexadecimal characters".into());
    }
    Ok(())
}

fn runtime_binding_path_text<'a>(path: &'a Path, label: &str) -> StoreResult<&'a str> {
    path.to_str()
        .ok_or_else(|| format!("{label} runtime executable path is not valid UTF-8").into())
}

fn build_version_evidence(
    observation: SafeVersionObservation,
) -> RuntimeDiscoveryResult<RuntimeVersionEvidence> {
    match observation {
        SafeVersionObservation::Observed(value) => Ok(RuntimeVersionEvidence {
            state: RuntimeVersionState::Observed,
            value: Some(validate_version_text(value)?),
            source: EvidenceSource::WindsLocallyObserved,
        }),
        SafeVersionObservation::Unsupported(value) => Ok(RuntimeVersionEvidence {
            state: RuntimeVersionState::Unsupported,
            value: Some(validate_version_text(value)?),
            source: EvidenceSource::WindsLocallyObserved,
        }),
        SafeVersionObservation::Unavailable => Ok(RuntimeVersionEvidence {
            state: RuntimeVersionState::Unavailable,
            value: None,
            source: EvidenceSource::Unavailable,
        }),
    }
}

fn validate_version_text(value: String) -> RuntimeDiscoveryResult<String> {
    if value.is_empty() {
        return Err("runtime version observation must not be empty".to_owned());
    }
    if value.len() > MAX_VERSION_BYTES {
        return Err(format!(
            "runtime version observation exceeds {MAX_VERSION_BYTES} bytes"
        ));
    }
    if value.trim() != value || value.chars().any(char::is_control) {
        return Err("runtime version observation must be one trimmed printable line".to_owned());
    }
    Ok(value)
}

fn build_capability_evidence(
    declarations: &[DeclaredCapability],
    local_observations: &[LocalCapabilityObservation],
) -> RuntimeDiscoveryResult<Vec<RuntimeCapabilityEvidence>> {
    let mut declared_keys = BTreeSet::new();
    for declaration in declarations {
        if !declared_keys.insert((declaration.capability, declaration.source)) {
            return Err("duplicate declared runtime capability evidence".to_owned());
        }
    }

    let mut local_keys = BTreeSet::new();
    for observation in local_observations {
        if !local_keys.insert(observation.capability) {
            return Err("duplicate local runtime capability evidence".to_owned());
        }
    }

    let mut evidence = Vec::new();
    for capability in [
        RuntimeCapability::StructuredControl,
        RuntimeCapability::NativeContinuation,
    ] {
        let mut found = false;
        for source in [DeclarationSource::Vendor, DeclarationSource::Catalog] {
            if let Some(declaration) = declarations
                .iter()
                .find(|item| item.capability == capability && item.source == source)
            {
                evidence.push(RuntimeCapabilityEvidence {
                    capability,
                    support: Some(declaration.support),
                    source: match source {
                        DeclarationSource::Vendor => EvidenceSource::VendorDeclared,
                        DeclarationSource::Catalog => EvidenceSource::CatalogDeclared,
                    },
                });
                found = true;
            }
        }
        if let Some(observation) = local_observations
            .iter()
            .find(|item| item.capability == capability)
        {
            evidence.push(RuntimeCapabilityEvidence {
                capability,
                support: Some(observation.support),
                source: EvidenceSource::WindsLocallyObserved,
            });
            found = true;
        }
        if !found {
            evidence.push(RuntimeCapabilityEvidence {
                capability,
                support: None,
                source: EvidenceSource::Unavailable,
            });
        }
    }
    Ok(evidence)
}

fn unknown_auth_readiness() -> AuthReadinessEvidence {
    AuthReadinessEvidence {
        readiness: AuthReadiness::Unknown,
        source: EvidenceSource::Unavailable,
    }
}

fn inspect_runtime_executable(
    observed_path: &Path,
) -> RuntimeDiscoveryResult<Option<RuntimeExecutableIdentity>> {
    if !observed_path.is_absolute() {
        return Err("runtime executable path must be absolute".to_owned());
    }

    let canonical_path = match fs::canonicalize(observed_path) {
        Ok(path) => path,
        Err(error) if expected_unavailable_path_error(&error) => return Ok(None),
        Err(error) => {
            return Err(format!(
                "runtime executable cannot be canonicalized ({}): {error}",
                observed_path.display()
            ));
        }
    };

    let Some(first) = snapshot_executable(&canonical_path)? else {
        return Ok(None);
    };
    let canonical_after = match fs::canonicalize(observed_path) {
        Ok(path) => path,
        Err(error) if expected_unavailable_path_error(&error) => {
            return Err("runtime executable changed during discovery".to_owned());
        }
        Err(error) => {
            return Err(format!(
                "runtime executable cannot be re-canonicalized ({}): {error}",
                observed_path.display()
            ));
        }
    };
    if canonical_after != canonical_path {
        return Err("runtime executable target changed during discovery".to_owned());
    }
    let Some(second) = snapshot_executable(&canonical_after)? else {
        return Err("runtime executable became unusable during discovery".to_owned());
    };
    if first != second {
        return Err("runtime executable bytes changed during discovery".to_owned());
    }

    Ok(Some(RuntimeExecutableIdentity {
        observed_path: observed_path.to_path_buf(),
        canonical_path,
        byte_len: first.byte_len,
        sha256: first.sha256,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutableSnapshot {
    byte_len: u64,
    sha256: String,
}

fn snapshot_executable(path: &Path) -> RuntimeDiscoveryResult<Option<ExecutableSnapshot>> {
    let initial_metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if expected_unavailable_path_error(&error) => return Ok(None),
        Err(error) => {
            return Err(format!(
                "runtime executable metadata cannot be read ({}): {error}",
                path.display()
            ));
        }
    };
    if !initial_metadata.is_file() || !has_platform_launch_permission(path, &initial_metadata) {
        return Ok(None);
    }
    if initial_metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(format!(
            "runtime executable exceeds bounded discovery size of {MAX_EXECUTABLE_BYTES} bytes"
        ));
    }

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(error) if expected_unavailable_path_error(&error) => return Ok(None),
        Err(error) => {
            return Err(format!(
                "runtime executable cannot be opened ({}): {error}",
                path.display()
            ));
        }
    };
    let metadata = file.metadata().map_err(|error| {
        format!(
            "runtime executable metadata cannot be read after open ({}): {error}",
            path.display()
        )
    })?;
    if !metadata.is_file() || !has_platform_launch_permission(path, &metadata) {
        return Ok(None);
    }
    if metadata.len() > MAX_EXECUTABLE_BYTES {
        return Err(format!(
            "runtime executable exceeds bounded discovery size of {MAX_EXECUTABLE_BYTES} bytes"
        ));
    }

    let mut digest = Sha256::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    let mut total_read = 0_u64;
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "runtime executable bytes cannot be read ({}): {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        total_read = total_read
            .checked_add(read as u64)
            .ok_or_else(|| "runtime executable byte count overflowed".to_owned())?;
        if total_read > MAX_EXECUTABLE_BYTES {
            return Err(format!(
                "runtime executable exceeded bounded discovery size of {MAX_EXECUTABLE_BYTES} bytes while reading"
            ));
        }
        digest.update(&buffer[..read]);
    }
    if total_read != metadata.len() {
        return Err("runtime executable size changed during snapshot".to_owned());
    }

    let sha256: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();

    Ok(Some(ExecutableSnapshot {
        byte_len: metadata.len(),
        sha256,
    }))
}

fn expected_unavailable_path_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        ErrorKind::NotFound | ErrorKind::PermissionDenied
    )
}

#[cfg(unix)]
fn has_platform_launch_permission(path: &Path, _metadata: &fs::Metadata) -> bool {
    let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };

    // SAFETY: `path` is a NUL-terminated CString whose pointer remains valid for the duration
    // of this call; `faccessat` does not retain the pointer. `AT_EACCESS` makes the check use
    // the process effective credentials and platform ACL/access rules without launching code.
    unsafe { libc::faccessat(libc::AT_FDCWD, path.as_ptr(), libc::X_OK, libc::AT_EACCESS) == 0 }
}

#[cfg(windows)]
fn has_platform_launch_permission(path: &Path, _metadata: &fs::Metadata) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "exe" | "com" | "cmd" | "bat"
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn has_platform_launch_permission(_path: &Path, _metadata: &fs::Metadata) -> bool {
    false
}
