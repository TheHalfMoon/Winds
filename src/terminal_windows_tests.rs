use super::{TerminalSession, TerminalSize};
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::git::wsl_launch::{WslCwdResolution, launch_wsl_terminal, prepare_wsl_terminal_launch};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
const OUTPUT_LIMIT: usize = 128 * 1024;
const CURSOR_POSITION_QUERY: &[u8] = b"\x1b[6n";
const TEST_CURSOR_POSITION_RESPONSE: &[u8] = b"\x1b[1;1R";

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(name: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t051-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let canonical_temp = match std::env::temp_dir().canonicalize() {
            Ok(value) => value,
            Err(_) => return,
        };
        let canonical_root = match self.0.canonicalize() {
            Ok(value) => value,
            Err(_) => return,
        };
        let owned_name = canonical_root
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.starts_with("winds-t051-"));
        if owned_name && canonical_root.starts_with(&canonical_temp) {
            let _ = fs::remove_dir_all(canonical_root);
        }
    }
}

enum OutputEvent {
    Chunk(Vec<u8>),
    Error(String),
    Eof,
}

fn native_cmd_profile() -> ShellProfile {
    let comspec = std::env::var("COMSPEC").expect("windows-latest must provide COMSPEC");
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: r"C:\unused\worktree".to_owned(),
        git_common_dir: r"C:\unused\git-common".to_owned(),
        shell_candidates: vec![comspec.clone()],
        detected_manifests: Vec::new(),
    };

    discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| profile.executable.eq_ignore_ascii_case(&comspec))
        .expect("COMSPEC must be a concrete T048 shell profile on Windows")
}

fn start_output_reader(mut reader: Box<dyn Read + Send>) -> Receiver<OutputEvent> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
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
    receiver
}

fn wait_for_output(receiver: &Receiver<OutputEvent>, needle: &[u8]) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = Vec::new();
    while Instant::now() < deadline {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(OutputEvent::Chunk(chunk)) => {
                output.extend_from_slice(&chunk);
                assert!(
                    output.len() <= OUTPUT_LIMIT,
                    "ConPTY test output exceeded bound"
                );
                if output.windows(needle.len()).any(|window| window == needle) {
                    return output;
                }
            }
            Ok(OutputEvent::Error(error)) => panic!("ConPTY output reader failed: {error}"),
            Ok(OutputEvent::Eof) => break,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    panic!(
        "timed out waiting for ConPTY output marker {:?}; observed {:?}",
        String::from_utf8_lossy(needle),
        String::from_utf8_lossy(&output)
    );
}

fn complete_headless_terminal_startup(
    session: &mut TerminalSession,
    output: &Receiver<OutputEvent>,
) {
    // ConPTY exposes terminal query traffic to its host. A real terminal frontend
    // answers DECXCPR; this focused headless fixture supplies the minimal valid
    // response so command execution is tested rather than terminal emulation.
    wait_for_output(output, CURSOR_POSITION_QUERY);
    session
        .send_input(TEST_CURSOR_POSITION_RESPONSE)
        .expect("headless ConPTY fixture must answer cursor-position query");
}

fn output_contains_exact_marker(output: &[u8], marker: &str) -> bool {
    let marker = marker.as_bytes();
    output
        .windows(marker.len())
        .any(|window| window == marker)
}

fn default_size() -> TerminalSize {
    TerminalSize { rows: 24, cols: 80 }
}

#[test]
fn conpty_streams_input_output_from_exact_start_cwd_and_observes_exit() {
    let root = TestRoot::new("stream");
    let canonical_root = root.path().canonicalize().unwrap();
    let profile = native_cmd_profile();
    let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
    let session_id = session.session_id();
    assert_eq!(session.profile_id(), profile.profile_id);
    assert_eq!(session.start_cwd(), canonical_root);

    let output = start_output_reader(session.take_output_reader().unwrap());
    assert!(session.take_output_reader().is_err());
    complete_headless_terminal_startup(&mut session, &output);
    session
        .send_input(b"cd\r\necho WINDS_READY\r\nexit\r\n")
        .unwrap();
    let observed = wait_for_output(&output, b"WINDS_READY");
    let cwd = canonical_root.to_string_lossy();
    assert!(
        observed
            .windows(cwd.len())
            .any(|window| window.eq_ignore_ascii_case(cwd.as_bytes()))
    );

    let exit = session.wait().unwrap();
    assert_eq!(exit.exit_code, 0);
    assert_eq!(session.session_id(), session_id);
    assert_eq!(session.try_wait().unwrap(), Some(exit));
}

#[test]
fn conpty_resize_updates_owned_pseudoconsole_dimensions() {
    let root = TestRoot::new("resize");
    let profile = native_cmd_profile();
    let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();

    let resized = TerminalSize {
        rows: 40,
        cols: 120,
    };
    session.resize(resized).unwrap();
    assert_eq!(session.current_size().unwrap(), resized);
    session.close().unwrap();
}

#[test]
fn conpty_interrupt_fails_closed_without_corrupting_the_session() {
    let root = TestRoot::new("interrupt");
    let profile = native_cmd_profile();
    let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
    let output = start_output_reader(session.take_output_reader().unwrap());

    complete_headless_terminal_startup(&mut session, &output);
    session.send_input(b"echo WINDS_READY\r\n").unwrap();
    wait_for_output(&output, b"WINDS_READY");

    let error = session.interrupt().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("interrupt is unsupported on native Windows")
    );

    session.send_input(b"echo WINDS_AFTER\r\nexit\r\n").unwrap();
    wait_for_output(&output, b"WINDS_AFTER");
    let exit = session.wait().unwrap();
    assert_eq!(exit.exit_code, 0);
}

#[test]
fn conpty_terminate_reaps_the_exact_owned_child() {
    let root = TestRoot::new("terminate");
    let profile = native_cmd_profile();
    let mut session = TerminalSession::start(&profile, root.path(), default_size()).unwrap();
    let output = start_output_reader(session.take_output_reader().unwrap());

    complete_headless_terminal_startup(&mut session, &output);
    session
        .send_input(b"echo WINDS_READY\r\nset /p WINDS_BLOCK=\r\n")
        .unwrap();
    wait_for_output(&output, b"WINDS_READY");
    let exit = session.terminate().unwrap();
    assert_ne!(exit.exit_code, 0);
    assert_eq!(session.try_wait().unwrap(), Some(exit.clone()));
    assert_eq!(session.close().unwrap(), exit);
}

#[test]
fn t062_real_wsl_backend_launch_is_opt_in_and_uses_production_path() {
    let distro = match std::env::var("WINDS_T062_WSL_DISTRO") {
        Ok(value) if !value.is_empty() => value,
        _ => return,
    };
    let expected = std::env::var("WINDS_T062_EXPECT_CWD")
        .expect("WINDS_T062_EXPECT_CWD must be set when the real T062 backend proof is enabled");
    let repo = std::env::current_dir()
        .expect("T062 backend proof must have a current directory")
        .canonicalize()
        .expect("T062 backend proof repository must canonicalize");

    let plan = prepare_wsl_terminal_launch(&repo, &distro)
        .expect("production WSL launch preparation must succeed on the provisioned distribution");
    let (expected_linux_cwd, expected_git_head) = match (expected.as_str(), &plan.cwd_resolution) {
        (
            "MAPPED",
            WslCwdResolution::MappedWorkspace {
                windows_workspace_root,
                linux_workspace_root,
                git_head_oid,
                ..
            },
        ) => {
            assert_eq!(
                Path::new(windows_workspace_root)
                    .canonicalize()
                    .expect("prepared Windows workspace must canonicalize"),
                repo
            );
            assert!(linux_workspace_root.starts_with('/'));
            (linux_workspace_root.clone(), Some(git_head_oid.clone()))
        }
        ("FALLBACK", WslCwdResolution::FallbackHome { linux_home, .. }) => {
            assert!(linux_home.starts_with('/'));
            (linux_home.clone(), None)
        }
        (expected, actual) => panic!(
            "production WSL launch preparation returned unexpected cwd resolution: expected={expected}, actual={actual:?}"
        ),
    };

    let prepared_profile = plan.profile.clone();
    let prepared_cwd = plan.cwd_resolution.clone();
    let mut launched = launch_wsl_terminal(&plan, default_size())
        .expect("production WSL terminal launch must succeed on the real selected distribution");
    assert_eq!(launched.profile, prepared_profile);
    assert_eq!(launched.cwd_resolution, prepared_cwd);
    assert_eq!(launched.session.profile_id(), launched.profile.profile_id);
    assert_eq!(launched.session.start_cwd(), repo);
    assert_eq!(launched.session.current_size().unwrap(), default_size());

    let output = start_output_reader(
        launched
            .session
            .take_output_reader()
            .expect("production WSL session must expose its owned output reader"),
    );
    assert!(launched.session.take_output_reader().is_err());
    complete_headless_terminal_startup(&mut launched.session, &output);
    let command = if expected_git_head.is_some() {
        "printf 'WINDS_T062_%s%s\\n' 'CWD=' \"$(pwd -P)\"; printf 'WINDS_T062_%s%s\\n' 'HEAD=' \"$(git rev-parse --verify HEAD^{commit})\"; printf 'WINDS_T062_%s\\n' 'DONE'; exit\r\n"
    } else {
        "printf 'WINDS_T062_%s%s\\n' 'CWD=' \"$(pwd -P)\"; printf 'WINDS_T062_%s\\n' 'DONE'; exit\r\n"
    };
    launched
        .session
        .send_input(command.as_bytes())
        .expect("production WSL shell must accept the real cwd proof command");
    let observed = wait_for_output(&output, b"WINDS_T062_DONE");
    let observed_text = String::from_utf8_lossy(&observed);
    let cwd_marker = format!("WINDS_T062_CWD={expected_linux_cwd}");
    assert!(
        output_contains_exact_marker(&observed, &cwd_marker),
        "production WSL shell did not observe the prepared Linux cwd; observed {observed_text:?}"
    );
    if let Some(expected_head) = expected_git_head {
        let head_marker = format!("WINDS_T062_HEAD={expected_head}");
        assert!(
            output_contains_exact_marker(&observed, &head_marker),
            "production WSL shell did not observe the prepared Git identity; observed {observed_text:?}"
        );
    }
    let exit = launched
        .session
        .wait()
        .expect("production WSL terminal session must observe its exact owned child exit");
    assert_eq!(exit.exit_code, 0);
}
