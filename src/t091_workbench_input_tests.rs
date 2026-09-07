use super::super::super::{PaneLifecycleView, PaneSize, WorkbenchState};
use super::super::WorkbenchTerminals;
use super::*;

#[cfg(unix)]
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
#[cfg(unix)]
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
#[cfg(unix)]
use std::fs;
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

fn default_size() -> PaneSize {
    PaneSize::new(80, 24)
}

#[test]
fn t091_dependency_is_exact_and_optional_search_backends_remain_disabled() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains(
        "ratatui-textarea = { version = \"=0.9.2\", default-features = false, features = [\"crossterm\"] }"
    ));
    assert!(!manifest.contains("features = [\"crossterm\", \"search\"]"));
    assert!(!manifest.contains("features = [\"search\"]"));

    let lock = include_str!("../Cargo.lock").replace("\r\n", "\n");
    let package = "[[package]]\nname = \"ratatui-textarea\"\nversion = \"0.9.2\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"3c78d5ba0f26f97baed69a4c479f268a31c7b5b89d68ab939842152e383d6e73\"";
    assert!(lock.contains(package));
}

#[test]
fn t091_shell_editor_preserves_unicode_metacharacters_and_explicit_terminators() {
    let mut editor = WorkbenchShellEditor::new();
    let text = "printf 'مرحبا 🦀' && echo $HOME | sed 's/a/b/' ; * ? [x]";
    assert_eq!(editor.mode(), WorkbenchInputMode::Shell);
    assert!(editor.insert_text(text).unwrap());
    assert_eq!(editor.lines(), &[text.to_owned()]);

    let mut expected_lf = text.as_bytes().to_vec();
    expected_lf.push(b'\n');
    assert_eq!(
        editor
            .submitted_bytes(
                MultilineSubmitPolicy::Reject,
                ShellSubmitTerminator::LineFeed,
            )
            .unwrap(),
        expected_lf
    );

    let mut expected_crlf = text.as_bytes().to_vec();
    expected_crlf.extend_from_slice(b"\r\n");
    assert_eq!(
        editor
            .submitted_bytes(
                MultilineSubmitPolicy::Reject,
                ShellSubmitTerminator::CarriageReturnLineFeed,
            )
            .unwrap(),
        expected_crlf
    );
}

#[test]
fn t091_editor_selection_edit_undo_and_redo_remain_local_editor_state() {
    let mut editor = WorkbenchShellEditor::new();
    assert!(editor.insert_text("alpha βeta").unwrap());
    editor.select_all();
    assert!(editor.is_selecting());
    assert!(editor.backspace());
    assert_eq!(editor.lines(), &[String::new()]);
    assert!(editor.undo());
    assert_eq!(editor.lines(), &["alpha βeta".to_owned()]);
    assert!(editor.redo());
    assert_eq!(editor.lines(), &[String::new()]);

    assert!(editor.undo());
    editor.cancel_selection();
    editor.move_cursor(ShellCursorMove::Head);
    editor.insert_char('>').unwrap();
    assert_eq!(editor.lines(), &[">alpha βeta".to_owned()]);
    assert_eq!(editor.cursor(), (0, 1));
    assert!(editor.delete_next());
    assert_eq!(editor.lines(), &[">lpha βeta".to_owned()]);
}

#[test]
fn t091_long_single_line_submission_is_not_truncated() {
    let mut editor = WorkbenchShellEditor::new();
    let text = format!("printf '{}'; # end", "x".repeat(128 * 1024));
    assert!(editor.insert_text(&text).unwrap());
    let submitted = editor
        .submitted_bytes(
            MultilineSubmitPolicy::Reject,
            ShellSubmitTerminator::LineFeed,
        )
        .unwrap();
    assert_eq!(&submitted[..text.len()], text.as_bytes());
    assert_eq!(&submitted[text.len()..], b"\n");
}

#[test]
fn t091_multiline_and_bracketed_paste_require_explicit_literal_policy() {
    let mut editor = WorkbenchShellEditor::new();
    let pasted = "\u{1b}[200~printf 'one'\nprintf 'two'\u{1b}[201~";
    assert!(editor.paste_multiline_literal(pasted).unwrap());

    let error = editor
        .submitted_bytes(
            MultilineSubmitPolicy::Reject,
            ShellSubmitTerminator::LineFeed,
        )
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("explicit literal-multiline policy")
    );

    let mut expected = pasted.as_bytes().to_vec();
    expected.push(b'\n');
    assert_eq!(
        editor
            .submitted_bytes(
                MultilineSubmitPolicy::Literal,
                ShellSubmitTerminator::LineFeed,
            )
            .unwrap(),
        expected
    );
}

#[test]
fn t091_carriage_return_paste_fails_closed_without_normalizing_editor_state() {
    let mut editor = WorkbenchShellEditor::new();
    assert!(editor.insert_text("prefix").unwrap());
    let before = editor.lines().to_vec();
    assert!(editor.paste_single_line("one\r\ntwo").is_err());
    assert!(editor.paste_multiline_literal("one\r\ntwo").is_err());
    assert_eq!(editor.lines(), before.as_slice());
}

#[test]
fn t091_failed_submit_keeps_editor_content_and_missing_ownership_fails_closed() {
    let mut editor = WorkbenchShellEditor::new();
    editor.insert_text("echo retained").unwrap();
    let mut state = WorkbenchState::new();
    let mut terminals = WorkbenchTerminals::new();

    assert!(
        editor
            .submit_selected(
                &mut state,
                &mut terminals,
                MultilineSubmitPolicy::Reject,
                ShellSubmitTerminator::LineFeed,
            )
            .is_err()
    );
    assert_eq!(editor.lines(), &["echo retained".to_owned()]);

    let pane = state.create_pane("lost", None, None, default_size());
    assert!(state.set_pane_lifecycle(pane, PaneLifecycleView::Live));
    assert!(
        editor
            .submit_selected(
                &mut state,
                &mut terminals,
                MultilineSubmitPolicy::Reject,
                ShellSubmitTerminator::LineFeed,
            )
            .is_err()
    );
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::OwnershipLost
    );
    assert_eq!(editor.lines(), &["echo retained".to_owned()]);
}

#[cfg(unix)]
struct TestRoot(PathBuf);

#[cfg(unix)]
impl TestRoot {
    fn new(name: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "winds-t091-{name}-{}-{sequence}",
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
            .is_some_and(|name| name.starts_with("winds-t091-"));
        if owned_name && canonical_root.starts_with(&canonical_temp) {
            let _ = fs::remove_dir_all(canonical_root);
        }
    }
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
        .expect("T091 fixture executable must become a valid native shell profile")
}

#[cfg(unix)]
fn wait_for_path(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "T091 fixture output did not appear before the deadline: {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
#[test]
fn t091_focus_at_submit_dispatches_exactly_once_to_the_selected_owned_pane() {
    let root = TestRoot::new("selected-dispatch");
    let profile = executable_profile(
        &root,
        "capture.sh",
        "#!/bin/sh\nIFS= read -r line\nprintf '%s' \"$line\" > received.txt\nIFS= read -r _\n",
    );
    let pane_a_cwd = root.path().join("pane-a");
    let pane_b_cwd = root.path().join("pane-b");
    fs::create_dir(&pane_a_cwd).unwrap();
    fs::create_dir(&pane_b_cwd).unwrap();

    let mut state = WorkbenchState::new();
    let pane_a = state.create_pane("pane-a", None, None, default_size());
    let pane_b = state.create_pane("pane-b", None, None, default_size());
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane_a, &profile, &pane_a_cwd)
        .unwrap();
    terminals
        .start_native(&mut state, pane_b, &profile, &pane_b_cwd)
        .unwrap();

    assert!(state.focus_pane(pane_a));
    let payload = "printf 'مرحبا 🦀' && echo $HOME ; * ? [x]";
    let mut editor = WorkbenchShellEditor::new();
    editor.insert_text(payload).unwrap();
    assert!(state.focus_pane(pane_b));

    let receipt = editor
        .submit_selected(
            &mut state,
            &mut terminals,
            MultilineSubmitPolicy::Reject,
            ShellSubmitTerminator::LineFeed,
        )
        .unwrap();
    assert_eq!(receipt.pane_id, pane_b);
    let mut expected = payload.as_bytes().to_vec();
    expected.push(b'\n');
    assert_eq!(receipt.submitted_bytes, expected);
    assert_eq!(editor.lines(), &[String::new()]);

    let pane_b_received = pane_b_cwd.join("received.txt");
    wait_for_path(&pane_b_received);
    assert_eq!(fs::read(&pane_b_received).unwrap(), payload.as_bytes());
    assert!(!pane_a_cwd.join("received.txt").exists());

    // T090 owns terminal-close qualification. This T091 fixture ends after
    // exactly-one-pane dispatch proof and delegates bounded cleanup to the
    // already-qualified TerminalSession RAII path.
    drop(terminals);
}

#[cfg(unix)]
#[test]
fn t091_child_exit_before_submit_rejects_dispatch_and_preserves_editor() {
    let root = TestRoot::new("exit-before-submit");
    let profile = executable_profile(&root, "exit.sh", "#!/bin/sh\nexit 0\n");
    let mut state = WorkbenchState::new();
    let pane = state.create_pane("exit", None, None, default_size());
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
            "T091 child did not exit before fixture deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }

    let mut editor = WorkbenchShellEditor::new();
    editor.insert_text("echo should-not-send").unwrap();
    assert!(
        editor
            .submit_selected(
                &mut state,
                &mut terminals,
                MultilineSubmitPolicy::Reject,
                ShellSubmitTerminator::LineFeed,
            )
            .is_err()
    );
    assert_eq!(editor.lines(), &["echo should-not-send".to_owned()]);
    terminals.close(&mut state, pane).unwrap();
}
