use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(windows)]
#[test]
fn native_windows_refuses_authoritative_required_checks_without_mutation() {
    let root = unique_temp_dir("winds-walking-skeleton");
    let repo = root.join("repo with spaces");
    let winds_home = root.join("winds-home");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, ["init", "-b", "main"]);
    git(
        &repo,
        ["config", "user.email", "winds-test@example.invalid"],
    );
    git(&repo, ["config", "user.name", "Winds Test"]);
    fs::write(repo.join("result.txt"), "base\n").unwrap();
    git(&repo, ["add", "result.txt"]);
    git(&repo, ["commit", "-m", "base"]);
    let head = git_text(&repo, ["rev-parse", "HEAD"]);

    let unsupported = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &head,
            "--candidate",
            &head,
            "--check",
            "true",
        ],
    );
    assert!(!unsupported.status.success());
    assert!(
        String::from_utf8_lossy(&unsupported.stderr)
            .contains("authoritative required-check execution is unsupported on native Windows"),
        "unexpected native-Windows verification refusal: {}",
        String::from_utf8_lossy(&unsupported.stderr)
    );
    assert!(!winds_home.exists());
    assert_eq!(git_text(&repo, ["branch", "--show-current"]), "main");
    assert_eq!(git_text(&repo, ["rev-parse", "HEAD"]), head);
    assert!(git_bytes(&repo, ["status", "--porcelain=v1", "-z"]).is_empty());

    remove_owned_temp_dir(&root);
}

#[cfg(unix)]
#[test]
fn verifies_blocks_and_promotes_without_touching_primary_checkout() {
    let root = unique_temp_dir("winds-walking-skeleton");
    let repo = root.join("repo with spaces");
    let winds_home = root.join("winds-home");
    fs::create_dir_all(&repo).unwrap();

    git(&repo, ["init", "-b", "main"]);
    git(
        &repo,
        ["config", "user.email", "winds-test@example.invalid"],
    );
    git(&repo, ["config", "user.name", "Winds Test"]);
    fs::write(repo.join("result.txt"), "base\n").unwrap();
    git(&repo, ["add", "result.txt"]);
    git(&repo, ["commit", "-m", "base"]);
    let base_oid = git_text(&repo, ["rev-parse", "HEAD"]);

    git(&repo, ["switch", "-c", "candidate-pass"]);
    fs::write(repo.join("result.txt"), "ok\n").unwrap();
    git(&repo, ["commit", "-am", "passing candidate"]);
    let passing_oid = git_text(&repo, ["rev-parse", "HEAD"]);

    git(&repo, ["switch", "main"]);
    git(&repo, ["switch", "-c", "candidate-fail"]);
    fs::write(repo.join("result.txt"), "bad\n").unwrap();
    git(&repo, ["commit", "-am", "failing candidate"]);
    let failing_oid = git_text(&repo, ["rev-parse", "HEAD"]);
    git(&repo, ["switch", "main"]);

    let primary_branch_before = git_text(&repo, ["branch", "--show-current"]);
    let primary_head_before = git_text(&repo, ["rev-parse", "HEAD"]);
    let primary_content_before = fs::read(repo.join("result.txt")).unwrap();

    let repo_local_home = repo.join(".winds-local");
    let repository_local_state = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "true",
            "--home",
            repo_local_home.to_str().unwrap(),
        ],
    );
    assert!(!repository_local_state.status.success());
    assert!(
        String::from_utf8_lossy(&repository_local_state.stderr)
            .contains("outside the source checkout")
    );
    assert!(!repo_local_home.exists());
    assert!(git_bytes(&repo, ["status", "--porcelain=v1", "-z"]).is_empty());

    git(&repo, ["config", "status.showUntrackedFiles", "no"]);
    fs::write(repo.join("dirty.tmp"), "dirty\n").unwrap();
    let dirty_primary = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "true",
        ],
    );
    assert!(!dirty_primary.status.success());
    assert!(String::from_utf8_lossy(&dirty_primary.stderr).contains("primary checkout is dirty"));
    fs::remove_file(repo.join("dirty.tmp")).unwrap();
    git(&repo, ["config", "--unset", "status.showUntrackedFiles"]);

    let hostile_git_dir = root.join("hostile-git-dir");
    fs::create_dir_all(&hostile_git_dir).unwrap();
    let isolated_git_context = winds_with_env(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "true",
        ],
        "GIT_DIR",
        hostile_git_dir.to_str().unwrap(),
    );
    assert_success(&isolated_git_context);
    let isolated_json: Value = serde_json::from_slice(&isolated_git_context.stdout).unwrap();
    assert_eq!(isolated_json["eligibility"], "ELIGIBLE");
    assert_eq!(isolated_json["candidate_oid"], passing_oid);

    let passing = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "test \"$(cat result.txt)\" = ok",
            "--timeout-secs",
            "5",
        ],
    );
    assert_success(&passing);
    let passing_json: Value = serde_json::from_slice(&passing.stdout).unwrap();
    assert_eq!(passing_json["eligibility"], "ELIGIBLE");
    assert_eq!(passing_json["base_oid"], base_oid);
    assert_eq!(passing_json["candidate_oid"], passing_oid);
    assert!(passing_json.get("run_branch").is_none());
    let run_id = passing_json["run_id"].as_str().unwrap().to_owned();
    let passing_worktree = PathBuf::from(passing_json["worktree_path"].as_str().unwrap());
    assert!(git_text(&passing_worktree, ["branch", "--show-current"]).is_empty());

    let promotion = winds(
        &winds_home,
        [
            "promote",
            "--repo",
            repo.to_str().unwrap(),
            "--run",
            &run_id,
        ],
    );
    assert_success(&promotion);
    let promotion_json: Value = serde_json::from_slice(&promotion.stdout).unwrap();
    assert_eq!(promotion_json["authority"], "CALLER_REQUESTED");
    let selected = git_text(
        &repo,
        ["rev-parse", &format!("refs/heads/winds/selected/{run_id}")],
    );
    assert_eq!(selected, passing_oid);

    let db = rusqlite::Connection::open(winds_home.join("winds.db")).unwrap();
    let decision_authority: String = db
        .query_row(
            "SELECT authority FROM events WHERE run_id = ?1 AND kind = 'DecisionRecorded' ORDER BY event_id DESC LIMIT 1",
            rusqlite::params![&run_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(decision_authority, "CALLER_REQUESTED");
    let recheck_payload: String = db
        .query_row(
            "SELECT payload_json FROM events WHERE run_id = ?1 AND kind = 'PromotionRecheckObserved' ORDER BY event_id DESC LIMIT 1",
            rusqlite::params![&run_id],
            |row| row.get(0),
        )
        .unwrap();
    let recheck_json: Value = serde_json::from_str(&recheck_payload).unwrap();
    assert_eq!(recheck_json["status"], "PASS");
    let recheck_stdout_relative = recheck_json["stdout"]["relative_path"]
        .as_str()
        .unwrap()
        .to_owned();
    drop(db);

    fs::write(winds_home.join(recheck_stdout_relative), b"corrupt").unwrap();
    let corrupt_blob_retry = winds(
        &winds_home,
        [
            "promote",
            "--repo",
            repo.to_str().unwrap(),
            "--run",
            &run_id,
        ],
    );
    assert!(!corrupt_blob_retry.status.success());
    assert!(
        String::from_utf8_lossy(&corrupt_blob_retry.stderr)
            .contains("existing evidence blob does not match")
    );

    let failing = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &failing_oid,
            "--check",
            "test \"$(cat result.txt)\" = ok",
            "--timeout-secs",
            "5",
        ],
    );
    assert!(!failing.status.success());
    let failing_json: Value = serde_json::from_slice(&failing.stdout).unwrap();
    assert_eq!(failing_json["eligibility"], "BLOCKED");
    assert_eq!(failing_json["check"]["status"], "FAIL");

    let mutating = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "printf 'changed\\n' > result.txt",
            "--timeout-secs",
            "5",
        ],
    );
    assert!(!mutating.status.success());
    let mutating_json: Value = serde_json::from_slice(&mutating.stdout).unwrap();
    assert_eq!(mutating_json["check"]["status"], "PASS");
    assert_eq!(mutating_json["eligibility"], "BLOCKED");
    assert!(
        mutating_json["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning.as_str().unwrap().contains("mutated candidate"))
    );

    let timeout_started = Instant::now();
    let timed_out = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "sleep 5",
            "--timeout-secs",
            "1",
        ],
    );
    assert!(!timed_out.status.success());
    assert!(timeout_started.elapsed() < Duration::from_secs(3));
    let timeout_json: Value = serde_json::from_slice(&timed_out.stdout).unwrap();
    assert_eq!(timeout_json["check"]["status"], "TIMEOUT");
    assert_eq!(timeout_json["eligibility"], "BLOCKED");

    let stale = winds(
        &winds_home,
        [
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &passing_oid,
            "--check",
            "true",
        ],
    );
    assert_success(&stale);
    let stale_json: Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(stale_json["eligibility"], "ELIGIBLE");
    let stale_run_id = stale_json["run_id"].as_str().unwrap();
    let stale_worktree = PathBuf::from(stale_json["worktree_path"].as_str().unwrap());
    fs::write(stale_worktree.join("manual.txt"), "manual edit\n").unwrap();
    let stale_promotion = winds(
        &winds_home,
        [
            "promote",
            "--repo",
            repo.to_str().unwrap(),
            "--run",
            stale_run_id,
        ],
    );
    assert!(!stale_promotion.status.success());
    assert!(
        String::from_utf8_lossy(&stale_promotion.stderr).contains("changed after verification")
    );

    let recovery = winds(&winds_home, ["recover", "--repo", repo.to_str().unwrap()]);
    assert!(!recovery.status.success());
    let recovery_json: Value = serde_json::from_slice(&recovery.stdout).unwrap();
    let stale_recovery = recovery_json["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run["run_id"] == stale_run_id)
        .unwrap();
    assert_eq!(stale_recovery["status"], "MANUAL_RECOVERY_REQUIRED");
    assert!(stale_worktree.join("manual.txt").exists());

    assert_eq!(
        git_text(&repo, ["branch", "--show-current"]),
        primary_branch_before
    );
    assert_eq!(git_text(&repo, ["rev-parse", "HEAD"]), primary_head_before);
    assert_eq!(
        fs::read(repo.join("result.txt")).unwrap(),
        primary_content_before
    );
    assert!(git_bytes(&repo, ["status", "--porcelain=v1", "-z"]).is_empty());

    remove_owned_temp_dir(&root);
}

fn winds<const N: usize>(home: &Path, args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_HOME", home)
        .output()
        .unwrap()
}

#[cfg(unix)]
fn winds_with_env<const N: usize>(home: &Path, args: [&str; N], key: &str, value: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_HOME", home)
        .env(key, value)
        .output()
        .unwrap()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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

fn git_text<const N: usize>(repo: &Path, args: [&str; N]) -> String {
    String::from_utf8(git_bytes(repo, args))
        .unwrap()
        .trim()
        .to_owned()
}

fn git_bytes<const N: usize>(repo: &Path, args: [&str; N]) -> Vec<u8> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_success(&output);
    output.stdout
}

fn remove_owned_temp_dir(path: &Path) {
    let temp = std::env::temp_dir().canonicalize().unwrap();
    let canonical = path.canonicalize().unwrap();
    assert_eq!(canonical.parent(), Some(temp.as_path()));
    assert!(
        canonical
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .starts_with("winds-walking-skeleton-")
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
