#![allow(dead_code)]

#[path = "../src/command.rs"]
mod command;
#[path = "../src/domain.rs"]
mod domain;
#[path = "../src/execution.rs"]
mod execution;
#[path = "../src/git.rs"]
mod git;
#[path = "../src/store.rs"]
mod store;

use crate::domain::{ExecutionStatus, FactSource, TerminalCloseReason};
use crate::execution::TerminalExecution;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::store::{NewWorkspace, Store};
use std::fs;
#[cfg(windows)]
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(windows)]
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
#[cfg(windows)]
use std::thread::{self, JoinHandle};
#[cfg(windows)]
use std::time::{Duration, Instant};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
const INITIAL_SIZE: TerminalSize = TerminalSize { rows: 24, cols: 80 };
#[cfg(windows)]
const RESIZED: TerminalSize = TerminalSize { rows: 33, cols: 101 };
#[cfg(windows)]
const CURSOR_POSITION_QUERY: &[u8] = b"\x1b[6n";
#[cfg(windows)]
const CURSOR_POSITION_RESPONSE: &[u8] = b"\x1b[1;1R";

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let temp = std::env::temp_dir()
            .canonicalize()
            .expect("T063 CI must expose a canonical temporary directory");
        let path = temp.join(format!(
            "winds-t063-close-guard-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn state(&self) -> PathBuf {
        self.0.join("state")
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let Ok(temp) = std::env::temp_dir().canonicalize() else {
            return;
        };
        let Ok(root) = self.0.canonicalize() else {
            return;
        };
        let owned = root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("winds-t063-close-guard-"));
        if owned && root.parent() == Some(temp.as_path()) {
            let _ = fs::remove_dir_all(root);
        }
    }
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

fn store_with_workspace(root: &TestRoot) -> Store {
    let git_common_dir = root.path().join(".git");
    fs::create_dir(&git_common_dir).unwrap();
    let store = Store::open(&root.state()).unwrap();
    store
        .create_workspace(
            NewWorkspace {
                workspace_id: "workspace-1",
                canonical_worktree_root: root.path().to_str().unwrap(),
                git_common_dir: git_common_dir.to_str().unwrap(),
            },
            1,
        )
        .unwrap();
    store
}

#[cfg(windows)]
enum OutputEvent {
    Chunk(Vec<u8>),
    Error(String),
    Eof,
}

#[cfg(windows)]
struct OutputPump {
    receiver: Receiver<OutputEvent>,
    handle: Option<JoinHandle<()>>,
    observed: Vec<u8>,
}

#[cfg(windows)]
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

    fn wait_for(&mut self, needle: &[u8]) {
        if contains_bytes(&self.observed, needle) {
            return;
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(OutputEvent::Chunk(chunk)) => {
                    self.observed.extend_from_slice(&chunk);
                    assert!(
                        self.observed.len() <= 64 * 1024,
                        "T063 Windows child-size guard exceeded its output bound"
                    );
                    if contains_bytes(&self.observed, needle) {
                        return;
                    }
                }
                Ok(OutputEvent::Error(error)) => {
                    panic!("T063 Windows output reader failed: {error}")
                }
                Ok(OutputEvent::Eof) => break,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        panic!(
            "T063 Windows guard timed out waiting for {:?}; observed {:?}",
            String::from_utf8_lossy(needle),
            String::from_utf8_lossy(&self.observed)
        );
    }

    fn finish(mut self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if Instant::now() >= deadline {
                panic!("T063 Windows output reader did not reach EOF after active close");
            }
            match self.receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(OutputEvent::Chunk(chunk)) => self.observed.extend_from_slice(&chunk),
                Ok(OutputEvent::Error(error)) => {
                    panic!("T063 Windows output reader failed after close: {error}")
                }
                Ok(OutputEvent::Eof) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => continue,
            }
        }
        if let Some(handle) = self.handle.take() {
            handle.join().expect("T063 Windows output reader panicked");
        }
    }
}

#[cfg(windows)]
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

#[test]
fn active_close_and_windows_child_resize_guard() {
    let root = TestRoot::new();
    let mut store = store_with_workspace(&root);
    let profile = native_profile();
    let mut execution = TerminalExecution::start_native(
        &mut store,
        "t063-active-close-guard",
        "workspace-1",
        &profile,
        root.path(),
        INITIAL_SIZE,
    )
    .unwrap();

    assert_eq!(
        execution.try_wait().unwrap(),
        None,
        "T063 close guard requires a live owned child before close()"
    );

    #[cfg(windows)]
    let mut output = {
        let mut output = OutputPump::start(execution.take_output_reader().unwrap());
        output.wait_for(CURSOR_POSITION_QUERY);
        execution.send_input(CURSOR_POSITION_RESPONSE).unwrap();
        execution.resize(RESIZED).unwrap();
        let command = "powershell -NoProfile -NonInteractive -Command \"$s=$Host.UI.RawUI.WindowSize; Write-Output ('WINDS_T063_CHILD_SIZE_' + $s.Height + ' ' + $s.Width)\"\r\n";
        execution.send_input(command.as_bytes()).unwrap();
        let expected = format!("WINDS_T063_CHILD_SIZE_{} {}", RESIZED.rows, RESIZED.cols);
        output.wait_for(expected.as_bytes());
        assert_eq!(
            execution.try_wait().unwrap(),
            None,
            "T063 Windows child-size proof must leave the owned shell live before close()"
        );
        output
    };

    let closed_exit = execution
        .close()
        .expect("T063 active close must complete bounded owned-child cleanup");
    assert_eq!(execution.try_wait().unwrap(), Some(closed_exit));
    drop(execution);

    #[cfg(windows)]
    output.finish();

    let record = store.load_execution("t063-active-close-guard").unwrap();
    assert_eq!(record.status, ExecutionStatus::Interrupted);
    assert_eq!(record.status_source, FactSource::WindsObserved);
    assert!(record.started_unix_ms.is_some());
    assert!(record.ended_unix_ms.is_some());
    assert!(record.duration_ms.is_some());
    let terminal = store
        .load_terminal_session("t063-active-close-guard")
        .unwrap();
    assert_eq!(
        terminal.close_reason,
        Some(TerminalCloseReason::ClosedByWinds)
    );
}
