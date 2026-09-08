use super::*;
use crate::domain::WorkspaceRecord;
use crossterm::event::{KeyEvent, KeyEventState};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })
}

fn workspace(id: &str, root: &str) -> WorkspaceRecord {
    WorkspaceRecord {
        workspace_id: id.to_owned(),
        canonical_worktree_root: root.to_owned(),
        git_common_dir: format!("{root}/.git"),
        created_unix_ms: 1,
        last_opened_unix_ms: 1,
    }
}

#[test]
fn t098_keyboard_paths_cover_core_actions_search_and_verification_inspection_without_pointer() {
    let mut state = WorkbenchState::new();
    state.create_pane(
        "primary",
        Some("workspace-1".into()),
        Some("session-1".into()),
        PaneSize::new(80, 24),
    );
    let workspaces = vec![workspace("workspace-1", "/repos/winds")];
    let mut terminals = terminal::WorkbenchTerminals::new();
    let mut editor = terminal::input::WorkbenchShellEditor::new();
    let mut navigation = ui::WorkbenchNavigation::new();
    let mut accessibility = WorkbenchAccessibilityState::default();

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('f'), KeyModifiers::CONTROL),
        )
        .unwrap();
    for character in "workspace-1".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &workspaces,
                &[],
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    let search = navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Enter, KeyModifiers::NONE),
        )
        .unwrap();
    assert!(matches!(
        search,
        ui::NavigationEffect::Find(ui::FindResolution::Unique(ui::FindMatch {
            target: ui::NavigationTarget::Workspace { ref workspace_id },
            ..
        })) if workspace_id == "workspace-1"
    ));

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('n'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 2);

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('h'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 3);

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('v'), KeyModifiers::ALT),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 4);

    let before_focus = state.selected_pane();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Left, KeyModifiers::ALT),
        )
        .unwrap();
    assert_ne!(state.selected_pane(), before_focus);

    let selected = state.selected_pane().unwrap();
    let before_size = state.pane(selected).unwrap().size;
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(
                KeyCode::Right,
                KeyModifiers::ALT | KeyModifiers::SHIFT,
            ),
        )
        .unwrap();
    assert_eq!(
        state.pane(selected).unwrap().size.columns,
        before_size.columns.saturating_add(1)
    );

    let pane_count = state.panes().len();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &workspaces,
            &[],
            key(KeyCode::Char('w'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), pane_count - 1);

    assert!(accessibility.handle_event(
        &key(KeyCode::Char('e'), KeyModifiers::CONTROL),
        false
    ));
    assert!(accessibility.verification_inspection_open());

    assert!(accessibility.handle_event(&key(KeyCode::Enter, KeyModifiers::NONE), false));
    assert!(editor.lines().iter().all(String::is_empty));
    assert!(accessibility.verification_inspection_open());

    assert!(accessibility.handle_event(&key(KeyCode::Esc, KeyModifiers::NONE), false));
    assert!(!accessibility.verification_inspection_open());
}

#[test]
fn t098_selected_and_lifecycle_states_are_explicit_text_not_color_only() {
    let mut state = WorkbenchState::new();
    let live = state.create_pane("live", None, None, PaneSize::new(80, 24));
    let exited = state.create_pane("exited", None, None, PaneSize::new(80, 24));
    let ownership_lost =
        state.create_pane("ownership", None, None, PaneSize::new(80, 24));
    let error = state.create_pane("error", None, None, PaneSize::new(80, 24));
    state.set_pane_lifecycle(live, PaneLifecycleView::Live);
    state.set_pane_lifecycle(exited, PaneLifecycleView::Exited);
    state.set_pane_lifecycle(ownership_lost, PaneLifecycleView::OwnershipLost);
    state.set_pane_lifecycle(error, PaneLifecycleView::Error);
    state.focus_pane(ownership_lost);

    let text = pane_accessibility_text(&state, true);
    assert!(text.contains("selection=SELECTED lifecycle=OWNERSHIP_LOST"));
    assert!(text.contains("selection=NOT_SELECTED lifecycle=LIVE"));
    assert!(text.contains("lifecycle=EXITED"));
    assert!(text.contains("lifecycle=ERROR"));
}

#[test]
fn t098_verification_projection_wording_keeps_done_verified_and_accepted_distinct() {
    assert_eq!(
        agent_progress_accessibility_label(context::AgentProgressProjection::AgentReportedDone),
        "AGENT_REPORTED_DONE"
    );
    assert_eq!(
        verification_accessibility_label(context::VerificationProjectionState::NotRun),
        "VERIFICATION_NOT_RUN"
    );
    assert_eq!(
        verification_accessibility_label(
            context::VerificationProjectionState::VerifiedForExactCandidate
        ),
        "VERIFIED_FOR_EXACT_CANDIDATE"
    );
    assert_eq!(
        verification_accessibility_label(context::VerificationProjectionState::Stale),
        "VERIFICATION_STALE_NOT_APPLICABLE"
    );
    assert_eq!(
        acceptance_accessibility_label(
            context::HumanAcceptanceProjectionState::AcceptedForExactCandidate
        ),
        "HUMAN_ACCEPTED_FOR_EXACT_CANDIDATE"
    );
    assert_ne!(
        agent_progress_accessibility_label(context::AgentProgressProjection::AgentReportedDone),
        verification_accessibility_label(
            context::VerificationProjectionState::VerifiedForExactCandidate
        )
    );
    assert_ne!(
        verification_accessibility_label(
            context::VerificationProjectionState::VerifiedForExactCandidate
        ),
        acceptance_accessibility_label(
            context::HumanAcceptanceProjectionState::AcceptedForExactCandidate
        )
    );

    let unloaded = verification_inspection_text("candidate-oid", "candidate-tree", None);
    assert!(unloaded.contains("verification_state=CANONICAL_EVIDENCE_NOT_LOADED"));
    assert!(unloaded.contains("INVARIANT=AGENT_REPORTED_DONE != VERIFIED != ACCEPTED"));
    assert!(unloaded.contains("MODE=READ_ONLY"));
}

#[test]
fn t098_small_terminal_fallback_is_deterministic_and_uses_canonical_identity_not_title() {
    assert!(use_compact_workbench_layout(59, 10));
    assert!(use_compact_workbench_layout(60, 9));
    assert!(!use_compact_workbench_layout(60, 10));

    let mut state = WorkbenchState::new();
    state.create_pane(
        "transient 界e\u{301}",
        Some("workspace-canonical".into()),
        Some("session-canonical".into()),
        PaneSize::new(40, 8),
    );
    let output = output::WorkbenchOutput::new();
    let accessibility = WorkbenchAccessibilityState::default();

    let first = compact_workbench_text(
        &state,
        &output,
        accessibility,
        "0123456789abcdef",
        "fedcba9876543210",
    );
    let second = compact_workbench_text(
        &state,
        &output,
        accessibility,
        "0123456789abcdef",
        "fedcba9876543210",
    );
    assert_eq!(first, second);
    assert!(first.contains("workspace=workspace-canonical"));
    assert!(first.contains("session=session-canonical"));
    assert!(first.contains("candidate_oid=0123456789abcdef"));
    assert!(first.contains("candidate_tree=fedcba9876543210"));
    assert!(first.contains("selection=SELECTED"));
    assert!(!first.contains("transient 界e\u{301}"));
}

#[test]
fn t098_unicode_wide_combining_selection_copy_fixture_preserves_identity_and_exact_text() {
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "editor",
        Some("workspace-u".into()),
        Some("session-u".into()),
        PaneSize::new(80, 24),
    );
    let identity_before = (
        state.pane(pane).unwrap().canonical_workspace_id.clone(),
        state.pane(pane).unwrap().canonical_winds_session_id.clone(),
        state.selected_pane(),
    );

    let mut editor = terminal::input::WorkbenchShellEditor::new();
    let fixture = "wide=界 combining=e\u{301} emoji=🙂";
    editor.insert_text(fixture).unwrap();
    editor.select_all();
    assert!(editor.is_selecting());

    let copied_fixture = editor.lines().join("\n");
    assert_eq!(copied_fixture.as_bytes(), fixture.as_bytes());
    editor.cancel_selection();

    let identity_after = (
        state.pane(pane).unwrap().canonical_workspace_id.clone(),
        state.pane(pane).unwrap().canonical_winds_session_id.clone(),
        state.selected_pane(),
    );
    assert_eq!(identity_after, identity_before);
}
