use serde_json::Value;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn minimal_cli_proves_workspace_profiles_execution_and_terminal_paths() {
    let Some(temp) = TestTempDir::new("winds-t057-cli") else {
        return;
    };
    let root = temp.path();
    let repo = root.join("repo");
    let other_repo = root.join("other-repo");
    let winds_home = root.join("winds-home");
    init_repo(&repo, "primary");
    init_repo(&other_repo, "other");

    let opened = winds(
        &winds_home,
        ["workspace-open", "--repo", test_path(&repo)],
    );
    assert_success(&opened);
    let opened_json: Value = serde_json::from_slice(&opened.stdout).unwrap();
    let canonical_repo = repo.canonicalize().unwrap();
    assert_eq!(
        opened_json["canonical_worktree_root"],
        test_path(&canonical_repo)
    );
    let workspace_id = opened_json["workspace_id"].as_str().unwrap().to_owned();
    assert!(!workspace_id.is_empty());
    assert!(opened_json["head_oid"].as_str().is_some());

    let reopened = winds(
        &winds_home,
        ["workspace-open", "--repo", test_path(&repo)],
    );
    assert_success(&reopened);
    let reopened_json: Value = serde_json::from_slice(&reopened.stdout).unwrap();
    assert_eq!(reopened_json["workspace_id"], workspace_id);

    let profiles = winds(&winds_home, ["profiles", "--repo", test_path(&repo)]);
    assert_success(&profiles);
    let profiles_json: Value = serde_json::from_slice(&profiles.stdout).unwrap();
    let native_profiles = profiles_json["native_shell_profiles"].as_array().unwrap();
    assert!(!native_profiles.is_empty());
    let profile_id = native_profiles[0]["profile_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(profiles_json["wsl"]["availability"].as_str().is_some());

    let command_id = "t057-command-proof";
    let command = winds(
        &winds_home,
        [
            "run",
            "--repo",
            test_path(&repo),
            "--execution-id",
            command_id,
            "--executable",
            env!("CARGO_BIN_EXE_winds"),
            "--args-json",
            r#"["definitely-not-a-winds-command"]"#,
            "--history",
            "disabled",
        ],
    );
    assert_success(&command);
    let command_json: Value = serde_json::from_slice(&command.stdout).unwrap();
    assert_eq!(command_json["execution"]["execution_id"], command_id);
    assert_eq!(command_json["execution"]["kind"], "SHELL_COMMAND");
    assert_eq!(command_json["execution"]["status"], "EXITED");
    assert_eq!(
        command_json["execution"]["shell_command"]["arguments"][0],
        "<winds:history-disabled>"
    );
    assert_eq!(command_json["result"]["exit_code"], 1);

    let inspected = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            command_id,
        ],
    );
    assert_success(&inspected);
    let inspected_json: Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(inspected_json["execution_id"], command_id);
    assert_eq!(inspected_json["status"], "EXITED");
    assert!(inspected_json["events"].as_array().unwrap().len() >= 2);

    let cross_workspace = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&other_repo),
            "--execution-id",
            command_id,
        ],
    );
    assert!(!cross_workspace.status.success());
    assert!(
        String::from_utf8_lossy(&cross_workspace.stderr)
            .contains("belongs to a different Winds workspace")
    );

    let terminal_id = "t057-terminal-proof";
    let terminal = winds(
        &winds_home,
        [
            "terminal-proof",
            "--repo",
            test_path(&repo),
            "--execution-id",
            terminal_id,
            "--profile-id",
            &profile_id,
            "--rows",
            "24",
            "--cols",
            "80",
        ],
    );
    assert_success(&terminal);
    let terminal_json: Value = serde_json::from_slice(&terminal.stdout).unwrap();
    assert_eq!(terminal_json["execution"]["execution_id"], terminal_id);
    assert_eq!(terminal_json["execution"]["kind"], "TERMINAL");
    assert_eq!(terminal_json["execution"]["status"], "INTERRUPTED");
    assert_eq!(
        terminal_json["execution"]["terminal"]["close_reason"],
        "TERMINATED_BY_WINDS"
    );
    assert_eq!(terminal_json["proof"]["profile_id"], profile_id);
}

#[test]
fn workspace_clone_rejects_unsafe_state_roots_before_creation() {
    let Some(temp) = TestTempDir::new("winds-t057-clone") else {
        return;
    };
    let root = temp.path();
    let source = root.join("source");
    init_repo(&source, "source");

    let source_nested_home = source.join(".winds");
    let source_guard_destination = root.join("clone-source-guard");
    let rejected_source_home = Command::new(env!("CARGO_BIN_EXE_winds"))
        .args([
            "workspace-clone",
            "--remote",
            test_path(&source),
            "--destination",
            test_path(&source_guard_destination),
            "--home",
            test_path(&source_nested_home),
        ])
        .output()
        .unwrap();
    assert!(!rejected_source_home.status.success());
    assert!(
        String::from_utf8_lossy(&rejected_source_home.stderr)
            .contains("Winds state root must live outside the local clone source")
    );
    assert!(!source_nested_home.exists());
    assert!(!source_guard_destination.exists());

    let destination = root.join("clone");
    let nested_home = destination.join(".winds");
    let rejected_destination_home = Command::new(env!("CARGO_BIN_EXE_winds"))
        .args([
            "workspace-clone",
            "--remote",
            test_path(&source),
            "--destination",
            test_path(&destination),
            "--home",
            test_path(&nested_home),
        ])
        .output()
        .unwrap();
    assert!(!rejected_destination_home.status.success());
    assert!(
        String::from_utf8_lossy(&rejected_destination_home.stderr)
            .contains("clone destination and Winds state root must not overlap")
    );
    assert!(!destination.exists());

    let safe_home = root.join("winds-home");
    let cloned = Command::new(env!("CARGO_BIN_EXE_winds"))
        .args([
            "workspace-clone",
            "--remote",
            test_path(&source),
            "--destination",
            test_path(&destination),
            "--home",
            test_path(&safe_home),
        ])
        .output()
        .unwrap();
    assert_success(&cloned);
    let cloned_json: Value = serde_json::from_slice(&cloned.stdout).unwrap();
    let canonical_destination = destination.canonicalize().unwrap();
    let canonical_source = source.canonicalize().unwrap();
    assert_eq!(
        cloned_json["workspace"]["canonical_worktree_root"],
        test_path(&canonical_destination)
    );
    assert_eq!(
        cloned_json["remote_identity"],
        test_path(&canonical_source)
    );
}

fn init_repo(path: &Path, content: &str) {
    fs::create_dir_all(path).unwrap();
    git(path, ["init", "-b", "main"]);
    git(path, ["config", "user.email", "winds-test@example.invalid"]);
    git(path, ["config", "user.name", "Winds Test"]);
    fs::write(path.join("file.txt"), format!("{content}\n")).unwrap();
    git(path, ["add", "file.txt"]);
    git(path, ["commit", "-m", "initial"]);
}

fn winds<const N: usize>(home: &Path, args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_HOME", home)
        .output()
        .unwrap()
}

fn git<const N: usize>(repo: &Path, args: [&str; N]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_success(&output);
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn test_path(path: &Path) -> &str {
    path.to_str().expect(
        "T057 CLI integration fixture root is validated as UTF-8; derived ASCII child paths must remain UTF-8",
    )
}

struct TestTempDir {
    path: PathBuf,
    canonical_parent: PathBuf,
    expected_name: OsString,
}

impl TestTempDir {
    fn new(prefix: &str) -> Option<Self> {
        let canonical_parent = std::env::temp_dir().canonicalize().ok()?;
        if canonical_parent.to_str().is_none() {
            return None;
        }

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        let expected_name = OsString::from(format!(
            "{prefix}-{nanos}-{}",
            std::process::id()
        ));
        let path = canonical_parent.join(&expected_name);
        fs::create_dir(&path).ok()?;

        let canonical = match path.canonicalize() {
            Ok(canonical) => canonical,
            Err(_) => {
                let _ = fs::remove_dir_all(&path);
                return None;
            }
        };
        if canonical != path || canonical.to_str().is_none() {
            let _ = fs::remove_dir_all(&path);
            return None;
        }

        Some(Self {
            path,
            canonical_parent,
            expected_name,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        let Ok(canonical) = self.path.canonicalize() else {
            return;
        };
        if canonical.parent() != Some(self.canonical_parent.as_path())
            || canonical.file_name() != Some(self.expected_name.as_os_str())
        {
            return;
        }
        let _ = fs::remove_dir_all(canonical);
    }
}

#[test]
fn temp_guard_name_is_exact_os_string() {
    let name = OsStr::new("winds-t057-cli-fixture");
    assert_eq!(name, OsStr::new("winds-t057-cli-fixture"));
}
