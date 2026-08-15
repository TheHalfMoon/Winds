#![cfg(unix)]

use serde_json::Value;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SOAK_CYCLES: usize = 100;
static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
#[ignore = "SC-001 pre-release soak; run via the pre-release-soak workflow"]
fn sc_001_runs_100_clean_create_verify_promote_reconcile_cycles() {
    let root = unique_temp_dir("winds-pre-release-soak");

    for cycle in 0..SOAK_CYCLES {
        run_cycle(&root, cycle);
        if (cycle + 1) % 10 == 0 || cycle + 1 == SOAK_CYCLES {
            eprintln!(
                "SC-001 progress: {}/{} cycles passed",
                cycle + 1,
                SOAK_CYCLES
            );
        }
    }

    remove_owned_temp_dir(&root);
    eprintln!(
        "SC-001 PASS: {SOAK_CYCLES}/{SOAK_CYCLES} create/verify/promote/reconcile cycles completed with zero observed primary-checkout mutations"
    );
}

fn run_cycle(root: &Path, cycle: usize) {
    let cycle_root = root.join(format!("cycle-{cycle:03}"));
    let repo = cycle_root.join("repo");
    let winds_home = cycle_root.join("winds-home");
    fs::create_dir_all(&repo).unwrap();

    git(&repo, &["init", "-b", "main"]);
    git(
        &repo,
        &["config", "user.email", "winds-soak@example.invalid"],
    );
    git(&repo, &["config", "user.name", "Winds Soak"]);
    fs::write(repo.join("result.txt"), b"base\n").unwrap();
    git(&repo, &["add", "result.txt"]);
    git(&repo, &["commit", "-m", "base"]);
    let base_oid = git_text(&repo, &["rev-parse", "HEAD"]);

    git(&repo, &["switch", "-c", "candidate"]);
    fs::write(repo.join("result.txt"), b"candidate\n").unwrap();
    git(&repo, &["commit", "-am", "candidate"]);
    let candidate_oid = git_text(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["switch", "main"]);

    let primary_before = PrimarySnapshot::capture(&repo);
    assert!(
        primary_before.status.is_empty(),
        "cycle {cycle}: fixture primary checkout is unexpectedly dirty before Winds runs"
    );

    let verify = winds(
        &winds_home,
        &[
            "verify",
            "--repo",
            repo.to_str().unwrap(),
            "--base",
            &base_oid,
            "--candidate",
            &candidate_oid,
            "--check",
            "test \"$(cat result.txt)\" = candidate",
            "--timeout-secs",
            "5",
        ],
    );
    assert_success(&verify, cycle, "verify");
    let verify_json: Value = serde_json::from_slice(&verify.stdout).unwrap();
    assert_eq!(verify_json["eligibility"], "ELIGIBLE", "cycle {cycle}");
    assert_eq!(verify_json["base_oid"], base_oid, "cycle {cycle}");
    assert_eq!(verify_json["candidate_oid"], candidate_oid, "cycle {cycle}");
    let run_id = verify_json["run_id"].as_str().unwrap().to_owned();
    let worktree = PathBuf::from(verify_json["worktree_path"].as_str().unwrap());

    let promote = winds(
        &winds_home,
        &[
            "promote",
            "--repo",
            repo.to_str().unwrap(),
            "--run",
            &run_id,
        ],
    );
    assert_success(&promote, cycle, "promote");
    let promote_json: Value = serde_json::from_slice(&promote.stdout).unwrap();
    assert_eq!(
        promote_json["authority"], "CALLER_REQUESTED",
        "cycle {cycle}"
    );
    assert_eq!(promote_json["commit_oid"], candidate_oid, "cycle {cycle}");
    assert_eq!(
        git_text(
            &repo,
            &["rev-parse", &format!("refs/heads/winds/selected/{run_id}")],
        ),
        candidate_oid,
        "cycle {cycle}: selected ref drifted"
    );

    let recover = winds(&winds_home, &["recover", "--repo", repo.to_str().unwrap()]);
    assert_success(&recover, cycle, "recover");
    let recover_json: Value = serde_json::from_slice(&recover.stdout).unwrap();
    let runs = recover_json["runs"].as_array().unwrap();
    assert_eq!(
        runs.len(),
        1,
        "cycle {cycle}: recovery returned unexpected run count"
    );
    assert_eq!(runs[0]["run_id"], run_id, "cycle {cycle}");
    assert_eq!(runs[0]["status"], "PRESENT", "cycle {cycle}");

    primary_before.assert_unchanged(&repo, cycle);

    remove_registered_worktree(&repo, &worktree, cycle);
    remove_owned_cycle_dir(root, &cycle_root, cycle);
}

#[derive(Debug)]
struct PrimarySnapshot {
    branch: String,
    head: String,
    status: Vec<u8>,
    index: Vec<u8>,
    tracked_file: Vec<u8>,
}

impl PrimarySnapshot {
    fn capture(repo: &Path) -> Self {
        Self {
            branch: git_text(repo, &["branch", "--show-current"]),
            head: git_text(repo, &["rev-parse", "HEAD"]),
            status: git_bytes(
                repo,
                &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            ),
            index: fs::read(repo.join(".git/index")).unwrap(),
            tracked_file: fs::read(repo.join("result.txt")).unwrap(),
        }
    }

    fn assert_unchanged(&self, repo: &Path, cycle: usize) {
        assert_eq!(
            git_text(repo, &["branch", "--show-current"]),
            self.branch,
            "cycle {cycle}: primary branch changed"
        );
        assert_eq!(
            git_text(repo, &["rev-parse", "HEAD"]),
            self.head,
            "cycle {cycle}: primary HEAD changed"
        );
        assert_eq!(
            git_bytes(
                repo,
                &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
            ),
            self.status,
            "cycle {cycle}: primary status changed"
        );
        assert_eq!(
            fs::read(repo.join(".git/index")).unwrap(),
            self.index,
            "cycle {cycle}: primary index bytes changed"
        );
        assert_eq!(
            fs::read(repo.join("result.txt")).unwrap(),
            self.tracked_file,
            "cycle {cycle}: primary tracked file changed"
        );
    }
}

fn winds(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_HOME", home)
        .output()
        .unwrap()
}

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_command_success(&output, "git");
}

fn git_text(repo: &Path, args: &[&str]) -> String {
    String::from_utf8(git_bytes(repo, args))
        .unwrap()
        .trim()
        .to_owned()
}

fn git_bytes(repo: &Path, args: &[&str]) -> Vec<u8> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_command_success(&output, "git");
    output.stdout
}

fn remove_registered_worktree(repo: &Path, worktree: &Path, cycle: usize) {
    let unlock = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "unlock"])
        .arg(worktree)
        .output()
        .unwrap();
    assert_success(&unlock, cycle, "git worktree unlock");

    let remove = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "remove"])
        .arg(worktree)
        .output()
        .unwrap();
    assert_success(&remove, cycle, "git worktree remove");
}

fn assert_success(output: &Output, cycle: usize, operation: &str) {
    assert!(
        output.status.success(),
        "cycle {cycle}: {operation} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_command_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    loop {
        let attempt = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            env::temp_dir().join(format!("{prefix}-{nanos}-{}-{attempt}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("failed to create exclusive soak root: {error}"),
        }
    }
}

fn remove_owned_cycle_dir(root: &Path, cycle_root: &Path, cycle: usize) {
    let canonical_root = root.canonicalize().unwrap();
    let canonical_cycle = cycle_root.canonicalize().unwrap();
    assert_eq!(
        canonical_cycle.parent(),
        Some(canonical_root.as_path()),
        "cycle {cycle}: cycle root escaped owned soak root"
    );
    let expected_name = format!("cycle-{cycle:03}");
    assert_eq!(
        canonical_cycle.file_name().and_then(|name| name.to_str()),
        Some(expected_name.as_str()),
        "cycle {cycle}: unexpected cycle directory name"
    );
    fs::remove_dir_all(canonical_cycle).unwrap();
}

fn remove_owned_temp_dir(path: &Path) {
    let temp = env::temp_dir().canonicalize().unwrap();
    let canonical = path.canonicalize().unwrap();
    assert_eq!(canonical.parent(), Some(temp.as_path()));
    assert!(
        canonical
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap()
            .starts_with("winds-pre-release-soak-")
    );
    fs::remove_dir_all(canonical).unwrap();
}
