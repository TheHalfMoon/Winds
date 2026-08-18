use super::Result;
use super::workspace::WorkspaceInspection;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
#[cfg(unix)]
use std::fs::File;
use std::io::ErrorKind;
#[cfg(unix)]
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

const MANIFEST_PATHS: &[&str] = &[
    ".devcontainer/devcontainer.json",
    ".envrc",
    ".mise.toml",
    ".nvmrc",
    ".python-version",
    ".tool-versions",
    "devcontainer.json",
    "rust-toolchain.toml",
];

#[allow(
    dead_code,
    reason = "Spec 003 T047 backend API; the user-facing CLI caller lands in T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WorkspaceEnvironmentInventory {
    pub host_os: String,
    pub host_arch: String,
    pub canonical_worktree_root: String,
    pub git_common_dir: String,
    pub shell_candidates: Vec<String>,
    pub detected_manifests: Vec<String>,
}

#[allow(
    dead_code,
    reason = "Spec 003 T047 backend API; the user-facing CLI caller lands in T057"
)]
pub fn inventory_workspace_environment(
    workspace: &WorkspaceInspection,
) -> Result<WorkspaceEnvironmentInventory> {
    let worktree_root = require_current_canonical_directory(
        &workspace.canonical_worktree_root,
        "canonical worktree root",
    )?;
    require_current_canonical_directory(&workspace.git_common_dir, "Git common directory")?;

    Ok(WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: workspace.canonical_worktree_root.clone(),
        git_common_dir: workspace.git_common_dir.clone(),
        shell_candidates: discover_shell_candidates()?,
        detected_manifests: detect_manifests(&worktree_root)?,
    })
}

fn require_current_canonical_directory(value: &str, label: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if !path.is_absolute() {
        return Err(format!("{label} must be an absolute path").into());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{label} cannot be canonicalized: {error}"))?;
    if canonical != path {
        return Err(format!("{label} is no longer canonical").into());
    }
    if !canonical.is_dir() {
        return Err(format!("{label} is not a directory").into());
    }
    Ok(canonical)
}

fn detect_manifests(worktree_root: &Path) -> Result<Vec<String>> {
    let mut detected = Vec::new();
    for relative in MANIFEST_PATHS {
        if manifest_path_present(worktree_root, relative)? {
            detected.push((*relative).to_owned());
        }
    }
    Ok(detected)
}

fn manifest_path_present(worktree_root: &Path, relative: &str) -> Result<bool> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        return Err(format!("manifest path must be relative: {relative}").into());
    }

    let mut current_parent = worktree_root.to_path_buf();
    if let Some(parent) = relative_path.parent() {
        for component in parent.components() {
            let Component::Normal(name) = component else {
                return Err(
                    format!("manifest path contains an unsafe component: {relative}").into(),
                );
            };
            current_parent.push(name);
            match fs::symlink_metadata(&current_parent) {
                Ok(metadata) => {
                    let kind = metadata.file_type();
                    if kind.is_symlink() {
                        return Err(format!(
                            "manifest parent must not be a symlink ({relative}): {}",
                            current_parent.display()
                        )
                        .into());
                    }
                    if !kind.is_dir() {
                        return Err(format!(
                            "manifest parent is not a directory ({relative}): {}",
                            current_parent.display()
                        )
                        .into());
                    }
                }
                Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
                Err(error) => {
                    return Err(format!(
                        "manifest parent cannot be inspected ({relative}): {error}"
                    )
                    .into());
                }
            }
        }
    }

    match fs::symlink_metadata(worktree_root.join(relative_path)) {
        Ok(metadata) => {
            let kind = metadata.file_type();
            if kind.is_file() || kind.is_symlink() {
                Ok(true)
            } else {
                Err(format!("manifest path is not a file or symlink: {relative}").into())
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => {
            Err(format!("manifest path cannot be inspected ({relative}): {error}").into())
        }
    }
}

fn discover_shell_candidates() -> Result<Vec<String>> {
    let mut candidates = BTreeSet::new();
    add_environment_shell_candidate(&mut candidates, "SHELL");
    add_environment_shell_candidate(&mut candidates, "COMSPEC");

    for candidate in system_shell_candidates()? {
        add_shell_candidate(&mut candidates, &candidate);
    }

    Ok(candidates.into_iter().collect())
}

fn add_environment_shell_candidate(candidates: &mut BTreeSet<String>, variable: &str) {
    let Some(value) = std::env::var_os(variable) else {
        return;
    };
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return;
    }
    let Some(value) = path.to_str() else {
        return;
    };
    candidates.insert(value.to_owned());
}

fn add_shell_candidate(candidates: &mut BTreeSet<String>, value: &str) {
    let path = Path::new(value);
    if path.is_absolute() {
        candidates.insert(value.to_owned());
    }
}

#[cfg(unix)]
fn system_shell_candidates() -> Result<Vec<String>> {
    let file = match File::open("/etc/shells") {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("/etc/shells cannot be inspected: {error}").into()),
    };
    parse_system_shells(BufReader::new(file))
}

#[cfg(not(unix))]
fn system_shell_candidates() -> Result<Vec<String>> {
    Ok(Vec::new())
}

#[cfg(unix)]
fn parse_system_shells(reader: impl BufRead) -> Result<Vec<String>> {
    let mut candidates = BTreeSet::new();
    for line in reader.lines() {
        let line =
            line.map_err(|error| format!("/etc/shells cannot be read completely: {error}"))?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        add_shell_candidate(&mut candidates, line);
    }
    Ok(candidates.into_iter().collect())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::parse_system_shells;
    use super::{MANIFEST_PATHS, inventory_workspace_environment};
    use crate::git::workspace::WorkspaceInspection;
    use std::ffi::OsStr;
    use std::fs;
    #[cfg(unix)]
    use std::io::Cursor;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    fn test_root(name: &str) -> PathBuf {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "winds-t047-{name}-{}-{sequence}",
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
            .is_some_and(|name| name.starts_with("winds-t047-"));
        assert!(canonical_root.starts_with(&canonical_temp));
        assert!(owned_name);
        fs::remove_dir_all(&canonical_root).unwrap();
    }

    fn fixture_workspace(root: &Path) -> (WorkspaceInspection, PathBuf) {
        let worktree = root.join("repo");
        let common_dir = root.join("git-common");
        fs::create_dir(&worktree).unwrap();
        fs::create_dir(&common_dir).unwrap();
        let worktree = worktree.canonicalize().unwrap();
        let common_dir = common_dir.canonicalize().unwrap();

        (
            WorkspaceInspection {
                workspace_id: "workspace-test".to_owned(),
                canonical_worktree_root: worktree.to_str().unwrap().to_owned(),
                git_common_dir: common_dir.to_str().unwrap().to_owned(),
                head_oid: None,
                branch: Some("main".to_owned()),
                detached: false,
                dirty: false,
            },
            worktree,
        )
    }

    #[test]
    fn inventory_reports_safe_metadata_without_reading_or_executing_manifests() {
        let root = test_root("safe-inventory");
        let (workspace, worktree) = fixture_workspace(&root);
        let marker = root.join("manifest-executed");
        let secret = "T047_SECRET_VALUE_MUST_NOT_APPEAR";

        fs::write(
            worktree.join(".envrc"),
            format!("export TOKEN={secret}\ntouch '{}'\n", marker.display()),
        )
        .unwrap();
        fs::write(worktree.join(".mise.toml"), format!("# {secret}\n")).unwrap();
        fs::write(worktree.join(".env"), format!("TOKEN={secret}\n")).unwrap();
        fs::create_dir(worktree.join(".devcontainer")).unwrap();
        fs::write(
            worktree.join(".devcontainer/devcontainer.json"),
            format!("{{\"secret\":\"{secret}\"}}\n"),
        )
        .unwrap();

        let inventory = inventory_workspace_environment(&workspace).unwrap();

        assert_eq!(inventory.host_os, std::env::consts::OS);
        assert_eq!(inventory.host_arch, std::env::consts::ARCH);
        assert_eq!(
            inventory.canonical_worktree_root,
            workspace.canonical_worktree_root
        );
        assert_eq!(inventory.git_common_dir, workspace.git_common_dir);
        assert_eq!(
            inventory.detected_manifests,
            vec![
                ".devcontainer/devcontainer.json".to_owned(),
                ".envrc".to_owned(),
                ".mise.toml".to_owned(),
            ]
        );
        assert!(!inventory.detected_manifests.contains(&".env".to_owned()));
        assert!(!marker.exists());

        let json = serde_json::to_string(&inventory).unwrap();
        assert!(!json.contains(secret));
        assert!(!json.contains("TOKEN="));
        assert_eq!(MANIFEST_PATHS.len(), 8);
        assert!(
            inventory
                .shell_candidates
                .windows(2)
                .all(|pair| pair[0] < pair[1])
        );
        assert!(
            inventory
                .shell_candidates
                .iter()
                .all(|candidate| Path::new(candidate).is_absolute())
        );

        cleanup_owned_root(&root);
    }

    #[test]
    fn inventory_fails_when_workspace_identity_path_is_stale() {
        let root = test_root("stale-path");
        let (workspace, worktree) = fixture_workspace(&root);
        let moved = root.join("repo-moved");
        fs::rename(&worktree, &moved).unwrap();

        let error = inventory_workspace_environment(&workspace).unwrap_err();
        assert!(error.to_string().contains("cannot be canonicalized"));

        cleanup_owned_root(&root);
    }

    #[test]
    fn inventory_fails_when_workspace_identity_path_exists_but_is_noncanonical() {
        let root = test_root("noncanonical-path");
        let (mut workspace, worktree) = fixture_workspace(&root);

        #[cfg(windows)]
        let noncanonical = {
            let canonical = worktree.to_str().unwrap();
            let ordinary = canonical
                .strip_prefix(r"\\?\")
                .expect("Windows temp canonical path should use the verbatim drive prefix");
            PathBuf::from(ordinary)
        };
        #[cfg(not(windows))]
        let noncanonical = worktree.join("..");

        assert!(noncanonical.is_absolute());
        assert!(noncanonical.exists());
        assert_ne!(noncanonical, worktree);
        workspace.canonical_worktree_root = noncanonical.to_str().unwrap().to_owned();

        let error = inventory_workspace_environment(&workspace).unwrap_err();
        assert!(error.to_string().contains("no longer canonical"));

        cleanup_owned_root(&root);
    }

    #[test]
    fn manifest_metadata_ambiguity_fails_closed_instead_of_looking_absent() {
        let root = test_root("manifest-ambiguity");
        let (workspace, worktree) = fixture_workspace(&root);
        fs::write(worktree.join(".devcontainer"), b"not a directory\n").unwrap();

        let error = inventory_workspace_environment(&workspace).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("manifest parent is not a directory")
        );
        assert!(
            error
                .to_string()
                .contains(".devcontainer/devcontainer.json")
        );

        cleanup_owned_root(&root);
    }

    #[test]
    fn manifest_leaf_type_ambiguity_fails_closed_instead_of_looking_absent() {
        let root = test_root("manifest-leaf-ambiguity");
        let (workspace, worktree) = fixture_workspace(&root);
        fs::create_dir(worktree.join(".envrc")).unwrap();

        let error = inventory_workspace_environment(&workspace).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("manifest path is not a file or symlink")
        );
        assert!(error.to_string().contains(".envrc"));

        cleanup_owned_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn nested_manifest_parent_symlink_fails_closed_without_inspecting_external_target() {
        let root = test_root("manifest-parent-symlink");
        let (workspace, worktree) = fixture_workspace(&root);
        let outside = root.join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("devcontainer.json"), b"{\"outside\":true}\n").unwrap();
        symlink(&outside, worktree.join(".devcontainer")).unwrap();

        let error = inventory_workspace_environment(&workspace).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("manifest parent must not be a symlink")
        );
        assert!(
            error
                .to_string()
                .contains(".devcontainer/devcontainer.json")
        );

        cleanup_owned_root(&root);
    }

    #[cfg(unix)]
    #[test]
    fn system_shell_parser_is_nonexecuting_deterministic_and_fail_closed() {
        let input = Cursor::new(
            b"# system shells\n/bin/zsh\n/bin/bash\nrelative-shell\n/bin/zsh\n\n/usr/bin/fish\n",
        );
        let parsed = parse_system_shells(input).unwrap();
        assert_eq!(
            parsed,
            vec![
                "/bin/bash".to_owned(),
                "/bin/zsh".to_owned(),
                "/usr/bin/fish".to_owned(),
            ]
        );
    }
}
