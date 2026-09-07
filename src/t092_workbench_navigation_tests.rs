use super::*;
use crossterm::event::{KeyEventState, MouseEvent};
use std::collections::VecDeque;

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

fn session(id: &str, workstream_id: &str, display_name: &str) -> WindsSessionRecord {
    WindsSessionRecord {
        session_id: id.to_owned(),
        workstream_id: workstream_id.to_owned(),
        display_name: display_name.to_owned(),
        created_unix_ms: 1,
        updated_unix_ms: 1,
    }
}

#[test]
fn t092_findability_prefers_exact_canonical_ids_and_returns_ambiguity_explicitly() {
    let mut state = WorkbenchState::new();
    state.create_pane(
        "API",
        Some("workspace-alpha".into()),
        Some("session-api-1".into()),
        PaneSize::new(80, 24),
    );
    state.create_pane(
        "api",
        Some("workspace-beta".into()),
        Some("session-api-2".into()),
        PaneSize::new(80, 24),
    );

    let workspaces = vec![
        workspace("workspace-beta", "/repos/alpha-archive"),
        workspace("workspace-alpha", "/repos/Alpha"),
    ];
    let sessions = vec![
        session("session-build-2", "workstream-2", "Build"),
        session("session-build-1", "workstream-1", "build"),
        session("session-unicode", "workstream-u", "Ångström"),
    ];

    let exact_workspace = resolve_find_query("WORKSPACE-ALPHA", &state, &workspaces, &sessions);
    assert!(matches!(
        exact_workspace,
        FindResolution::Unique(FindMatch {
            target: NavigationTarget::Workspace { ref workspace_id },
            ..
        }) if workspace_id == "workspace-alpha"
    ));

    let ambiguous_panes = resolve_find_query("api", &state, &workspaces, &sessions);
    let FindResolution::Ambiguous(pane_matches) = ambiguous_panes else {
        panic!("colliding normalized pane labels must remain explicit ambiguity");
    };
    assert_eq!(pane_matches.len(), 2);
    assert!(
        pane_matches
            .iter()
            .all(|found| matches!(&found.target, NavigationTarget::Pane(_)))
    );

    let ambiguous_sessions = resolve_find_query("BUILD", &state, &workspaces, &sessions);
    let FindResolution::Ambiguous(session_matches) = ambiguous_sessions else {
        panic!("same normalized session labels must remain explicit ambiguity");
    };
    let ids: Vec<&str> = session_matches
        .iter()
        .map(|found| match &found.target {
            NavigationTarget::Session { session_id, .. } => session_id.as_str(),
            _ => panic!("expected session match"),
        })
        .collect();
    assert_eq!(ids, vec!["session-build-1", "session-build-2"]);

    let unicode = resolve_find_query("ångström", &state, &workspaces, &sessions);
    assert!(matches!(
        unicode,
        FindResolution::Unique(FindMatch {
            target: NavigationTarget::Session { ref session_id, .. },
            ..
        }) if session_id == "session-unicode"
    ));
}

#[test]
fn t092_keyboard_paths_create_split_focus_resize_and_close_without_identity_rewrite() {
    let mut state = WorkbenchState::new();
    let first = state.create_pane(
        "first",
        Some("workspace-1".into()),
        Some("session-1".into()),
        PaneSize::new(80, 24),
    );
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Char('n'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 2);
    let created = state.selected_pane().unwrap();
    let created_state = state.pane(created).unwrap();
    assert_eq!(
        created_state.canonical_workspace_id.as_deref(),
        Some("workspace-1")
    );
    assert_eq!(
        created_state.canonical_winds_session_id.as_deref(),
        Some("session-1")
    );

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Char('h'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert_eq!(state.panes().len(), 3);
    let split = state.selected_pane().unwrap();
    assert_eq!(
        state.pane(split).unwrap().split_axis,
        Some(SplitAxis::Horizontal)
    );

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Left, KeyModifiers::ALT),
        )
        .unwrap();
    assert_eq!(state.selected_pane(), Some(created));

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT),
        )
        .unwrap();
    assert_eq!(state.pane(created).unwrap().size, PaneSize::new(81, 24));

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Char('w'), KeyModifiers::CONTROL),
        )
        .unwrap();
    assert!(state.pane(created).is_none());
    assert_eq!(
        state.pane(first).unwrap().canonical_workspace_id.as_deref(),
        Some("workspace-1")
    );
    assert_eq!(
        state
            .pane(first)
            .unwrap()
            .canonical_winds_session_id
            .as_deref(),
        Some("session-1")
    );
}

#[test]
fn t092_search_selection_resolves_to_canonical_id_and_never_guesses_ambiguity() {
    let mut state = WorkbenchState::new();
    state.create_pane("shell", None, None, PaneSize::new(80, 24));
    let sessions = vec![
        session("session-2", "workstream-2", "Deploy"),
        session("session-1", "workstream-1", "deploy"),
    ];
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Char('f'), KeyModifiers::CONTROL),
        )
        .unwrap();
    for character in "deploy".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &sessions,
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    let effect = navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Enter, KeyModifiers::NONE),
        )
        .unwrap();
    assert!(matches!(
        effect,
        NavigationEffect::Find(FindResolution::Ambiguous(ref matches)) if matches.len() == 2
    ));
    assert!(navigation.selected_canonical_target().is_none());

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Char('f'), KeyModifiers::CONTROL),
        )
        .unwrap();
    for character in "session-1".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &sessions,
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Enter, KeyModifiers::NONE),
        )
        .unwrap();
    assert!(matches!(
        navigation.selected_canonical_target(),
        Some(NavigationTarget::Session {
            session_id,
            workstream_id
        }) if session_id == "session-1" && workstream_id == "workstream-1"
    ));

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Char('f'), KeyModifiers::CONTROL),
        )
        .unwrap();
    for character in "deploy".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &sessions,
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    let effect = navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Enter, KeyModifiers::NONE),
        )
        .unwrap();
    assert!(matches!(
        effect,
        NavigationEffect::Find(FindResolution::Ambiguous(ref matches)) if matches.len() == 2
    ));
    assert!(navigation.selected_canonical_target().is_none());

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Char('f'), KeyModifiers::CONTROL),
        )
        .unwrap();
    for character in "missing".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &sessions,
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    let effect = navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &sessions,
            key(KeyCode::Enter, KeyModifiers::NONE),
        )
        .unwrap();
    assert_eq!(effect, NavigationEffect::Find(FindResolution::NotFound));
    assert!(navigation.selected_canonical_target().is_none());
}

#[test]
fn t092_pointer_focus_has_keyboard_equivalent_and_overlapping_hits_fail_closed() {
    let mut state = WorkbenchState::new();
    let first = state.create_pane("first", None, None, PaneSize::new(80, 24));
    let second = state.create_pane("second", None, None, PaneSize::new(80, 24));
    state.focus_pane(first);
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();

    navigation.set_hit_regions(vec![
        PaneHitRegion {
            pane_id: first,
            column: 0,
            row: 0,
            width: 10,
            height: 10,
        },
        PaneHitRegion {
            pane_id: second,
            column: 10,
            row: 0,
            width: 10,
            height: 10,
        },
    ]);
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 12,
                row: 4,
                modifiers: KeyModifiers::NONE,
            }),
        )
        .unwrap();
    assert_eq!(state.selected_pane(), Some(second));

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Left, KeyModifiers::ALT),
        )
        .unwrap();
    assert_eq!(state.selected_pane(), Some(first));

    navigation.set_hit_regions(vec![
        PaneHitRegion {
            pane_id: first,
            column: 0,
            row: 0,
            width: 20,
            height: 10,
        },
        PaneHitRegion {
            pane_id: second,
            column: 0,
            row: 0,
            width: 20,
            height: 10,
        },
    ]);
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 5,
                row: 5,
                modifiers: KeyModifiers::NONE,
            }),
        )
        .unwrap();
    assert_eq!(state.selected_pane(), Some(first));
}

#[test]
fn t092_host_resize_preserves_canonical_identity_and_never_revives_ownership() {
    let mut state = WorkbenchState::new();
    let pane = state.restore_presentation(super::super::PanePresentationMetadata {
        display_title: "restored".into(),
        canonical_workspace_id: Some("workspace-r".into()),
        canonical_winds_session_id: Some("session-r".into()),
        size: PaneSize::new(80, 24),
    });
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();

    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            Event::Resize(120, 40),
        )
        .unwrap();

    let pane = state.pane(pane).unwrap();
    assert_eq!(pane.size, PaneSize::new(120, 40));
    assert_eq!(pane.lifecycle, PaneLifecycleView::OwnershipLost);
    assert_eq!(pane.canonical_workspace_id.as_deref(), Some("workspace-r"));
    assert_eq!(
        pane.canonical_winds_session_id.as_deref(),
        Some("session-r")
    );
}

#[test]
fn t092_shell_events_preserve_explicit_multiline_and_control_boundaries() {
    let mut state = WorkbenchState::new();
    state.create_pane("shell", None, None, PaneSize::new(80, 24));
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();

    for character in "echo λ".chars() {
        navigation
            .handle_event(
                &mut state,
                &mut terminals,
                &mut editor,
                &[],
                &[],
                key(KeyCode::Char(character), KeyModifiers::NONE),
            )
            .unwrap();
    }
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            key(KeyCode::Enter, KeyModifiers::SHIFT),
        )
        .unwrap();
    navigation
        .handle_event(
            &mut state,
            &mut terminals,
            &mut editor,
            &[],
            &[],
            Event::Paste("printf 'ok'\nwhoami".into()),
        )
        .unwrap();

    assert_eq!(
        editor.lines(),
        &[
            "echo λ".to_owned(),
            "printf 'ok'".to_owned(),
            "whoami".to_owned(),
        ]
    );

    let cr_paste = navigation.handle_event(
        &mut state,
        &mut terminals,
        &mut editor,
        &[],
        &[],
        Event::Paste("one\r\ntwo".into()),
    );
    assert!(cr_paste.is_err());
}

#[derive(Default)]
struct FakeEventSource {
    events: VecDeque<Option<Event>>,
    waits: Vec<Duration>,
}

impl HostEventSource for FakeEventSource {
    fn next_event(&mut self, wait: Duration) -> Result<Option<Event>> {
        self.waits.push(wait);
        Ok(self.events.pop_front().unwrap_or(None))
    }
}

#[test]
fn t092_event_loop_uses_nonzero_blocking_wait_and_renders_only_on_events() {
    let mut state = WorkbenchState::new();
    state.create_pane("shell", None, None, PaneSize::new(80, 24));
    let mut terminals = WorkbenchTerminals::new();
    let mut editor = WorkbenchShellEditor::new();
    let mut navigation = WorkbenchNavigation::new();
    let mut source = FakeEventSource {
        events: VecDeque::from([None, Some(key(KeyCode::Char('q'), KeyModifiers::CONTROL))]),
        waits: Vec::new(),
    };
    let mut renders = 0usize;

    run_host_event_loop(
        &mut navigation,
        &mut state,
        &mut terminals,
        &mut editor,
        (&[], &[]),
        &mut source,
        |_, _, _| {
            renders += 1;
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(source.waits, vec![HOST_EVENT_WAIT, HOST_EVENT_WAIT]);
    assert!(source.waits.iter().all(|wait| !wait.is_zero()));
    assert_eq!(renders, 2);
}
