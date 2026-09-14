use crate::desktop_terminal::{
    DesktopTerminalInputRequest, DesktopTerminalLifecycle, DesktopTerminalRegistry,
    DesktopTerminalTargetRequest,
};
#[cfg(unix)]
use crate::desktop_terminal::{DesktopTerminalResizeRequest, DesktopTerminalStartRequest};
use crate::git::shell_profiles::discover_native_shell_profiles;
#[cfg(unix)]
use crate::git::workspace::open_existing_workspace;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
#[cfg(unix)]
use crate::store::{NewWindsSession, NewWorkstream, Store};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(windows)]
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
#[cfg(windows)]
use std::thread;
#[cfg(unix)]
use std::time::{Duration, Instant};
#[cfg(windows)]
use std::time::{Duration as WindowsDuration, Instant};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

fn fixture_root(name: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "winds-t135-{name}-{}-{sequence}",
        std::process::id()
    ));
    fs::create_dir(&root).unwrap();
    root
}

#[cfg(unix)]
fn fixture_profile(root: &Path, body: &str) -> crate::git::shell_profiles::ShellProfile {
    use std::os::unix::fs::PermissionsExt;
    let shell = root.join("fixture-shell");
    fs::write(&shell, format!("#!/bin/sh\n{body}\n")).unwrap();
    let mut permissions = fs::metadata(&shell).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&shell, permissions).unwrap();
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.to_str().unwrap().to_owned(),
        git_common_dir: root.to_str().unwrap().to_owned(),
        shell_candidates: vec![shell.to_str().unwrap().to_owned()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .unwrap()
        .remove(0)
}

fn cleanup(root: &Path) {
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
const WINDOWS_OUTPUT_LIMIT: usize = 128 * 1024;
#[cfg(windows)]
const WINDOWS_CURSOR_POSITION_QUERY: &[u8] = b"\x1b[6n";
#[cfg(windows)]
const WINDOWS_CURSOR_POSITION_RESPONSE: &[u8] = b"\x1b[1;1R";

#[cfg(windows)]
enum WindowsOutputEvent {
    Chunk(Vec<u8>),
    Error(String),
    Eof,
}

#[cfg(windows)]
fn start_windows_output_reader(mut reader: Box<dyn Read + Send>) -> Receiver<WindowsOutputEvent> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    let _ = sender.send(WindowsOutputEvent::Eof);
                    return;
                }
                Ok(count) => {
                    if sender
                        .send(WindowsOutputEvent::Chunk(buffer[..count].to_vec()))
                        .is_err()
                    {
                        return;
                    }
                }
                Err(error) => {
                    let _ = sender.send(WindowsOutputEvent::Error(error.to_string()));
                    return;
                }
            }
        }
    });
    receiver
}

#[cfg(windows)]
fn wait_for_windows_marker(
    receiver: &Receiver<WindowsOutputEvent>,
    output: &mut Vec<u8>,
    marker: &[u8],
) {
    let deadline = Instant::now() + WindowsDuration::from_secs(10);
    while Instant::now() < deadline {
        match receiver.recv_timeout(WindowsDuration::from_millis(100)) {
            Ok(WindowsOutputEvent::Chunk(chunk)) => {
                output.extend_from_slice(&chunk);
                assert!(
                    output.len() <= WINDOWS_OUTPUT_LIMIT,
                    "T135 ConPTY output exceeded bound"
                );
                if output.windows(marker.len()).any(|window| window == marker) {
                    return;
                }
            }
            Ok(WindowsOutputEvent::Error(error)) => {
                panic!("T135 ConPTY output reader failed: {error}")
            }
            Ok(WindowsOutputEvent::Eof) => break,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    panic!(
        "timed out waiting for T135 ConPTY marker {:?}; observed {:?}",
        String::from_utf8_lossy(marker),
        String::from_utf8_lossy(output)
    );
}

#[cfg(windows)]
fn wait_for_windows_eof(receiver: &Receiver<WindowsOutputEvent>, output: &mut Vec<u8>) {
    let deadline = Instant::now() + WindowsDuration::from_secs(10);
    while Instant::now() < deadline {
        match receiver.recv_timeout(WindowsDuration::from_millis(100)) {
            Ok(WindowsOutputEvent::Chunk(chunk)) => {
                output.extend_from_slice(&chunk);
                assert!(
                    output.len() <= WINDOWS_OUTPUT_LIMIT,
                    "T135 ConPTY output exceeded bound"
                );
            }
            Ok(WindowsOutputEvent::Error(error)) => {
                panic!("T135 ConPTY output reader failed: {error}")
            }
            Ok(WindowsOutputEvent::Eof) => return,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!("T135 ConPTY output reader disconnected before EOF")
            }
        }
    }
    panic!("timed out waiting for T135 ConPTY output EOF");
}

#[cfg(unix)]
#[test]
fn terminal_registry_preserves_exact_target_and_byte_order() {
    let root = fixture_root("target-order");
    let profile = fixture_profile(&root, "exec /bin/sh");
    let mut registry = DesktopTerminalRegistry::new();
    let mut started = registry
        .start_with_test_profile("session-a", "workspace-a", &profile, &root, 24, 80)
        .unwrap();
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-a".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    let wrong_target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-b".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    assert!(registry.status_for_target(&wrong_target).is_err());
    registry
        .send_input(DesktopTerminalInputRequest {
            canonical_session_id: target.canonical_session_id.clone(),
            terminal_id: target.terminal_id.clone(),
            bytes: b"printf 'alpha\\342\\230\\203beta\\n'; exit 0\n".to_vec(),
        })
        .unwrap();
    let mut output = Vec::new();
    started.output_reader.read_to_end(&mut output).unwrap();
    let status = registry.observe_output_end(target, None).unwrap();
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("alpha"));
    assert!(text.contains("beta"));
    assert!(output.windows(3).any(|bytes| bytes == [0xe2, 0x98, 0x83]));
    assert_eq!(status.lifecycle, DesktopTerminalLifecycle::Exited);
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn terminal_registry_observes_natural_exit_without_waiting_for_output_eof() {
    let root = fixture_root("process-exit-before-eof");
    let profile = fixture_profile(&root, "exec /bin/sh");
    let mut registry = DesktopTerminalRegistry::new();
    let mut started = registry
        .start_with_test_profile("session-exit", "workspace-a", &profile, &root, 24, 80)
        .unwrap();
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-exit".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    let output_reader = std::thread::spawn(move || {
        let mut output = Vec::new();
        started.output_reader.read_to_end(&mut output).unwrap();
        output
    });
    registry
        .send_input(DesktopTerminalInputRequest {
            canonical_session_id: target.canonical_session_id.clone(),
            terminal_id: target.terminal_id.clone(),
            bytes: b"printf 'T135_EXIT_BEFORE_EOF\n'; exit 0\n".to_vec(),
        })
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let final_status = loop {
        if let Some(status) = registry.observe_process_exit(&target).unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "timed out observing T135 natural exit independently from output EOF"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(final_status.lifecycle, DesktopTerminalLifecycle::Exited);

    let output = output_reader.join().unwrap();
    assert!(String::from_utf8_lossy(&output).contains("T135_EXIT_BEFORE_EOF"));
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn terminal_registry_rejects_cross_session_control_and_bounds_input_and_resize() {
    let root = fixture_root("bounds");
    let profile = fixture_profile(&root, "exec /bin/sh");
    let mut registry = DesktopTerminalRegistry::new();
    let started = registry
        .start_with_test_profile("session-a", "workspace-a", &profile, &root, 24, 80)
        .unwrap();
    let terminal_id = started.status.terminal_id.clone();
    drop(started.output_reader);
    assert!(
        registry
            .start_with_test_profile("session-a", "workspace-a", &profile, &root, 24, 80)
            .is_err()
    );
    assert!(
        registry
            .send_input(DesktopTerminalInputRequest {
                canonical_session_id: "session-b".to_owned(),
                terminal_id: terminal_id.clone(),
                bytes: b"echo forged\n".to_vec(),
            })
            .is_err()
    );
    assert!(
        registry
            .send_input(DesktopTerminalInputRequest {
                canonical_session_id: "session-a".to_owned(),
                terminal_id: terminal_id.clone(),
                bytes: vec![b'x'; 16 * 1024 + 1],
            })
            .is_err()
    );
    assert!(
        registry
            .resize(DesktopTerminalResizeRequest {
                canonical_session_id: "session-a".to_owned(),
                terminal_id: terminal_id.clone(),
                rows: 0,
                cols: 80,
            })
            .is_err()
    );
    let resized = registry
        .resize(DesktopTerminalResizeRequest {
            canonical_session_id: "session-a".to_owned(),
            terminal_id: terminal_id.clone(),
            rows: 40,
            cols: 120,
        })
        .unwrap();
    assert_eq!((resized.rows, resized.cols), (40, 120));
    let close_target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-a".to_owned(),
        terminal_id,
    };
    match registry.close(close_target.clone()) {
        Ok(closed) => {
            assert_eq!(closed.lifecycle, DesktopTerminalLifecycle::Interrupted);
            assert_eq!(closed.close_reason.as_deref(), Some("CLOSED_BY_WINDS"));
        }
        Err(_) => {
            let closed = registry.status_for_target(&close_target).unwrap();
            assert_eq!(closed.lifecycle, DesktopTerminalLifecycle::OwnershipLost);
            assert_eq!(
                closed.close_reason.as_deref(),
                Some("OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN")
            );
            let by_session = registry.status_for_session("session-a").unwrap();
            assert_eq!(by_session, closed);
        }
    }
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn terminal_registry_preserves_control_sequence_bytes_without_trusting_them() {
    let root = fixture_root("control-bytes");
    let profile = fixture_profile(
        &root,
        "printf '\\033]52;c;Zm9yZ2Vk\\007\\033[31mVERIFIED\\033[0m\\n'",
    );
    let mut registry = DesktopTerminalRegistry::new();
    let mut started = registry
        .start_with_test_profile("session-a", "workspace-a", &profile, &root, 24, 80)
        .unwrap();
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-a".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    let mut output = Vec::new();
    started.output_reader.read_to_end(&mut output).unwrap();
    let status = registry.observe_output_end(target, None).unwrap();
    assert!(output.windows(5).any(|bytes| bytes == b"]52;c"));
    assert!(String::from_utf8_lossy(&output).contains("VERIFIED"));
    assert_eq!(status.lifecycle, DesktopTerminalLifecycle::Exited);
    std::thread::sleep(Duration::from_millis(10));
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn terminal_start_resolves_canonical_session_workspace_without_renderer_cwd_or_executable() {
    let root = fixture_root("canonical-resolution");
    let repo = root.join("repo");
    fs::create_dir(&repo).unwrap();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "--initial-branch=main"]);
    git(&["config", "user.name", "Winds T135"]);
    git(&["config", "user.email", "winds-t135@example.invalid"]);
    fs::write(repo.join("tracked.txt"), b"tracked\n").unwrap();
    git(&["add", "--", "tracked.txt"]);
    git(&["commit", "--no-gpg-sign", "-m", "fixture"]);

    let home = root.join("winds-home");
    fs::create_dir(&home).unwrap();
    let home = home.canonicalize().unwrap();
    let workspace = open_existing_workspace(&repo, &home, 100).unwrap();
    let store = Store::open(&home).unwrap();
    store
        .create_workstream(
            NewWorkstream {
                workstream_id: "workstream-a",
                workspace_id: &workspace.workspace_id,
                display_name: "Primary",
            },
            101,
        )
        .unwrap();
    store
        .create_winds_session(
            NewWindsSession {
                session_id: "session-a",
                workstream_id: "workstream-a",
                display_name: "Terminal",
            },
            102,
        )
        .unwrap();
    drop(store);

    let mut registry = DesktopTerminalRegistry::new();
    let mut started = registry
        .start(
            &home,
            DesktopTerminalStartRequest {
                canonical_session_id: "session-a".to_owned(),
                rows: 24,
                cols: 80,
            },
        )
        .unwrap();
    assert_eq!(started.status.canonical_session_id, "session-a");
    assert_eq!(
        started.status.canonical_workspace_id,
        workspace.workspace_id
    );
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-a".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    registry
        .send_input(DesktopTerminalInputRequest {
            canonical_session_id: "session-a".to_owned(),
            terminal_id: target.terminal_id.clone(),
            bytes: b"printf 'T135_CANONICAL\\n'; exit 0\n".to_vec(),
        })
        .unwrap();
    let mut output = Vec::new();
    started.output_reader.read_to_end(&mut output).unwrap();
    let final_status = registry.observe_output_end(target, None).unwrap();
    assert!(String::from_utf8_lossy(&output).contains("T135_CANONICAL"));
    assert_eq!(final_status.lifecycle, DesktopTerminalLifecycle::Exited);
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn terminal_output_stress_preserves_large_stream_order_without_second_pty() {
    let root = fixture_root("output-stress");
    let profile = fixture_profile(&root, "exec /bin/sh");
    let mut registry = DesktopTerminalRegistry::new();
    let mut started = registry
        .start_with_test_profile("session-stress", "workspace-a", &profile, &root, 30, 100)
        .unwrap();
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-stress".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    let command = b"printf 'T135_BEGIN\\n'; i=0; while [ $i -lt 8192 ]; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; i=$((i+1)); done; printf '\\nT135_END\\n'; exit 0\n";
    registry
        .send_input(DesktopTerminalInputRequest {
            canonical_session_id: target.canonical_session_id.clone(),
            terminal_id: target.terminal_id.clone(),
            bytes: command.to_vec(),
        })
        .unwrap();
    let mut output = Vec::new();
    started.output_reader.read_to_end(&mut output).unwrap();
    let final_status = registry.observe_output_end(target, None).unwrap();
    let text = String::from_utf8_lossy(&output);
    let begin = text.find("T135_BEGIN").unwrap();
    let end = text.rfind("T135_END").unwrap();
    assert!(begin < end);
    assert!(output.iter().filter(|byte| **byte == b'x').count() >= 256 * 1024);
    assert_eq!(final_status.lifecycle, DesktopTerminalLifecycle::Exited);
    cleanup(&root);
}

#[cfg(windows)]
#[test]
fn terminal_registry_directly_qualifies_native_windows_conpty_path() {
    let root = fixture_root("windows-conpty");
    let comspec = std::env::var("COMSPEC").expect("windows-2025 must provide COMSPEC");
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.to_str().unwrap().to_owned(),
        git_common_dir: root.to_str().unwrap().to_owned(),
        shell_candidates: vec![comspec.clone()],
        detected_manifests: Vec::new(),
    };
    let profile = discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| profile.executable.eq_ignore_ascii_case(&comspec))
        .expect("COMSPEC must resolve to a qualified native shell profile");
    let mut registry = DesktopTerminalRegistry::new();
    let started = registry
        .start_with_test_profile(
            "session-windows",
            "workspace-windows",
            &profile,
            &root,
            24,
            80,
        )
        .unwrap();
    let target = DesktopTerminalTargetRequest {
        canonical_session_id: "session-windows".to_owned(),
        terminal_id: started.status.terminal_id.clone(),
    };
    let output_events = start_windows_output_reader(started.output_reader);
    let mut output = Vec::new();
    wait_for_windows_marker(&output_events, &mut output, WINDOWS_CURSOR_POSITION_QUERY);
    registry
        .send_input(DesktopTerminalInputRequest {
            canonical_session_id: target.canonical_session_id.clone(),
            terminal_id: target.terminal_id.clone(),
            bytes: WINDOWS_CURSOR_POSITION_RESPONSE.to_vec(),
        })
        .unwrap();
    registry
        .send_input(DesktopTerminalInputRequest {
            canonical_session_id: target.canonical_session_id.clone(),
            terminal_id: target.terminal_id.clone(),
            bytes: b"echo T135_WINDOWS\r\nexit\r\n".to_vec(),
        })
        .unwrap();
    wait_for_windows_marker(&output_events, &mut output, b"T135_WINDOWS");
    let deadline = Instant::now() + WindowsDuration::from_secs(10);
    let final_status = loop {
        if let Some(status) = registry.observe_process_exit(&target).unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "timed out observing T135 ConPTY child exit independently from output EOF"
        );
        std::thread::sleep(WindowsDuration::from_millis(25));
    };
    assert_eq!(final_status.lifecycle, DesktopTerminalLifecycle::Exited);
    wait_for_windows_eof(&output_events, &mut output);
    assert!(String::from_utf8_lossy(&output).contains("T135_WINDOWS"));
    cleanup(&root);
}
