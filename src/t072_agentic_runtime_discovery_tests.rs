use crate::agentic_runtime::{
    AgentExecutionObservation, AuthReadiness, CapabilitySupport, DeclarationSource,
    DeclaredCapability, EvidenceSource, LocalCapabilityObservation, MAX_EXECUTABLE_BYTES,
    RuntimeCapability, RuntimeDiscoveryState, RuntimeIdentityRevalidation, RuntimeKind,
    RuntimeVersionState, SafeVersionObservation, discover_runtime_from_safe_observations,
    revalidate_runtime_identity,
};
use std::ffi::OsStr;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn test_root(name: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t072-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}

fn cleanup_owned_root(root: &Path) {
    let canonical_root = root.canonicalize().unwrap();
    let canonical_temp = std::env::temp_dir().canonicalize().unwrap();
    let owned_name = canonical_root
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with("winds-t072-"));
    assert!(canonical_root.starts_with(&canonical_temp));
    assert!(owned_name);
    fs::remove_dir_all(&canonical_root).unwrap();
}

fn fake_executable_path(root: &Path, stem: &str) -> PathBuf {
    #[cfg(windows)]
    {
        root.join(format!("{stem}.exe"))
    }
    #[cfg(not(windows))]
    {
        root.join(stem)
    }
}

fn create_fake_executable(root: &Path, stem: &str, bytes: &[u8]) -> PathBuf {
    let path = fake_executable_path(root, stem);
    fs::write(&path, bytes).unwrap();
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }
    path
}

#[test]
fn absent_runtime_is_unavailable_without_agent_execution() {
    let root = test_root("absent");
    let missing = root.join("missing-codex");

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &missing,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap();

    assert_eq!(discovery.runtime, RuntimeKind::Codex);
    assert_eq!(discovery.state, RuntimeDiscoveryState::Unavailable);
    assert!(discovery.executable.is_none());
    assert_eq!(discovery.version.state, RuntimeVersionState::Unavailable);
    assert_eq!(discovery.version.source, EvidenceSource::Unavailable);
    assert_eq!(discovery.auth_readiness.readiness, AuthReadiness::Unknown);
    assert_eq!(discovery.auth_readiness.source, EvidenceSource::Unavailable);
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );
    assert!(discovery.capabilities.iter().all(|evidence| {
        evidence.support.is_none() && evidence.source == EvidenceSource::Unavailable
    }));

    cleanup_owned_root(&root);
}

#[test]
fn non_file_runtime_path_is_unavailable_instead_of_aborting_discovery() {
    let root = test_root("non-file");
    let runtime_dir = root.join("runtime-dir");
    fs::create_dir(&runtime_dir).unwrap();

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &runtime_dir,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap();

    assert_eq!(discovery.state, RuntimeDiscoveryState::Unavailable);
    assert!(discovery.executable.is_none());
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );

    cleanup_owned_root(&root);
}

#[cfg(unix)]
#[test]
fn unix_execute_bits_do_not_override_effective_user_access() {
    let root = test_root("unix-effective-exec");
    let executable = fake_executable_path(&root, "fixture-codex");
    fs::write(&executable, b"readable-but-not-owner-executable\n").unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o401);
    fs::set_permissions(&executable, permissions).unwrap();

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap();

    assert_eq!(discovery.state, RuntimeDiscoveryState::Unavailable);
    assert!(discovery.executable.is_none());
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );

    cleanup_owned_root(&root);
}

#[cfg(unix)]
#[test]
fn oversized_runtime_fails_closed_before_unbounded_hashing() {
    let root = test_root("oversized");
    let executable = fake_executable_path(&root, "fixture-codex");
    let file = fs::File::create(&executable).unwrap();
    file.set_len(MAX_EXECUTABLE_BYTES + 1).unwrap();
    drop(file);
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions).unwrap();

    let error = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap_err();

    assert!(error.contains("bounded discovery size"));
    cleanup_owned_root(&root);
}

#[cfg(windows)]
#[test]
fn windows_regular_file_without_launch_extension_is_unavailable() {
    let root = test_root("windows-extension");
    let non_executable = root.join("fixture-codex.txt");
    fs::write(&non_executable, b"not-a-windows-launch-file\n").unwrap();

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &non_executable,
        SafeVersionObservation::Unavailable,
        &[],
        &[],
    )
    .unwrap();

    assert_eq!(discovery.state, RuntimeDiscoveryState::Unavailable);
    assert!(discovery.executable.is_none());
    cleanup_owned_root(&root);
}

#[test]
fn present_runtime_keeps_declared_and_locally_observed_capabilities_distinct() {
    let root = test_root("present");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");

    let declarations = [
        DeclaredCapability {
            capability: RuntimeCapability::StructuredControl,
            support: CapabilitySupport::Supported,
            source: DeclarationSource::Vendor,
        },
        DeclaredCapability {
            capability: RuntimeCapability::NativeContinuation,
            support: CapabilitySupport::Supported,
            source: DeclarationSource::Catalog,
        },
    ];
    let observations = [LocalCapabilityObservation {
        capability: RuntimeCapability::StructuredControl,
        support: CapabilitySupport::Supported,
    }];

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed("codex-cli 1.2.3-fixture".to_owned()),
        &declarations,
        &observations,
    )
    .unwrap();

    assert_eq!(discovery.state, RuntimeDiscoveryState::Present);
    let identity = discovery.executable.as_ref().unwrap();
    assert_eq!(identity.observed_path, executable);
    assert_eq!(identity.canonical_path, executable.canonicalize().unwrap());
    assert_eq!(identity.byte_len, b"fixture-codex-v1\n".len() as u64);
    assert_eq!(identity.sha256.len(), 64);
    assert_eq!(discovery.version.state, RuntimeVersionState::Observed);
    assert_eq!(
        discovery.version.value.as_deref(),
        Some("codex-cli 1.2.3-fixture")
    );
    assert_eq!(
        discovery.version.source,
        EvidenceSource::WindsLocallyObserved
    );

    let structured: Vec<_> = discovery
        .capabilities
        .iter()
        .filter(|evidence| evidence.capability == RuntimeCapability::StructuredControl)
        .collect();
    assert_eq!(structured.len(), 2);
    assert_eq!(structured[0].source, EvidenceSource::VendorDeclared);
    assert_eq!(structured[1].source, EvidenceSource::WindsLocallyObserved);

    let continuation: Vec<_> = discovery
        .capabilities
        .iter()
        .filter(|evidence| evidence.capability == RuntimeCapability::NativeContinuation)
        .collect();
    assert_eq!(continuation.len(), 1);
    assert_eq!(continuation[0].source, EvidenceSource::CatalogDeclared);

    assert_eq!(discovery.auth_readiness.readiness, AuthReadiness::Unknown);
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );

    cleanup_owned_root(&root);
}

#[test]
fn unsupported_version_is_explicit_and_does_not_invent_auth_readiness() {
    let root = test_root("unsupported-version");
    let executable = create_fake_executable(&root, "fixture-claude", b"fixture-claude-old\n");

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Claude,
        &executable,
        SafeVersionObservation::Unsupported("claude-code 0.0-fixture".to_owned()),
        &[],
        &[],
    )
    .unwrap();

    assert_eq!(discovery.runtime, RuntimeKind::Claude);
    assert_eq!(discovery.state, RuntimeDiscoveryState::UnsupportedVersion);
    assert_eq!(discovery.version.state, RuntimeVersionState::Unsupported);
    assert_eq!(
        discovery.version.source,
        EvidenceSource::WindsLocallyObserved
    );
    assert_eq!(discovery.auth_readiness.readiness, AuthReadiness::Unknown);
    assert_eq!(discovery.auth_readiness.source, EvidenceSource::Unavailable);
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );

    cleanup_owned_root(&root);
}

#[test]
fn unobservable_capability_stays_unavailable_instead_of_becoming_observed() {
    let root = test_root("unobservable-capability");
    let executable = create_fake_executable(&root, "fixture-claude", b"fixture-claude-v1\n");

    let declarations = [DeclaredCapability {
        capability: RuntimeCapability::StructuredControl,
        support: CapabilitySupport::Supported,
        source: DeclarationSource::Vendor,
    }];
    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Claude,
        &executable,
        SafeVersionObservation::Observed("claude-code 1.0-fixture".to_owned()),
        &declarations,
        &[],
    )
    .unwrap();

    let structured: Vec<_> = discovery
        .capabilities
        .iter()
        .filter(|evidence| evidence.capability == RuntimeCapability::StructuredControl)
        .collect();
    assert_eq!(structured.len(), 1);
    assert_eq!(structured[0].source, EvidenceSource::VendorDeclared);

    let continuation: Vec<_> = discovery
        .capabilities
        .iter()
        .filter(|evidence| evidence.capability == RuntimeCapability::NativeContinuation)
        .collect();
    assert_eq!(continuation.len(), 1);
    assert_eq!(continuation[0].support, None);
    assert_eq!(continuation[0].source, EvidenceSource::Unavailable);

    cleanup_owned_root(&root);
}

#[test]
fn revalidation_detects_replaced_executable_before_use() {
    let root = test_root("replacement");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-v1\n");

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed("codex-cli 1.2.3-fixture".to_owned()),
        &[],
        &[],
    )
    .unwrap();
    let identity = discovery.executable.as_ref().unwrap();

    assert_eq!(
        revalidate_runtime_identity(identity).unwrap(),
        RuntimeIdentityRevalidation::Match
    );

    fs::write(&executable, b"fixture-codex-v2-replaced\n").unwrap();
    assert_eq!(
        revalidate_runtime_identity(identity).unwrap(),
        RuntimeIdentityRevalidation::Changed
    );

    fs::remove_file(&executable).unwrap();
    assert_eq!(
        revalidate_runtime_identity(identity).unwrap(),
        RuntimeIdentityRevalidation::Unavailable
    );

    cleanup_owned_root(&root);
}

#[test]
fn unavailable_runtime_rejects_fabricated_local_observation() {
    let root = test_root("fabricated-local-observation");
    let missing = root.join("missing-claude");
    let observations = [LocalCapabilityObservation {
        capability: RuntimeCapability::StructuredControl,
        support: CapabilitySupport::Supported,
    }];

    let error = discover_runtime_from_safe_observations(
        RuntimeKind::Claude,
        &missing,
        SafeVersionObservation::Unavailable,
        &[],
        &observations,
    )
    .unwrap_err();

    assert!(error.contains("unavailable runtime"));
    cleanup_owned_root(&root);
}

#[test]
fn runtime_identity_is_explicit_and_not_inferred_from_version_text() {
    let root = test_root("runtime-model-separation");
    let executable = create_fake_executable(&root, "fixture-codex", b"fixture-codex-model-text\n");

    let discovery = discover_runtime_from_safe_observations(
        RuntimeKind::Codex,
        &executable,
        SafeVersionObservation::Observed("vendor text mentioning claude and model-x".to_owned()),
        &[],
        &[],
    )
    .unwrap();

    assert_eq!(discovery.runtime, RuntimeKind::Codex);
    assert_eq!(discovery.state, RuntimeDiscoveryState::Present);
    assert_eq!(
        discovery.agent_execution,
        AgentExecutionObservation::NotPerformed
    );

    cleanup_owned_root(&root);
}
