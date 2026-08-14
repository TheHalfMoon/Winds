use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn verifies_blocks_and_promotes_without_touching_primary_checkout() {
    let root = unique_temp_dir("winds-walking-skeleton");
    let repo = root.join("repo");
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
    let run_id = passing_json["run_id"].as_str().unwrap().to_owned();

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
    let selected = git_text(
        &repo,
        ["rev-parse", &format!("refs/heads/winds/selected/{run_id}")],
    );
    assert_eq!(selected, passing_oid);

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
    assert_success(&failing);
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
    assert_success(&mutating);
    let mutating_json: Value = serde_json::from_slice(&mutating.stdout).unwrap();
    assert_eq!(mutating_json["check"]["status"], "PASS");
    assert_eq!(mutating_json["eligibility"], "BLOCKED");
    assert!(mutating_json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning.as_str().unwrap().contains("mutated candidate")));

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

    let _ = fs::remove_dir_all(root);
}

fn winds<const N: usize>(home: &Path, args: [&str; N]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_HOME", home)
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

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nanos}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
