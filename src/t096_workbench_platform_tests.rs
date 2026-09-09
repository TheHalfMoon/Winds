use super::screen::WorkbenchScreen;
use super::screen::host_safety::{HOST_INTEGRATION_CAPABILITIES, TerminalHostSafetySnapshot};
use super::terminal::WorkbenchTerminals;
use super::terminal::input::{MultilineSubmitPolicy, ShellSubmitTerminator, WorkbenchShellEditor};
use super::ui::WorkbenchNavigation;
use super::{PaneLifecycleView, PaneSize, WorkbenchState, render_inert_workbench};
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use crate::git::wsl_launch::{WslCwdResolution, prepare_wsl_terminal_launch};

const DEFAULT_SIZE: PaneSize = PaneSize::new(80, 24);
const RESIZED_SIZE: PaneSize = PaneSize::new(100, 30);
const NATIVE_MARKER: &str = "WINDS_T096_NATIVE_OK";
#[cfg(windows)]
const WSL_MARKER: &str = "WINDS_T096_WSL_OK";
#[cfg(windows)]
const CURSOR_POSITION_QUERY: &[u8] = b"\x1b[6n";
#[cfg(windows)]
const TEST_CURSOR_POSITION_RESPONSE: &[u8] = b"\x1b[1;1R";

const _: () = {
    assert!(!HOST_INTEGRATION_CAPABILITIES.terminal_clipboard_write);
    assert!(!HOST_INTEGRATION_CAPABILITIES.external_url_open);
    assert!(!HOST_INTEGRATION_CAPABILITIES.external_file_open);
    assert!(!HOST_INTEGRATION_CAPABILITIES.network_or_browser_integration);
};

fn current_checkout() -> PathBuf {
    std::env::current_dir()
        .expect("T096 runner must expose a current checkout")
        .canonicalize()
        .expect("T096 checkout must have a canonical path")
}

fn inventory(root: &Path, candidates: Vec<String>) -> WorkspaceEnvironmentInventory {
    WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.to_string_lossy().into_owned(),
        git_common_dir: root.join(".git").to_string_lossy().into_owned(),
        shell_candidates: candidates,
        detected_manifests: Vec::new(),
    }
}

#[cfg(unix)]
fn native_profile(root: &Path) -> ShellProfile {
    discover_native_shell_profiles(&inventory(root, vec!["/bin/sh".to_owned()]))
        .expect("T096 native Unix shell discovery must succeed")
        .into_iter()
        .find(|profile| profile.executable == "/bin/sh")
        .expect("T096 Unix qualification requires /bin/sh")
}

#[cfg(windows)]
fn native_profile(root: &Path) -> ShellProfile {
    let profiles = discover_native_shell_profiles(&inventory(root, Vec::new()))
        .expect("T096 native Windows shell discovery must succeed");
    profiles
        .iter()
        .find(|profile| profile.display_name.eq_ignore_ascii_case("cmd.exe"))
        .cloned()
        .or_else(|| profiles.into_iter().next())
        .expect("T096 native Windows qualification requires a discovered shell")
}

fn host_submit_terminator() -> ShellSubmitTerminator {
    #[cfg(windows)]
    {
        ShellSubmitTerminator::CarriageReturnLineFeed
    }
    #[cfg(not(windows))]
    {
        ShellSubmitTerminator::LineFeed
    }
}

fn key(character: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(character),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn exercise_host_input_and_dispatch(
    state: &mut WorkbenchState,
    terminals: &mut WorkbenchTerminals,
    marker: &str,
) {
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();

    navigation
        .handle_event(
            state,
            terminals,
            &mut editor,
            &[],
            &[],
            Event::Paste("echo ".to_owned()),
        )
        .expect("T096 paste event must remain a bounded editor operation");
    for character in marker.chars() {
        navigation
            .handle_event(state, terminals, &mut editor, &[], &[], key(character))
            .expect("T096 keyboard input must remain a bounded editor operation");
    }

    let receipt = editor
        .submit_selected(
            state,
            terminals,
            MultilineSubmitPolicy::Reject,
            host_submit_terminator(),
        )
        .expect("T096 selected-pane dispatch must reach the owned terminal");
    assert!(
        receipt
            .submitted_bytes
            .windows(marker.len())
            .any(|window| window == marker.as_bytes())
    );
}

fn exercise_unicode_parser_and_fail_closed_osc52(screen: &mut WorkbenchScreen) {
    let unicode = "λ中e\u{301}";
    screen.process_observed_bytes(unicode.as_bytes());
    assert!(
        screen
            .transcript_snapshot()
            .lines
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>()
            .windows(unicode.len())
            .any(|window| window == unicode.as_bytes()),
        "T096 Unicode bytes must remain present in bounded transcript truth"
    );

    screen.process_observed_bytes(b"\x1b]52;c;U0VDUkVU\x07");
    let callbacks = screen.callback_summary();
    assert_eq!(callbacks.clipboard_copy_requests, 1);
    let safety = TerminalHostSafetySnapshot::from_callbacks(callbacks);
    assert_eq!(safety.host_actions_performed, 0);
    assert_eq!(safety.trusted_ui_state_transitions, 0);
}

#[cfg(windows)]
fn complete_windows_headless_terminal_startup(
    state: &mut WorkbenchState,
    terminals: &mut WorkbenchTerminals,
    screen: &mut WorkbenchScreen,
) {
    let pane = state
        .selected_pane()
        .expect("T096 Windows startup requires one selected pane");
    let mut observed = Vec::new();
    for _ in 0..64 {
        let mut buffer = [0_u8; 4096];
        let count = terminals
            .read_output_once(state, pane, &mut buffer)
            .expect("T096 Windows startup output must remain readable");
        if count == 0 {
            break;
        }
        screen.process_observed_bytes(&buffer[..count]);
        observed.extend_from_slice(&buffer[..count]);
        if observed
            .windows(CURSOR_POSITION_QUERY.len())
            .any(|window| window == CURSOR_POSITION_QUERY)
        {
            let dispatched = terminals
                .dispatch_selected_input(state, TEST_CURSOR_POSITION_RESPONSE)
                .expect("T096 headless ConPTY fixture must answer cursor-position query");
            assert_eq!(dispatched, pane);
            return;
        }
    }
    panic!(
        "T096 timed out before observing the ConPTY cursor-position query; bytes={:?}",
        String::from_utf8_lossy(&observed)
    );
}

fn read_until_marker(
    state: &mut WorkbenchState,
    terminals: &mut WorkbenchTerminals,
    screen: &mut WorkbenchScreen,
    marker: &str,
) {
    let mut observed = Vec::new();
    for _ in 0..64 {
        let mut buffer = [0_u8; 4096];
        let count = terminals
            .read_output_once(state, state.selected_pane().unwrap(), &mut buffer)
            .expect("T096 owned terminal output must remain readable");
        if count == 0 {
            break;
        }
        screen.process_observed_bytes(&buffer[..count]);
        observed.extend_from_slice(&buffer[..count]);
        if observed
            .windows(marker.len())
            .any(|window| window == marker.as_bytes())
        {
            return;
        }
    }
    panic!(
        "T096 marker was not observed through the owned terminal path; bytes={:?}",
        String::from_utf8_lossy(&observed)
    );
}

fn render_host_tui() {
    let backend = TestBackend::new(DEFAULT_SIZE.columns, DEFAULT_SIZE.rows);
    let mut terminal = Terminal::new(backend).expect("T096 test backend must initialize");
    terminal
        .draw(render_inert_workbench)
        .expect("T096 host TUI render path must draw successfully");
}

#[test]
fn t096_native_workbench_path_directly_qualifies_current_host_domain() {
    assert!(
        matches!(std::env::consts::OS, "windows" | "linux" | "macos"),
        "unsupported host domain must remain NOT_CLAIMED"
    );

    render_host_tui();
    let root = current_checkout();
    let profile = native_profile(&root);
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "t096-native",
        Some("workspace-t096-native".to_owned()),
        None,
        DEFAULT_SIZE,
    );
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane, &profile, &root)
        .expect("T096 native workbench terminal must start on the current host");
    assert!(terminals.has_owned_terminal(pane));
    assert_eq!(state.pane(pane).unwrap().lifecycle, PaneLifecycleView::Live);

    let mut screen = WorkbenchScreen::new(DEFAULT_SIZE).unwrap();
    #[cfg(windows)]
    complete_windows_headless_terminal_startup(&mut state, &mut terminals, &mut screen);

    let mut navigation = WorkbenchNavigation::new();
    let mut resize_editor = WorkbenchShellEditor::new();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut resize_editor,
            &[],
            &[],
            Event::Resize(RESIZED_SIZE.columns, RESIZED_SIZE.rows),
        )
        .expect("T096 host resize event must reach the exact owned pane");
    assert_eq!(state.pane(pane).unwrap().size, RESIZED_SIZE);
    screen
        .explicit_resize(RESIZED_SIZE)
        .expect("T096 screen projection must follow the accepted host resize");

    exercise_host_input_and_dispatch(&mut state, &mut terminals, NATIVE_MARKER);
    read_until_marker(&mut state, &mut terminals, &mut screen, NATIVE_MARKER);
    assert!(screen.screen_contents().contains(NATIVE_MARKER));
    exercise_unicode_parser_and_fail_closed_osc52(&mut screen);

    let close_result = terminals.close(&mut state, pane);
    assert!(!terminals.has_owned_terminal(pane));
    match close_result {
        Ok(_) => assert_eq!(
            state.pane(pane).unwrap().lifecycle,
            PaneLifecycleView::Exited
        ),
        Err(error) => {
            assert!(
                error.to_string().contains(
                    "terminal close could not prove owned child exit inside bounded cleanup window"
                ),
                "unexpected T096 native terminal close error: {error}"
            );
            assert_eq!(
                state.pane(pane).unwrap().lifecycle,
                PaneLifecycleView::OwnershipLost
            );
        }
    }
}

#[cfg(windows)]
#[test]
#[ignore = "requires a real provisioned WSL2 distribution in the windows-terminal workflow"]
fn t096_real_wsl2_workbench_path_preserves_host_guest_domain_and_path_truth() {
    let distro = std::env::var("WINDS_T096_WSL_DISTRO")
        .expect("T096 real WSL2 proof requires WINDS_T096_WSL_DISTRO");
    let root = current_checkout();
    let plan = prepare_wsl_terminal_launch(&root, &distro)
        .expect("T096 production WSL launch preparation must succeed");
    assert_eq!(plan.profile.execution_domain.host_os, "windows");
    assert_eq!(plan.profile.execution_domain.distribution, distro);
    assert_eq!(plan.profile.execution_domain.version, 2);
    match &plan.cwd_resolution {
        WslCwdResolution::MappedWorkspace {
            windows_workspace_root,
            linux_workspace_root,
            linux_git_common_dir,
            git_head_oid,
        } => {
            assert!(!windows_workspace_root.is_empty());
            assert!(linux_workspace_root.starts_with('/'));
            assert!(linux_git_common_dir.starts_with('/'));
            assert!(!git_head_oid.is_empty());
        }
        WslCwdResolution::FallbackHome { reason, .. } => {
            panic!("T096 real WSL2 qualification requires a mapped workspace: {reason}");
        }
    }

    render_host_tui();
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "t096-wsl2",
        Some("workspace-t096-wsl2".to_owned()),
        None,
        DEFAULT_SIZE,
    );
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_wsl(&mut state, pane, &plan)
        .expect("T096 real WSL2 workbench terminal must start");
    assert!(terminals.has_owned_terminal(pane));
    assert_eq!(state.pane(pane).unwrap().lifecycle, PaneLifecycleView::Live);

    let mut screen = WorkbenchScreen::new(DEFAULT_SIZE).unwrap();
    complete_windows_headless_terminal_startup(&mut state, &mut terminals, &mut screen);

    let mut navigation = WorkbenchNavigation::new();
    let mut resize_editor = WorkbenchShellEditor::new();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut resize_editor,
            &[],
            &[],
            Event::Resize(RESIZED_SIZE.columns, RESIZED_SIZE.rows),
        )
        .expect("T096 WSL2 resize must pass through the owned Windows/WSL terminal path");
    assert_eq!(state.pane(pane).unwrap().size, RESIZED_SIZE);
    screen
        .explicit_resize(RESIZED_SIZE)
        .expect("T096 WSL2 screen projection must follow the accepted host resize");

    exercise_host_input_and_dispatch(&mut state, &mut terminals, WSL_MARKER);
    read_until_marker(&mut state, &mut terminals, &mut screen, WSL_MARKER);
    assert!(screen.screen_contents().contains(WSL_MARKER));
    exercise_unicode_parser_and_fail_closed_osc52(&mut screen);

    terminals
        .close(&mut state, pane)
        .expect("T096 real WSL2 owned terminal must close with proven cleanup");
    assert!(!terminals.has_owned_terminal(pane));
    assert_eq!(
        state.pane(pane).unwrap().lifecycle,
        PaneLifecycleView::Exited
    );
}
