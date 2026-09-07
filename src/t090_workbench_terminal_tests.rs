use super::terminal::WorkbenchTerminals;
use super::{PaneLifecycleView, PanePresentationMetadata, PaneSize, WorkbenchState};
#[cfg(unix)]
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
#[cfg(unix)]
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::io::{self, Cursor, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
struct TestRoot(PathBuf);

#[cfg(unix)]
impl TestRoot {
    fn new(name: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t090-{name}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

#[cfg(unix)]
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
            .is_some_and(|name| name.starts_with("winds-t090-"));
        if owned_name && canonical_root.starts_with(&canonical_temp) {
            let _ = fs::remove_dir_all(canonical_root);
        }
    }
}

#[cfg(unix)]
struct FailingReader;

#[cfg(unix)]
impl Read for FailingReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("synthetic T090 reader failure"))
    }
}

fn default_size() -> PaneSize {
    PaneSize::new(80, 24)
}

#[cfg(unix)]
fn new_pane(state: &mut WorkbenchState, title: &str) -> super::PaneId {
    state.create_pane(
        title,
        Some("workspace-t090".to_owned()),
        None,
        default_size(),
    )
}

#[cfg(unix)]
fn executable_profile(root: &TestRoot, name: &str, body: &str) -> ShellProfile {
    let executable = root.path().join(name);
    fs::write(&executable, body).unwrap();
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions).unwrap();
    let executable = executable.to_str().unwrap().to_owned();
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.path().to_str().unwrap().to_owned(),
        git_common_dir: root.path().to_str().unwrap().to_owned(),
        shell_candidates: vec![executable.clone()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| profile.executable == executable)
        .expect("T090 fixture executable must become a valid native shell profile")
}

#[cfg(unix)]
fn lingering_profile(root: &TestRoot) -> ShellProfile {
    executable_profile(
        root,
        "linger.sh",
        "#!/bin/sh\nprintf 'WINDS_T090_READY\\n'\nsleep 30\n",
    )
}

#[cfg(unix)]
#[test]
fn t090_start_and_close_bind_a_pane_to_the_exact_owned_terminal_session() {
    let root = TestRoot::new("start-close");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "owned");
    let mut terminals = WorkbenchTerminals::new();

    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();
    assert!(terminals.has_owned_terminal(pane));
    assert_eq!(state.pane(pane).unwrap().lifecycle, PaneLifecycleView::Live);

    terminals.close(&mut state, pane).unwrap();
    assert!(!terminals.has_owned_terminal(pane));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Exited
    );
}

#[cfg(unix)]
#[test]
fn t090_resize_changes_presentation_size_only_after_the_owned_session_accepts_it() {
    let root = TestRoot::new("resize");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "resize");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();

    let accepted = PaneSize::new(120, 40);
    terminals.resize(&mut state, pane, accepted).unwrap();
    assert_eq!(state.pane(pane).unwrap().size, accepted);

    assert!(
        terminals
            .resize(&mut state, pane, PaneSize::new(0, 40))
            .is_err()
    );
    assert_eq!(state.pane(pane).unwrap().size, accepted);
    terminals.close(&mut state, pane).unwrap();
}

#[cfg(unix)]
#[test]
fn t090_natural_child_exit_updates_lifecycle_without_removing_the_visual_pane() {
    let root = TestRoot::new("natural-exit");
    let profile = executable_profile(&root, "exit.sh", "#!/bin/sh\nexit 7\n");
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "exit");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let exit = loop {
        if let Some(exit) = terminals.poll_exit(&mut state, pane).unwrap() {
            break exit;
        }
        assert!(
            Instant::now() < deadline,
            "T090 child did not exit inside fixture deadline"
        );
        thread::sleep(Duration::from_millis(10));
    };

    assert_eq!(exit.exit_code, 7);
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Exited
    );
    assert!(
        state.pane(pane).is_some(),
        "visual pane must remain after child exit"
    );
    assert_eq!(terminals.close(&mut state, pane).unwrap(), exit);
}

#[cfg(unix)]
#[test]
fn t090_observed_exit_can_drain_buffered_output_before_owned_close() {
    let root = TestRoot::new("exit-drain");
    let profile = executable_profile(
        &root,
        "tail.sh",
        "#!/bin/sh\nprintf 'WINDS_T090_TAIL\\n'\nexit 0\n",
    );
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "tail");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if terminals.poll_exit(&mut state, pane).unwrap().is_some() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "T090 child did not exit inside fixture deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }

    let mut captured = Vec::new();
    let mut buffer = [0_u8; 64];
    loop {
        let count = terminals
            .read_output_once(&mut state, pane, &mut buffer)
            .unwrap();
        if count == 0 {
            break;
        }
        captured.extend_from_slice(&buffer[..count]);
    }
    assert!(String::from_utf8_lossy(&captured).contains("WINDS_T090_TAIL"));
    terminals.close(&mut state, pane).unwrap();
}

#[cfg(unix)]
#[test]
fn t090_output_reader_eof_while_child_is_live_fails_closed_but_retains_owned_cleanup() {
    let root = TestRoot::new("reader-eof");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "reader-eof");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();
    assert!(
        terminals.replace_output_reader_for_test(pane, Box::new(Cursor::new(Vec::<u8>::new())))
    );

    let mut buffer = [0_u8; 64];
    let error = terminals
        .read_output_once(&mut state, pane, &mut buffer)
        .unwrap_err();
    assert!(error.to_string().contains("reader closed"));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Error
    );
    assert!(terminals.has_owned_terminal(pane));

    terminals.close(&mut state, pane).unwrap();
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Exited
    );
}

#[cfg(unix)]
#[test]
fn t090_output_reader_error_while_child_is_live_fails_closed_but_retains_owned_cleanup() {
    let root = TestRoot::new("reader-error");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "reader-error");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();
    assert!(terminals.replace_output_reader_for_test(pane, Box::new(FailingReader)));

    let mut buffer = [0_u8; 64];
    let error = terminals
        .read_output_once(&mut state, pane, &mut buffer)
        .unwrap_err();
    assert!(error.to_string().contains("reader failed"));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Error
    );
    assert!(terminals.has_owned_terminal(pane));

    terminals.close(&mut state, pane).unwrap();
}

#[cfg(unix)]
#[test]
fn t090_terminal_aware_pane_close_resolves_reader_failure_before_visual_removal() {
    let root = TestRoot::new("pane-close-reader-error");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "pane-close");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();
    assert!(terminals.replace_output_reader_for_test(pane, Box::new(FailingReader)));

    let mut buffer = [0_u8; 64];
    assert!(
        terminals
            .read_output_once(&mut state, pane, &mut buffer)
            .is_err()
    );
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Error
    );
    assert!(terminals.has_owned_terminal(pane));

    let exit = terminals.close_pane(&mut state, pane).unwrap();
    assert!(exit.is_some());
    assert!(!terminals.has_owned_terminal(pane));
    assert!(state.pane(pane).is_none());
}

#[cfg(unix)]
#[test]
fn t090_interrupt_and_terminate_reuse_the_accepted_owned_session_lifecycle() {
    let root = TestRoot::new("interrupt-terminate");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "interrupt");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();

    terminals.interrupt(&mut state, pane).unwrap();
    assert_ne!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
    terminals.terminate(&mut state, pane).unwrap();
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Exited
    );
    assert!(!terminals.has_owned_terminal(pane));
}

#[cfg(unix)]
#[test]
fn t090_start_revalidates_profile_and_working_directory_before_marking_live() {
    let root = TestRoot::new("revalidate");
    let profile = lingering_profile(&root);
    let missing_cwd = root.path().join("missing");
    let mut state = WorkbenchState::new();
    let cwd_pane = new_pane(&mut state, "missing-cwd");
    let mut terminals = WorkbenchTerminals::new();

    assert!(
        terminals
            .start_native(&mut state, cwd_pane, &profile, &missing_cwd)
            .is_err()
    );
    assert_eq!(
        state.pane(cwd_pane).unwrap().lifecycle,
        PaneLifecycleView::Error
    );
    assert!(!terminals.has_owned_terminal(cwd_pane));

    let profile_pane = new_pane(&mut state, "invalid-profile");
    let mut invalid_profile = profile;
    invalid_profile.executable = root.path().join("gone.sh").to_str().unwrap().to_owned();
    assert!(
        terminals
            .start_native(&mut state, profile_pane, &invalid_profile, root.path())
            .is_err()
    );
    assert_eq!(
        state.pane(profile_pane).unwrap().lifecycle,
        PaneLifecycleView::Error
    );
    assert!(!terminals.has_owned_terminal(profile_pane));
}

#[test]
fn t090_restart_presentation_never_reattaches_to_persisted_or_native_identifiers() {
    let mut state = WorkbenchState::new();
    let restored = state.restore_presentation(PanePresentationMetadata {
        display_title: "restored".to_owned(),
        canonical_workspace_id: Some("workspace-stable".to_owned()),
        canonical_winds_session_id: Some("winds-session-stable".to_owned()),
        size: default_size(),
    });
    let mut terminals = WorkbenchTerminals::new();

    assert_eq!(
        state.pane(restored).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
    assert!(!terminals.has_owned_terminal(restored));
    assert!(terminals.poll_exit(&mut state, restored).is_err());
    assert_eq!(
        state.pane(restored).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
}

#[test]
fn t090_live_presentation_without_owned_session_refuses_close_and_marks_ownership_lost() {
    let mut state = WorkbenchState::new();
    let pane = state.create_pane("orphan", None, None, default_size());
    assert!(state.set_pane_lifecycle(pane, PaneLifecycleView::Live));
    let mut terminals = WorkbenchTerminals::new();

    let error = terminals.close_pane(&mut state, pane).unwrap_err();
    assert!(error.to_string().contains("refusing visual close"));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
    assert!(state.pane(pane).is_some());
}

#[cfg(unix)]
#[test]
fn t090_duplicate_start_never_replaces_the_existing_owned_terminal() {
    let root = TestRoot::new("duplicate");
    let profile = lingering_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = new_pane(&mut state, "duplicate");
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, root.path())
        .unwrap();

    assert!(
        terminals
            .start_native(&mut state, pane, &profile, root.path())
            .is_err()
    );
    assert!(terminals.has_owned_terminal(pane));
    assert_eq!(state.pane(pane).unwrap().lifecycle, PaneLifecycleView::Live);
    terminals.close(&mut state, pane).unwrap();
}
