use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
#[cfg(unix)]
use std::ffi::CString;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const HASH_BUFFER_BYTES: usize = 64 * 1024;
pub(crate) const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_VERSION_BYTES: usize = 256;

pub(crate) type RuntimeDiscoveryResult<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RuntimeKind {
    Codex,
    Claude,
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
