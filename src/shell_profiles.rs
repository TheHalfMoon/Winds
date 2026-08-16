use super::Result;
use super::workspace_inventory::WorkspaceEnvironmentInventory;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[allow(
    dead_code,
    reason = "Spec 003 T048 backend API; terminal launch callers land in T050/T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShellExecutionDomain {
    NativeHost { os: String, arch: String },
}

#[allow(
    dead_code,
    reason = "Spec 003 T048 backend API; terminal launch callers land in T050/T057"
)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShellCwdStrategy {
    WorkspaceRoot,
}

#[allow(
    dead_code,
    reason = "Spec 003 T048 backend API; terminal launch callers land in T050/T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShellProfile {
    pub profile_id: String,
    pub display_name: String,
    pub execution_domain: ShellExecutionDomain,
    pub executable: String,
    pub arguments: Vec<String>,
    pub cwd_strategy: ShellCwdStrategy,
}

#[allow(
    dead_code,
    reason = "Spec 003 T048 backend API; terminal launch callers land in T050/T057"
)]
pub fn discover_native_shell_profiles(
    inventory: &WorkspaceEnvironmentInventory,
) -> Result<Vec<ShellProfile>> {
    require_current_host(inventory)?;

    let mut candidates: BTreeSet<String> = inventory.shell_candidates.iter().cloned().collect();
    extend_native_platform_candidates(&mut candidates);

    let mut profiles = Vec::new();
    for executable in candidates {
        if !candidate_is_usable(Path::new(&executable))? {
            continue;
        }
        profiles.push(build_profile(executable));
    }
    Ok(profiles)
}

#[allow(
    dead_code,
    reason = "Spec 003 T048 backend API; terminal launch callers land in T050/T057"
)]
pub fn validate_shell_profile_for_launch(profile: &ShellProfile) -> Result<()> {
    require_native_domain(&profile.execution_domain)?;
    if profile.cwd_strategy != ShellCwdStrategy::WorkspaceRoot {
        return Err("unsupported shell cwd strategy".into());
    }
    if !Path::new(&profile.executable).is_absolute() {
        return Err("shell profile executable must be an absolute path".into());
    }

    let expected_id = stable_profile_id(
        &profile.execution_domain,
        &profile.executable,
        &profile.arguments,
        profile.cwd_strategy,
    );
    if profile.profile_id != expected_id {
        return Err("shell profile identity does not match its launch data".into());
    }
    if !candidate_is_usable(Path::new(&profile.executable))? {
        return Err(format!(
            "shell profile executable is no longer usable: {}",
            profile.executable
        )
        .into());
    }
    Ok(())
}

fn require_current_host(inventory: &WorkspaceEnvironmentInventory) -> Result<()> {
    if inventory.host_os != std::env::consts::OS || inventory.host_arch != std::env::consts::ARCH {
        return Err(format!(
            "workspace inventory host does not match the current execution host: {} / {}",
            inventory.host_os, inventory.host_arch
        )
        .into());
    }
    Ok(())
}

fn require_native_domain(domain: &ShellExecutionDomain) -> Result<()> {
    match domain {
        ShellExecutionDomain::NativeHost { os, arch }
            if os == std::env::consts::OS && arch == std::env::consts::ARCH =>
        {
            Ok(())
        }
        ShellExecutionDomain::NativeHost { os, arch } => Err(format!(
            "shell profile execution domain does not match the current host: {os} / {arch}"
        )
        .into()),
    }
}

fn build_profile(executable: String) -> ShellProfile {
    let execution_domain = ShellExecutionDomain::NativeHost {
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    };
    let arguments = Vec::new();
    let cwd_strategy = ShellCwdStrategy::WorkspaceRoot;
    let display_name = Path::new(&executable)
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| executable.clone(), str::to_owned);
    let profile_id = stable_profile_id(
        &execution_domain,
        &executable,
        &arguments,
        cwd_strategy,
    );

    ShellProfile {
        profile_id,
        display_name,
        execution_domain,
        executable,
        arguments,
        cwd_strategy,
    }
}

fn stable_profile_id(
    domain: &ShellExecutionDomain,
    executable: &str,
    arguments: &[String],
    cwd_strategy: ShellCwdStrategy,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"WindsShellProfileV1\0");
    match domain {
        ShellExecutionDomain::NativeHost { os, arch } => {
            digest.update(b"NATIVE_HOST\0");
            digest.update(os.as_bytes());
            digest.update(b"\0");
            digest.update(arch.as_bytes());
            digest.update(b"\0");
        }
    }
    digest.update(executable.as_bytes());
    digest.update(b"\0");
    for argument in arguments {
        digest.update(argument.as_bytes());
        digest.update(b"\0");
    }
    match cwd_strategy {
        ShellCwdStrategy::WorkspaceRoot => digest.update(b"WORKSPACE_ROOT\0"),
    }
    let hex: String = digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("shell-profile-{hex}")
}

fn candidate_is_usable(path: &Path) -> Result<bool> {
    if !path.is_absolute() {
        return Ok(false);
    }
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(format!(
                "shell candidate cannot be inspected ({}): {error}",
                path.display()
            )
            .into());
        }
    };
    if !metadata.is_file() {
        return Ok(false);
    }
    candidate_has_platform_launch_permission(&metadata)
}

#[cfg(unix)]
fn candidate_has_platform_launch_permission(metadata: &fs::Metadata) -> Result<bool> {
    Ok(metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn candidate_has_platform_launch_permission(_metadata: &fs::Metadata) -> Result<bool> {
    Ok(true)
}

#[cfg(windows)]
fn extend_native_platform_candidates(candidates: &mut BTreeSet<String>) {
    for name in ["pwsh.exe", "powershell.exe", "cmd.exe"] {
        add_path_search_candidates(candidates, name);
    }

    if let Some(system_root) = std::env::var_os("SystemRoot") {
        let powershell = PathBuf::from(system_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe");
        add_utf8_absolute_candidate(candidates, &powershell);
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let pwsh = PathBuf::from(program_files)
            .join("PowerShell")
            .join("7")
            .join("pwsh.exe");
        add_utf8_absolute_candidate(candidates, &pwsh);
    }
}

#[cfg(not(windows))]
fn extend_native_platform_candidates(_candidates: &mut BTreeSet<String>) {}

#[cfg(windows)]
fn add_path_search_candidates(candidates: &mut BTreeSet<String>, executable_name: &str) {
    let Some(path_value) = std::env::var_os("PATH") else {
        return;
    };
    for directory in std::env::split_paths(&path_value) {
        add_utf8_absolute_candidate(candidates, &directory.join(executable_name));
    }
}

#[cfg(windows)]
fn add_utf8_absolute_candidate(candidates: &mut BTreeSet<String>, path: &Path) {
    if !path.is_absolute() {
        return;
    }
    if let Some(value) = path.to_str() {
        candidates.insert(value.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ShellCwdStrategy, ShellExecutionDomain, discover_native_shell_profiles,
        validate_shell_profile_for_launch,
    };
    use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
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
            "winds-t048-{name}-{}-{sequence}",
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
            .is_some_and(|name| name.starts_with("winds-t048-"));
        assert!(canonical_root.starts_with(&canonical_temp));
        assert!(owned_name);
        fs::remove_dir_all(&canonical_root).unwrap();
    }

    fn inventory(shell_candidates: Vec<String>) -> WorkspaceEnvironmentInventory {
        WorkspaceEnvironmentInventory {
            host_os: std::env::consts::OS.to_owned(),
            host_arch: std::env::consts::ARCH.to_owned(),
            canonical_worktree_root: "/unused/worktree".to_owned(),
            git_common_dir: "/unused/git-common".to_owned(),
            shell_candidates,
            detected_manifests: Vec::new(),
        }
    }

    #[cfg(unix)]
    fn create_executable(path: &Path, body: &str) {
        fs::write(path, body).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn discovery_returns_exact_nonexecuted_native_launch_profiles() {
        let root = test_root("discover");
        let marker = root.join("executed");
        let shell = root.join("fixture-shell");
        create_executable(
            &shell,
            &format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
        );
        let non_executable = root.join("not-executable");
        fs::write(&non_executable, b"not executable\n").unwrap();
        let directory = root.join("directory-shell");
        fs::create_dir(&directory).unwrap();
        let missing = root.join("missing-shell");

        let discovered = discover_native_shell_profiles(&inventory(vec![
            missing.to_str().unwrap().to_owned(),
            shell.to_str().unwrap().to_owned(),
            non_executable.to_str().unwrap().to_owned(),
            directory.to_str().unwrap().to_owned(),
            shell.to_str().unwrap().to_owned(),
        ]))
        .unwrap();

        assert_eq!(discovered.len(), 1);
        let profile = &discovered[0];
        assert_eq!(profile.executable, shell.to_str().unwrap());
        assert!(profile.arguments.is_empty());
        assert_eq!(profile.display_name, "fixture-shell");
        assert_eq!(profile.cwd_strategy, ShellCwdStrategy::WorkspaceRoot);
        assert_eq!(
            profile.execution_domain,
            ShellExecutionDomain::NativeHost {
                os: std::env::consts::OS.to_owned(),
                arch: std::env::consts::ARCH.to_owned(),
            }
        );
        assert!(profile.profile_id.starts_with("shell-profile-"));
        assert!(!marker.exists());
        validate_shell_profile_for_launch(profile).unwrap();
        assert!(!marker.exists());

        let json = serde_json::to_string(profile).unwrap();
        assert!(!json.contains("touch"));

        cleanup_owned_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn launch_validation_rechecks_stale_executable_identity() {
        let root = test_root("stale");
        let shell = root.join("fixture-shell");
        create_executable(&shell, "#!/bin/sh\n");
        let mut discovered = discover_native_shell_profiles(&inventory(vec![
            shell.to_str().unwrap().to_owned(),
        ]))
        .unwrap();
        assert_eq!(discovered.len(), 1);
        let profile = discovered.pop().unwrap();

        fs::remove_file(&shell).unwrap();
        let error = validate_shell_profile_for_launch(&profile).unwrap_err();
        assert!(error.to_string().contains("no longer usable"));

        cleanup_owned_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn stable_profile_identity_binds_launch_data_but_not_display_name() {
        let root = test_root("identity");
        let shell = root.join("fixture-shell");
        create_executable(&shell, "#!/bin/sh\n");
        let mut discovered = discover_native_shell_profiles(&inventory(vec![
            shell.to_str().unwrap().to_owned(),
        ]))
        .unwrap();
        let mut profile = discovered.pop().unwrap();

        profile.display_name = "UX label only".to_owned();
        validate_shell_profile_for_launch(&profile).unwrap();

        profile.arguments.push("--changed".to_owned());
        let error = validate_shell_profile_for_launch(&profile).unwrap_err();
        assert!(error.to_string().contains("identity does not match"));

        cleanup_owned_root(&root);
    }

    #[test]
    fn discovery_rejects_inventory_from_another_execution_host() {
        let mut inventory = inventory(Vec::new());
        inventory.host_os = "different-os".to_owned();
        let error = discover_native_shell_profiles(&inventory).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("does not match the current execution host")
        );
    }
}
