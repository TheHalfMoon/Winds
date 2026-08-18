use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[test]
fn cli_process_start_reconciles_stale_execution_truth_before_display() {
    let Some(temp) = TestTempDir::new("winds-t066-restart") else {
        return;
    };
    let root = temp.path();
    let repo = root.join("repo");
    let winds_home = root.join("winds-home");
    init_repo(&repo);

    let opened = winds(&winds_home, ["workspace-open", "--repo", test_path(&repo)]);
    assert_success(&opened);
    let opened_json: Value = serde_json::from_slice(&opened.stdout).unwrap();
    let workspace_id = opened_json["workspace_id"].as_str().unwrap().to_owned();
    let canonical_repo = repo.canonicalize().unwrap();

    seed_stale_execution_rows(&winds_home, &workspace_id, &canonical_repo);

    let reopened = winds(&winds_home, ["workspace-open", "--repo", test_path(&repo)]);
    assert_success(&reopened);

    let connection = Connection::open(winds_home.join("winds.db")).unwrap();
    assert_execution_state(
        &connection,
        "t066-stale-terminal",
        "OWNERSHIP_LOST",
        None,
        None,
    );
    assert_execution_state(
        &connection,
        "t066-stale-command",
        "OWNERSHIP_LOST",
        None,
        None,
    );
    assert_execution_state(
        &connection,
        "t066-observed-command",
        "EXITED",
        Some(120),
        Some(10),
    );

    let terminal_reason: String = connection
        .query_row(
            "SELECT close_reason FROM terminal_sessions WHERE execution_id = ?1",
            ["t066-stale-terminal"],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(terminal_reason, "OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN");

    assert_event(
        &connection,
        "t066-stale-terminal",
        "TerminalOwnershipLostAfterRestart",
    );
    assert_event(
        &connection,
        "t066-stale-command",
        "ShellCommandOwnershipLostAfterRestart",
    );
    assert_event(&connection, "t066-observed-command", "ShellCommandExited");
    drop(connection);

    let terminal = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-stale-terminal",
        ],
    );
    assert_success(&terminal);
    let terminal_json: Value = serde_json::from_slice(&terminal.stdout).unwrap();
    assert_eq!(terminal_json["status"], "OWNERSHIP_LOST");
    assert!(terminal_json["ended_unix_ms"].is_null());
    assert!(terminal_json["duration_ms"].is_null());
    assert_eq!(
        terminal_json["terminal"]["close_reason"],
        "OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN"
    );

    let command = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-stale-command",
        ],
    );
    assert_success(&command);
    let command_json: Value = serde_json::from_slice(&command.stdout).unwrap();
    assert_eq!(command_json["status"], "OWNERSHIP_LOST");
    assert!(command_json["ended_unix_ms"].is_null());
    assert!(command_json["duration_ms"].is_null());

    let observed = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-observed-command",
        ],
    );
    assert_success(&observed);
    let observed_json: Value = serde_json::from_slice(&observed.stdout).unwrap();
    assert_eq!(observed_json["status"], "EXITED");
    assert_eq!(observed_json["ended_unix_ms"], 120);
    assert_eq!(observed_json["duration_ms"], 10);
    assert_eq!(
        observed_json["shell_command"]["exit_source"],
        "WINDS_OBSERVED"
    );
}

#[test]
fn concurrent_live_owner_is_preserved_and_ambiguous_stale_display_fails_closed() {
    let Some(temp) = TestTempDir::new("winds-t066-concurrent") else {
        return;
    };
    let root = temp.path();
    let repo = root.join("repo");
    let winds_home = root.join("winds-home");
    init_repo(&repo);

    let opened = winds(&winds_home, ["workspace-open", "--repo", test_path(&repo)]);
    assert_success(&opened);
    let opened_json: Value = serde_json::from_slice(&opened.stdout).unwrap();
    let workspace_id = opened_json["workspace_id"].as_str().unwrap().to_owned();
    let canonical_repo = repo.canonicalize().unwrap();

    let (executable, arguments) = long_running_command();
    let arguments_json = serde_json::to_string(&arguments).unwrap();
    let live = Command::new(env!("CARGO_BIN_EXE_winds"))
        .args([
            "run",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-live-command",
            "--executable",
            test_path(&executable),
            "--args-json",
            &arguments_json,
            "--history",
            "disabled",
        ])
        .env("WINDS_HOME", &winds_home)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    wait_for_status(&winds_home, "t066-live-command", "RUNNING");

    let inspected_live = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-live-command",
        ],
    );
    assert_success(&inspected_live);
    let inspected_live_json: Value = serde_json::from_slice(&inspected_live.stdout).unwrap();
    assert_eq!(inspected_live_json["status"], "RUNNING");

    seed_one_stale_shell_command(
        &winds_home,
        &workspace_id,
        &canonical_repo,
        "t066-concurrent-stale",
    );
    let stale_during_live = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-concurrent-stale",
        ],
    );
    assert!(!stale_during_live.status.success());
    assert!(
        String::from_utf8_lossy(&stale_during_live.stderr)
            .contains("refuses to display a falsely-live status")
    );

    let live_output = live.wait_with_output().unwrap();
    assert_success(&live_output);

    let final_live = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-live-command",
        ],
    );
    assert_success(&final_live);
    let final_live_json: Value = serde_json::from_slice(&final_live.stdout).unwrap();
    assert_eq!(final_live_json["status"], "EXITED");

    let stale_after_live = winds(
        &winds_home,
        [
            "execution",
            "--repo",
            test_path(&repo),
            "--execution-id",
            "t066-concurrent-stale",
        ],
    );
    assert_success(&stale_after_live);
    let stale_after_live_json: Value = serde_json::from_slice(&stale_after_live.stdout).unwrap();
    assert_eq!(stale_after_live_json["status"], "OWNERSHIP_LOST");
    assert!(stale_after_live_json["ended_unix_ms"].is_null());
    assert!(stale_after_live_json["duration_ms"].is_null());
}

fn seed_stale_execution_rows(home: &Path, workspace_id: &str, repo: &Path) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .unwrap();
    let tx = connection.transaction().unwrap();
    let execution_domain = execution_domain_json();
    let cwd = test_path(repo);

    tx.execute(
        "INSERT INTO executions(
            execution_id, workspace_id, kind, request_source, execution_domain,
            status, status_source, requested_unix_ms,
            started_unix_ms, ended_unix_ms, duration_ms
         ) VALUES (?1, ?2, 'TERMINAL', 'CALLER_REQUESTED', ?3,
                   'RUNNING', 'WINDS_OBSERVED', 100, 110, NULL, NULL)",
        params!["t066-stale-terminal", workspace_id, execution_domain],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO terminal_sessions(
            execution_id, profile_id, shell_executable, shell_arguments_json,
            requested_cwd, initial_cols, initial_rows, close_reason
         ) VALUES (?1, 't066-profile', 'stale-shell', '[]', ?2, 80, 24, NULL)",
        params!["t066-stale-terminal", cwd],
    )
    .unwrap();

    for execution_id in ["t066-stale-command", "t066-observed-command"] {
        tx.execute(
            "INSERT INTO executions(
                execution_id, workspace_id, kind, request_source, execution_domain,
                status, status_source, requested_unix_ms,
                started_unix_ms, ended_unix_ms, duration_ms
             ) VALUES (?1, ?2, 'SHELL_COMMAND', 'CALLER_REQUESTED', ?3,
                       'RUNNING', 'WINDS_OBSERVED', 100, 110, NULL, NULL)",
            params![execution_id, workspace_id, execution_domain],
        )
        .unwrap();
    }
    tx.execute(
        "INSERT INTO shell_commands(
            execution_id, executable, arguments_json, command_source,
            requested_cwd, cwd_source, exit_code, exit_source, observed_end_unix_ms
         ) VALUES (?1, 'stale-command', '[]', 'CALLER_REQUESTED', ?2,
                   'CALLER_REQUESTED', NULL, NULL, NULL)",
        params!["t066-stale-command", cwd],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO shell_commands(
            execution_id, executable, arguments_json, command_source,
            requested_cwd, cwd_source, exit_code, exit_source, observed_end_unix_ms
         ) VALUES (?1, 'observed-command', '[]', 'CALLER_REQUESTED', ?2,
                   'CALLER_REQUESTED', 0, 'WINDS_OBSERVED', 120)",
        params!["t066-observed-command", cwd],
    )
    .unwrap();
    tx.commit().unwrap();
}

fn seed_one_stale_shell_command(home: &Path, workspace_id: &str, repo: &Path, execution_id: &str) {
    let mut connection = Connection::open(home.join("winds.db")).unwrap();
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .unwrap();
    let tx = connection.transaction().unwrap();
    tx.execute(
        "INSERT INTO executions(
            execution_id, workspace_id, kind, request_source, execution_domain,
            status, status_source, requested_unix_ms,
            started_unix_ms, ended_unix_ms, duration_ms
         ) VALUES (?1, ?2, 'SHELL_COMMAND', 'CALLER_REQUESTED', ?3,
                   'RUNNING', 'WINDS_OBSERVED', 100, 110, NULL, NULL)",
        params![execution_id, workspace_id, execution_domain_json()],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO shell_commands(
            execution_id, executable, arguments_json, command_source,
            requested_cwd, cwd_source, exit_code, exit_source, observed_end_unix_ms
         ) VALUES (?1, 'stale-command', '[]', 'CALLER_REQUESTED', ?2,
                   'CALLER_REQUESTED', NULL, NULL, NULL)",
        params![execution_id, test_path(repo)],
    )
    .unwrap();
    tx.commit().unwrap();
}

fn execution_domain_json() -> String {
    json!({
        "kind": "NATIVE_HOST",
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
    .to_string()
}

fn wait_for_status(home: &Path, execution_id: &str, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let connection = Connection::open(home.join("winds.db")).unwrap();
        let status = connection
            .query_row(
                "SELECT status FROM executions WHERE execution_id = ?1",
                [execution_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .unwrap();
        if status.as_deref() == Some(expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for execution {execution_id} to reach {expected}; last status: {status:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn long_running_command() -> (PathBuf, Vec<String>) {
    (
        PathBuf::from("/bin/sh"),
        vec!["-c".to_owned(), "sleep 5".to_owned()],
    )
}

#[cfg(windows)]
fn long_running_command() -> (PathBuf, Vec<String>) {
    (
        PathBuf::from(std::env::var_os("COMSPEC").expect("COMSPEC on Windows CI")),
        vec![
            "/D".to_owned(),
            "/C".to_owned(),
            "ping -n 6 127.0.0.1 >NUL".to_owned(),
        ],
    )
}

fn assert_execution_state(
    connection: &Connection,
    execution_id: &str,
    expected_status: &str,
    expected_end: Option<i64>,
    expected_duration: Option<i64>,
) {
    let row = connection
        .query_row(
            "SELECT status, ended_unix_ms, duration_ms
             FROM executions WHERE execution_id = ?1",
            [execution_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(row.0, expected_status);
    assert_eq!(row.1, expected_end);
    assert_eq!(row.2, expected_duration);
}

fn assert_event(connection: &Connection, execution_id: &str, kind: &str) {
    let found = connection
        .query_row(
            "SELECT kind FROM execution_events
             WHERE execution_id = ?1 AND kind = ?2 LIMIT 1",
            params![execution_id, kind],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .unwrap();
    assert_eq!(found.as_deref(), Some(kind));
}

fn init_repo(path: &Path) {
    fs::create_dir_all(path).unwrap();
    git(path, ["init", "-b", "main"]);
    git(path, ["config", "user.email", "winds-test@example.invalid"]);
    git(path, ["config", "user.name", "Winds Test"]);
    fs::write(path.join("file.txt"), b"t066\n").unwrap();
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
        "T066 CLI integration fixture root is validated as UTF-8; derived ASCII child paths must remain UTF-8",
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
        canonical_parent.to_str()?;

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        let expected_name = OsString::from(format!("{prefix}-{nanos}-{}", std::process::id()));
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
