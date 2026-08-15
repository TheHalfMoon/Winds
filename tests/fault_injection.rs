#![cfg(unix)]

use rusqlite::Connection;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[test]
fn partial_worktree_creation_remains_manual_recovery_required() {
    let fixture = Fixture::new("winds-partial-worktree");
    let shim_dir = fixture.root.join("shim-fail-lock");
    install_fail_worktree_lock_shim(&shim_dir);

    let verify = fixture.verify(Some(&shim_dir));
    assert!(
        !verify.status.success(),
        "fault-injected verify unexpectedly passed"
    );

    let db = open_db(&fixture.home);
    let (run_id, worktree_path, state) = only_run(&db);
    assert_eq!(state, "PROVISIONING");
    let worktree = PathBuf::from(&worktree_path);
    assert!(
        worktree.exists(),
        "git worktree add did not partially succeed"
    );
    assert!(
        fixture.git_worktree_paths().contains(&worktree),
        "partially provisioned worktree is not registered in Git inventory"
    );

    assert_manual_recovery(&fixture, &run_id);
    assert_eq!(event_count(&db, &run_id, "RecoveryRequired"), 1);

    drop(db);
    fixture.remove_worktree(&worktree, false);
    fixture.cleanup();
}

#[test]
fn failed_workspace_ready_db_write_retains_provisioning_state() {
    let fixture = Fixture::new("winds-ready-write-failure");
    fixture.initialize_store();

    let db = open_db(&fixture.home);
    db.execute_batch(
        r#"
        CREATE TRIGGER inject_workspace_ready_failure
        BEFORE UPDATE OF state ON candidate_runs
        WHEN NEW.state = 'READY'
        BEGIN
          SELECT RAISE(ABORT, 'injected workspace-ready write failure');
        END;
        "#,
    )
    .unwrap();
    drop(db);

    let verify = fixture.verify(None);
    assert!(
        !verify.status.success(),
        "fault-injected verify unexpectedly passed"
    );

    let db = open_db(&fixture.home);
    let (run_id, worktree_path, state) = only_run(&db);
    assert_eq!(state, "PROVISIONING");
    assert_eq!(event_count(&db, &run_id, "WorkspaceReady"), 0);
    let worktree = PathBuf::from(&worktree_path);
    assert!(worktree.exists());
    assert!(fixture.git_worktree_paths().contains(&worktree));

    assert_manual_recovery(&fixture, &run_id);
    assert_eq!(event_count(&db, &run_id, "RecoveryRequired"), 1);

    drop(db);
    fixture.remove_worktree(&worktree, true);
    fixture.cleanup();
}

#[test]
fn interrupted_promotion_db_transition_is_retryable_without_ref_drift() {
    let fixture = Fixture::new("winds-promotion-transition");
    let source_branch_before = fixture.git_text(&["symbolic-ref", "--short", "HEAD"]);
    let source_status_before =
        fixture.git_text(&["status", "--porcelain=v1", "--untracked-files=all"]);

    let verify = fixture.verify(None);
    assert_success(&verify);
    let report: Value = serde_json::from_slice(&verify.stdout).unwrap();
    let run_id = report["run_id"].as_str().unwrap().to_owned();
    let candidate_oid = report["candidate_oid"].as_str().unwrap().to_owned();
    let worktree = PathBuf::from(report["worktree_path"].as_str().unwrap());
    let selected_ref = format!("refs/heads/winds/selected/{run_id}");

    let db = open_db(&fixture.home);
    db.execute_batch(
        r#"
        CREATE TRIGGER inject_promotion_write_failure
        BEFORE INSERT ON promotions
        BEGIN
          SELECT RAISE(ABORT, 'injected promotion write failure');
        END;
        "#,
    )
    .unwrap();
    drop(db);

    let first_promote = fixture.promote(&run_id);
    assert!(
        !first_promote.status.success(),
        "fault-injected promotion unexpectedly passed"
    );
    assert_eq!(
        fixture.git_text(&["rev-parse", &selected_ref]),
        candidate_oid
    );

    let db = open_db(&fixture.home);
    assert_eq!(promotion_count(&db, &run_id), 0);
    assert_eq!(event_count(&db, &run_id, "DecisionRecorded"), 0);
    assert_eq!(event_count(&db, &run_id, "PromotionCreated"), 0);
    assert_eq!(event_count(&db, &run_id, "PromotionRecheckObserved"), 1);
    db.execute_batch("DROP TRIGGER inject_promotion_write_failure;")
        .unwrap();
    drop(db);

    let retry = fixture.promote(&run_id);
    assert_success(&retry);
    assert_eq!(
        fixture.git_text(&["rev-parse", &selected_ref]),
        candidate_oid
    );

    let db = open_db(&fixture.home);
    assert_eq!(promotion_count(&db, &run_id), 1);
    assert_eq!(event_count(&db, &run_id, "DecisionRecorded"), 1);
    assert_eq!(event_count(&db, &run_id, "PromotionCreated"), 1);
    assert_eq!(event_count(&db, &run_id, "PromotionRecheckObserved"), 2);
    drop(db);

    assert_eq!(
        fixture.git_text(&["symbolic-ref", "--short", "HEAD"]),
        source_branch_before
    );
    assert_eq!(
        fixture.git_text(&["status", "--porcelain=v1", "--untracked-files=all"]),
        source_status_before
    );

    fixture.remove_worktree(&worktree, true);
    fixture.cleanup();
}

struct Fixture {
    root: PathBuf,
    repo: PathBuf,
    home: PathBuf,
    real_git: PathBuf,
    base_oid: String,
    candidate_oid: String,
}

impl Fixture {
    fn new(prefix: &str) -> Self {
        let root = unique_temp_dir(prefix);
        let repo = root.join("repo");
        let home = root.join("winds-home");
        let real_git = real_git_path();
        fs::create_dir_all(&repo).unwrap();

        git(&real_git, &repo, &["init", "-b", "main"]);
        git(
            &real_git,
            &repo,
            &["config", "user.email", "winds-test@example.invalid"],
        );
        git(&real_git, &repo, &["config", "user.name", "Winds Test"]);
        fs::write(repo.join("result.txt"), "base\n").unwrap();
        git(&real_git, &repo, &["add", "result.txt"]);
        git(&real_git, &repo, &["commit", "-m", "base"]);
        let base_oid = git_text(&real_git, &repo, &["rev-parse", "HEAD"]);

        git(&real_git, &repo, &["switch", "-c", "candidate"]);
        fs::write(repo.join("result.txt"), "candidate\n").unwrap();
        git(&real_git, &repo, &["commit", "-am", "candidate"]);
        let candidate_oid = git_text(&real_git, &repo, &["rev-parse", "HEAD"]);
        git(&real_git, &repo, &["switch", "main"]);

        Self {
            root,
            repo,
            home,
            real_git,
            base_oid,
            candidate_oid,
        }
    }

    fn initialize_store(&self) {
        let output = winds(&[
            "recover",
            "--repo",
            self.repo.to_str().unwrap(),
            "--home",
            self.home.to_str().unwrap(),
        ]);
        assert_success(&output);
    }

    fn verify(&self, shim_dir: Option<&Path>) -> Output {
        let args = [
            "verify",
            "--repo",
            self.repo.to_str().unwrap(),
            "--base",
            self.base_oid.as_str(),
            "--candidate",
            self.candidate_oid.as_str(),
            "--check",
            "true",
            "--timeout-secs",
            "5",
            "--home",
            self.home.to_str().unwrap(),
        ];
        match shim_dir {
            Some(shim_dir) => winds_with_git_shim(&args, shim_dir, &self.real_git),
            None => winds(&args),
        }
    }

    fn promote(&self, run_id: &str) -> Output {
        winds(&[
            "promote",
            "--repo",
            self.repo.to_str().unwrap(),
            "--run",
            run_id,
            "--home",
            self.home.to_str().unwrap(),
        ])
    }

    fn git_text(&self, args: &[&str]) -> String {
        git_text(&self.real_git, &self.repo, args)
    }

    fn git_worktree_paths(&self) -> Vec<PathBuf> {
        let output = Command::new(&self.real_git)
            .arg("-C")
            .arg(&self.repo)
            .args(["worktree", "list", "--porcelain", "-z"])
            .output()
            .unwrap();
        assert_success(&output);
        String::from_utf8(output.stdout)
            .unwrap()
            .split('\0')
            .filter_map(|field| field.strip_prefix("worktree "))
            .map(PathBuf::from)
            .collect()
    }

    fn remove_worktree(&self, worktree: &Path, unlock_first: bool) {
        if unlock_first {
            let unlock = Command::new(&self.real_git)
                .arg("-C")
                .arg(&self.repo)
                .args(["worktree", "unlock"])
                .arg(worktree)
                .output()
                .unwrap();
            assert_success(&unlock);
        }
        let remove = Command::new(&self.real_git)
            .arg("-C")
            .arg(&self.repo)
            .args(["worktree", "remove"])
            .arg(worktree)
            .output()
            .unwrap();
        assert_success(&remove);
    }

    fn cleanup(&self) {
        remove_owned_temp_dir(&self.root);
    }
}

fn assert_manual_recovery(fixture: &Fixture, run_id: &str) {
    let recover = winds(&[
        "recover",
        "--repo",
        fixture.repo.to_str().unwrap(),
        "--home",
        fixture.home.to_str().unwrap(),
    ]);
    assert!(!recover.status.success());
    let json: Value = serde_json::from_slice(&recover.stdout).unwrap();
    assert!(
        json["runs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|run| run["run_id"] == run_id && run["status"] == "MANUAL_RECOVERY_REQUIRED")
    );
}

fn open_db(home: &Path) -> Connection {
    Connection::open(home.join("winds.db")).unwrap()
}

fn only_run(db: &Connection) -> (String, String, String) {
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM candidate_runs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1, "fault fixture must contain exactly one candidate run");
    db.query_row(
        "SELECT run_id, worktree_path, state FROM candidate_runs",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .unwrap()
}

fn promotion_count(db: &Connection, run_id: &str) -> i64 {
    db.query_row(
        "SELECT COUNT(*) FROM promotions WHERE run_id = ?1",
        [run_id],
        |row| row.get(0),
    )
    .unwrap()
}

fn event_count(db: &Connection, run_id: &str, kind: &str) -> i64 {
    db.query_row(
        "SELECT COUNT(*) FROM events WHERE run_id = ?1 AND kind = ?2",
        [run_id, kind],
        |row| row.get(0),
    )
    .unwrap()
}

fn install_fail_worktree_lock_shim(shim_dir: &Path) {
    fs::create_dir_all(shim_dir).unwrap();
    let shim = shim_dir.join("git");
    fs::write(
        &shim,
        r#"#!/bin/sh
is_fault_target() {
    while [ "$#" -gt 0 ]; do
        case "$1" in
            -c|-C)
                [ "$#" -ge 2 ] || return 1
                shift 2
                ;;
            *)
                break
                ;;
        esac
    done
    [ "${1-}" = "worktree" ] && [ "${2-}" = "lock" ]
}

if is_fault_target "$@"; then
    echo "injected worktree lock failure" >&2
    exit 86
fi
exec "$WINDS_REAL_GIT" "$@"
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&shim).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(shim, permissions).unwrap();
}

fn winds(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .output()
        .unwrap()
}

fn winds_with_git_shim(args: &[&str], shim_dir: &Path, real_git: &Path) -> Output {
    let mut paths = vec![shim_dir.to_path_buf()];
    if let Some(current_path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&current_path));
    }
    let path = env::join_paths(paths).unwrap();
    Command::new(env!("CARGO_BIN_EXE_winds"))
        .args(args)
        .env("WINDS_REAL_GIT", real_git)
        .env("PATH", path)
        .output()
        .unwrap()
}

fn real_git_path() -> PathBuf {
    let output = Command::new("sh")
        .args(["-c", "command -v git"])
        .output()
        .unwrap();
    assert_success(&output);
    PathBuf::from(String::from_utf8(output.stdout).unwrap().trim())
}

fn git(real_git: &Path, repo: &Path, args: &[&str]) {
    let output = Command::new(real_git)
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_success(&output);
}

fn git_text(real_git: &Path, repo: &Path, args: &[&str]) -> String {
    let output = Command::new(real_git)
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert_success(&output);
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
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
            Err(error) => panic!("failed to create exclusive fixture root: {error}"),
        }
    }
}

fn remove_owned_temp_dir(path: &Path) {
    let temp = env::temp_dir().canonicalize().unwrap();
    let canonical = path.canonicalize().unwrap();
    assert_eq!(canonical.parent(), Some(temp.as_path()));
    let name = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();
    assert!(name.starts_with("winds-"));
    fs::remove_dir_all(canonical).unwrap();
}
