use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn minimal_cli_proves_workspace_profiles_execution_and_terminal_paths() {
    let root = unique_temp_dir("winds-t057-cli");
    let repo = root.join("repo");
    let other_repo = root.join("other-repo");
    let winds_home = root.join("winds-home");
    init_repo(&repo, "primary");
    init_repo(&other_repo, "other");

    let opened = winds(
        &winds_home,
        ["workspace-open", "--repo", repo.to_str().unwrap()],
    );
    assert_success(&opened);
    let opened_json: Value = serde_json::from_slice(&opened.stdout).unwrap();
    assert_eq!(
        opened_json["canonical_worktree_root"],
        repo.canonicalize().unwrap().to_str().unwrap()
    );
    let workspace_id = opened_json["workspace_id"].as_str().unwrap().to_owned();
    assert!(!workspace_id.is_empty());
    assert!(opened_json["head_oid"].as_str().is_some());

    let reopened = winds(
        &winds_home,
        ["workspace-open", "--repo", repo.to_str().unwrap()],
    );
    assert_success(&reopened);
    let reopened_json: Value = serde_json::from_slice(&reopened.stdout).unwrap();
    assert_eq!(reopened_json["workspace_id"], workspace_id);

    let profiles = winds(&winds_home, ["profiles", "--repo", repo.to_str().unwrap()]);
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
            repo.to_str().unwrap(),
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
            repo.to_str().unwrap(),
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
            other_repo.to_str().unwrap(),
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
            repo.to_str().unwrap(),
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

    remove_owned_temp_dir(&root, "winds-t057-cli-");
}

#[test]
fn workspace_clone_rejects_state_root_inside_destination_before_creation() {
    let root = unique_temp_dir("winds-t057-clone");
    let source = root.join("source");
    init_repo(&source, "source");
    let destination = root.join("clone");
    let nested_home = destination.join(".winds");

    let rejected = Command::new(env!("CARGO_BIN_EXE_winds"))
        .args([
            "workspace-clone",
            "--remote",
            source.to_str().unwrap(),
            "--destination",
            destination.to_str().unwrap(),
            "--home",
            nested_home.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("clone destination and Winds state root must not overlap")
    );
    assert!(!destination.exists());

    let safe_home = root.join("winds-home");
    let cloned = Command::new(env!("CARGO_BIN_EXE_winds"))
        .args([
            "workspace-clone",
            "--remote",
            source.to_str().unwrap(),
            "--destination",
            destination.to_str().unwrap(),
            "--home",
            safe_home.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&cloned);
    let cloned_json: Value = serde_json::from_slice(&cloned.stdout).unwrap();
    assert_eq!(
        cloned_json["workspace"]["canonical_worktree_root"],
        destination.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(
        cloned_json["remote_identity"],
        source.canonicalize().unwrap().to_str().unwrap()
    );

    remove_owned_temp_dir(&root, "winds-t057-clone-");
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

fn remove_owned_temp_dir(path: &Path, prefix: &str) {
    let temp = std::env::temp_dir().canonicalize().unwrap();
    let canonical = path.canonicalize().unwrap();
    assert_eq!(canonical.parent(), Some(temp.as_path()));
    assert!(
        canonical
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .starts_with(prefix)
    );
    fs::remove_dir_all(canonical).unwrap();
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nanos}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
