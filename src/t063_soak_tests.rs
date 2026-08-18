use crate::command::history::SessionHistoryPolicy;
use crate::domain::{ExecutionStatus, FactSource, TerminalCloseReason};
use crate::execution::{LocalTerminalHistory, TerminalExecution, reconcile_terminal_executions_after_restart};
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::store::{NewWorkspace, Store};
use rusqlite::Connection;
use std::fs;
use std::io::{Read, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
const CYCLES: usize = 100;
const TRANSCRIPT_QUOTA: usize = 512;
const TOTAL_HISTORY_QUOTA: u64 = 64 * 1024;
const OUTPUT_LIMIT: usize = 64 * 1024;
const DEFAULT_SIZE: TerminalSize = TerminalSize { rows: 24, cols: 80 };
#[cfg(windows)]
const CURSOR_POSITION_QUERY: &[u8] = b"\x1b[6n";
#[cfg(windows)]
const TEST_CURSOR_POSITION_RESPONSE: &[u8] = b"\x1b[1;1R";

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let temp_root = std::env::temp_dir()
            .canonicalize()
            .expect("supported T063 CI hosts must provide a canonical temporary directory");
        let path = temp_root.join(format!(
            "winds-t063-soak-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn workspace(&self) -> PathBuf {
        self.0.join("workspace")
    }

    fn state(&self) -> PathBuf {
        self.0.join("state")
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let canonical_temp = match std::env::temp_dir().canonicalize() {
            Ok(path) => path,
            Err(_) => return,
        };
        let canonical_root = match self.0.canonicalize() {
            Ok(path) => path,
            Err(_) => return,
        };
        let owned_name = canonical_root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("winds-t063-soak-"));
        if owned_name && canonical_root.parent() == Some(canonical_temp.as_path()) {
            let _ = fs::remove_dir_all(canonical_root);
        }
    }
}

enum OutputEvent {
    Chunk(Vec<u8>),
    Error(String),
    Eof,
}

struct OutputPump {
    receiver: Receiver<OutputEvent>,
    handle: Option<JoinHandle<()>>,
    observed: Vec<u8>,
}

impl OutputPump {
    fn start(mut reader: Box<dyn Read + Send>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        let _ = sender.send(OutputEvent::Eof);
                        return;
                    }
                    Ok(count) => {
                        if sender
                            .send(OutputEvent::Chunk(buffer[..count].to_vec()))
                            .is_err()
                        {
                            return;
                        }
                    }
                    #[cfg(unix)]
                    Err(error) if error.raw_os_error() == Some(libc::EIO) => {
                        let _ = sender.send(OutputEvent::Eof);
                        return;
                    }
                    Err(error) => {
                        let _ = sender.send(OutputEvent::Error(error.to_string()));
                        return;
                    }
                }
            }
        });
        Self {
            receiver,
            handle: Some(handle),
            observed: Vec::new(),
        }
    }

    fn wait_for(&mut self, needle: &[u8], cycle: usize) {
        if contains_bytes(&self.observed, needle) {
            return;
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(OutputEvent::Chunk(chunk)) => {
                    self.push(&chunk, cycle);
                    if contains_bytes(&self.observed, needle) {
                        return;
                    }
                }
                Ok(OutputEvent::Error(error)) => {
                    panic!("T063 cycle {cycle} output reader failed: {error}")
                }
                Ok(OutputEvent::Eof) => break,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!(
            "T063 cycle {cycle} timed out waiting for output marker {:?}; observed {:?}",
            String::from_utf8_lossy(needle),
            String::from_utf8_lossy(&self.observed)
        );
    }

    fn drain_to_eof(&mut self, cycle: usize) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                Instant::now() < deadline,
                "T063 cycle {cycle} timed out draining terminal output"
            );
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(OutputEvent::Chunk(chunk)) => self.push(&chunk, cycle),
                Ok(OutputEvent::Error(error)) => {
                    panic!("T063 cycle {cycle} output reader failed: {error}")
                }
                Ok(OutputEvent::Eof) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => continue,
            }
        }
        if let Some(handle) = self.handle.take() {
            handle.join().expect("T063 output reader thread panicked");
        }
    }

    fn push(&mut self, chunk: &[u8], cycle: usize) {
        self.observed.extend_from_slice(chunk);
        assert!(
            self.observed.len() <= OUTPUT_LIMIT,
            "T063 cycle {cycle} terminal output exceeded the test observation bound"
        );
    }
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|window| window == needle)
}

fn utf8_path(path: &Path, label: &str) -> String {
    path.to_str()
        .unwrap_or_else(|| panic!("T063 {label} must be UTF-8 on supported CI: {}", path.display()))
        .to_owned()
}

fn store_with_workspace(root: &TestRoot) -> Store {
    fs::create_dir(root.workspace()).unwrap();
    let workspace = root.workspace().canonicalize().unwrap();
    let git_common_dir = workspace.join(".git");
    fs::create_dir(&git_common_dir).unwrap();
    let workspace_text = utf8_path(&workspace, "workspace path");
    let git_common_text = utf8_path(&git_common_dir, "Git common path");
    let store = Store::open(&root.state()).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: &workspace_text,
                git_common_dir: &git_common_text,
            },
            1,
        )
        .unwrap();
    store
}

fn native_profile() -> ShellProfile {
    #[cfg(unix)]
    let executable = "/bin/sh".to_owned();
    #[cfg(windows)]
    let executable = std::env::var("COMSPEC").expect("Windows T063 CI must provide COMSPEC");

    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: "unused-worktree".to_owned(),
        git_common_dir: "unused-git-common".to_owned(),
        shell_candidates: vec![executable.clone()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| {
            #[cfg(unix)]
            {
                profile.executable == executable
            }
            #[cfg(windows)]
            {
                profile.executable.eq_ignore_ascii_case(&executable)
            }
        })
        .unwrap_or_else(|| panic!("T063 native shell was not discoverable: {executable}"))
}

#[cfg(windows)]
fn complete_headless_terminal_startup(execution: &mut TerminalExecution<'_>, output: &mut OutputPump, cycle: usize) {
    output.wait_for(CURSOR_POSITION_QUERY, cycle);
    execution
        .send_input(TEST_CURSOR_POSITION_RESPONSE)
        .expect("T063 headless ConPTY fixture must answer cursor-position query");
}

#[cfg(unix)]
fn complete_headless_terminal_startup(
    _execution: &mut TerminalExecution<'_>,
    _output: &mut OutputPump,
    _cycle: usize,
) {
}

#[cfg(unix)]
fn ready_command(cycle: usize) -> Vec<u8> {
    format!("w=WINDS_T063_READY_; printf '%s%s\\n' \"$w\" '{cycle}'\n").into_bytes()
}

#[cfg(windows)]
fn ready_command(cycle: usize) -> Vec<u8> {
    format!("set \"W=WINDS_T063_READY_\"\r\necho %W%{cycle}\r\n").into_bytes()
}

#[cfg(unix)]
fn finish_command(cycle: usize) -> Vec<u8> {
    let payload = "x".repeat(2048);
    format!(
        "printf '%s\\n' '{payload}'; w=WINDS_T063_DONE_; printf '%s%s\\n' \"$w\" '{cycle}'; exit 0\n"
    )
    .into_bytes()
}

#[cfg(windows)]
fn finish_command(cycle: usize) -> Vec<u8> {
    let payload = "x".repeat(2048);
    format!(
        "echo {payload}\r\nset \"W=WINDS_T063_DONE_\"\r\necho %W%{cycle}\r\nexit\r\n"
    )
    .into_bytes()
}

fn marker(prefix: &str, cycle: usize) -> Vec<u8> {
    format!("{prefix}{cycle}").into_bytes()
}

fn seed_verification_sentinel(state_root: &Path) {
    let connection = Connection::open(state_root.join("winds.db")).unwrap();
    connection
        .execute_batch(
            "INSERT INTO candidate_runs (
                 run_id, repo_path, base_oid, candidate_ref, candidate_oid,
                 candidate_tree, worktree_path, check_command, timeout_secs, state,
                 created_unix_ms
             ) VALUES (
                 't063-sentinel-run', '/sentinel/repo', 'base-oid', 'candidate-ref',
                 'candidate-oid', 'candidate-tree', '/sentinel/worktree', 'true', 1,
                 'ELIGIBLE', 11
             );
             INSERT INTO events (
                 event_id, run_id, kind, authority, payload_json, created_unix_ms
             ) VALUES (
                 7001, 't063-sentinel-run', 'SentinelEvent', 'WINDS_OBSERVED',
                 '{\"sentinel\":true}', 12
             );
             INSERT INTO evidence_reports (
                 run_id, eligibility, report_json, created_unix_ms
             ) VALUES (
                 't063-sentinel-run', 'ELIGIBLE', '{\"sentinel\":true}', 13
             );
             INSERT INTO promotions (
                 run_id, branch, commit_oid, created_unix_ms
             ) VALUES (
                 't063-sentinel-run', 'sentinel-branch', 'sentinel-commit', 14
             );",
        )
        .unwrap();
}

fn verification_snapshot(state_root: &Path) -> Vec<String> {
    let connection = Connection::open(state_root.join("winds.db")).unwrap();
    [
        "SELECT COALESCE(group_concat(serialized, '\n'), '') FROM (
             SELECT quote(run_id) || '|' || quote(repo_path) || '|' || quote(base_oid) || '|' ||
                    quote(candidate_ref) || '|' || quote(candidate_oid) || '|' || quote(candidate_tree) || '|' ||
                    quote(worktree_path) || '|' || quote(check_command) || '|' || timeout_secs || '|' ||
                    quote(state) || '|' || created_unix_ms AS serialized
             FROM candidate_runs ORDER BY run_id
         )",
        "SELECT COALESCE(group_concat(serialized, '\n'), '') FROM (
             SELECT event_id || '|' || quote(run_id) || '|' || quote(kind) || '|' ||
                    quote(authority) || '|' || quote(payload_json) || '|' || created_unix_ms AS serialized
             FROM events ORDER BY event_id
         )",
        "SELECT COALESCE(group_concat(serialized, '\n'), '') FROM (
             SELECT quote(run_id) || '|' || quote(eligibility) || '|' || quote(report_json) || '|' ||
                    created_unix_ms AS serialized
             FROM evidence_reports ORDER BY run_id
         )",
        "SELECT COALESCE(group_concat(serialized, '\n'), '') FROM (
             SELECT quote(run_id) || '|' || quote(branch) || '|' || quote(commit_oid) || '|' ||
                    created_unix_ms AS serialized
             FROM promotions ORDER BY run_id
         )",
    ]
    .into_iter()
    .map(|query| connection.query_row(query, [], |row| row.get::<_, String>(0)).unwrap())
    .collect()
}

fn active_terminal_count(state_root: &Path) -> i64 {
    Connection::open(state_root.join("winds.db"))
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM executions
             WHERE kind = 'TERMINAL' AND status IN ('REQUESTED', 'RUNNING')",
            [],
            |row| row.get(0),
        )
        .unwrap()
}

fn table_count(state_root: &Path, table: &str) -> i64 {
    let query = format!("SELECT COUNT(*) FROM {table}");
    Connection::open(state_root.join("winds.db"))
        .unwrap()
        .query_row(&query, [], |row| row.get(0))
        .unwrap()
}

fn history_bytes(path: &Path) -> u64 {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return 0,
        Err(error) => panic!("failed to inspect T063 history directory {}: {error}", path.display()),
    };
    entries
        .map(|entry| entry.unwrap())
        .map(|entry| {
            let file_type = entry.file_type().unwrap();
            if file_type.is_dir() {
                history_bytes(&entry.path())
            } else if file_type.is_file() {
                entry.metadata().unwrap().len()
            } else {
                panic!("unexpected non-file history entry during T063 soak: {}", entry.path().display())
            }
        })
        .sum()
}

#[test]
#[ignore = "T063 100-cycle controlled terminal lifecycle soak; run explicitly in release-candidate"]
fn controlled_terminal_lifecycle_soak_100_cycles() {
    let root = TestRoot::new();
    let mut store = store_with_workspace(&root);
    let state_root = root.state();
    let workspace = root.workspace().canonicalize().unwrap();
    let profile = native_profile();
    let policy = SessionHistoryPolicy::local_bounded(
        false,
        TRANSCRIPT_QUOTA,
        TOTAL_HISTORY_QUOTA,
    )
    .unwrap();

    seed_verification_sentinel(&state_root);
    let verification_before = verification_snapshot(&state_root);

    for cycle in 0..CYCLES {
        let execution_id = format!("t063-soak-{cycle:03}");
        let mut execution = TerminalExecution::start_native_with_local_history(
            &mut store,
            &execution_id,
            "workspace-1",
            &profile,
            &workspace,
            DEFAULT_SIZE,
            LocalTerminalHistory::new(policy, &state_root),
        )
        .unwrap();
        let mut output = OutputPump::start(execution.take_output_reader().unwrap());
        complete_headless_terminal_startup(&mut execution, &mut output, cycle);

        let ready = marker("WINDS_T063_READY_", cycle);
        execution.send_input(&ready_command(cycle)).unwrap();
        output.wait_for(&ready, cycle);

        let resized = TerminalSize {
            rows: 25 + u16::try_from(cycle % 8).unwrap(),
            cols: 81 + u16::try_from(cycle % 16).unwrap(),
        };
        execution.resize(resized).unwrap();
        assert_eq!(execution.current_size().unwrap(), resized);

        let done = marker("WINDS_T063_DONE_", cycle);
        execution.send_input(&finish_command(cycle)).unwrap();
        output.wait_for(&done, cycle);

        let observed_exit = execution.wait().unwrap();
        assert_eq!(
            observed_exit.exit_code, 0,
            "T063 cycle {cycle} shell exited unsuccessfully"
        );
        let closed_exit = execution.close().unwrap();
        assert_eq!(closed_exit, observed_exit);
        assert_eq!(execution.try_wait().unwrap(), Some(observed_exit.clone()));

        output.drain_to_eof(cycle);
        let history = execution
            .persist_history()
            .unwrap()
            .expect("T063 local transcript history must persist");
        assert!(
            history.manifest.transcript_observed_bytes
                > u64::try_from(TRANSCRIPT_QUOTA).unwrap(),
            "T063 cycle {cycle} did not exercise transcript truncation"
        );
        assert!(
            history.manifest.transcript_retained_bytes <= TRANSCRIPT_QUOTA,
            "T063 cycle {cycle} retained output above its per-session bound"
        );
        assert!(history.manifest.transcript_truncated);
        drop(execution);

        assert_eq!(
            reconcile_terminal_executions_after_restart(&mut store).unwrap(),
            0,
            "T063 cycle {cycle} left a non-final terminal row requiring restart reconciliation"
        );
        assert_eq!(
            active_terminal_count(&state_root),
            0,
            "T063 cycle {cycle} left a falsely-live terminal execution"
        );
        assert!(
            history_bytes(&state_root.join("history")) <= TOTAL_HISTORY_QUOTA,
            "T063 cycle {cycle} exceeded the total local history bound"
        );

        let record = store.load_execution(&execution_id).unwrap();
        assert_eq!(record.status, ExecutionStatus::Exited);
        assert_eq!(record.status_source, FactSource::WindsObserved);
        assert!(record.started_unix_ms.is_some());
        assert!(record.ended_unix_ms.is_some());
        assert!(record.duration_ms.is_some());
        assert_eq!(
            store
                .load_terminal_session(&execution_id)
                .unwrap()
                .close_reason,
            Some(TerminalCloseReason::ProcessExited)
        );
    }

    assert_eq!(active_terminal_count(&state_root), 0);
    assert_eq!(table_count(&state_root, "terminal_sessions"), CYCLES as i64);
    assert_eq!(table_count(&state_root, "executions"), CYCLES as i64);
    assert_eq!(verification_snapshot(&state_root), verification_before);
    assert!(history_bytes(&state_root.join("history")) <= TOTAL_HISTORY_QUOTA);
}
