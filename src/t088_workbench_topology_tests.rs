use super::{
    PaneId, PaneLifecycleView, PanePresentationMetadata, PaneSize, SplitAxis, WorkbenchState,
};
use std::collections::HashSet;

fn size(columns: u16, rows: u16) -> PaneSize {
    PaneSize::new(columns, rows)
}

#[test]
fn topology_transitions_are_deterministic_and_selection_is_explicit() {
    let mut state = WorkbenchState::new();
    let first = state.create_pane(
        "first",
        Some("workspace-a".to_owned()),
        Some("session-a".to_owned()),
        size(80, 24),
    );
    let second = state.create_pane("second", None, None, size(100, 30));

    assert_eq!(state.selected_pane(), Some(first));
    assert_eq!(state.selected_dispatch_candidate(), Some(first));

    let split = state
        .split_pane(first, SplitAxis::Horizontal, "split")
        .expect("split an existing pane");
    assert_eq!(state.selected_pane(), Some(split));
    assert_eq!(
        state.panes().iter().map(|pane| pane.pane_id).collect::<Vec<_>>(),
        vec![first, split, second]
    );
    assert_eq!(state.pane(split).unwrap().split_from, Some(first));
    assert_eq!(state.pane(split).unwrap().split_axis, Some(SplitAxis::Horizontal));

    assert!(state.resize_pane(split, size(120, 40)));
    assert_eq!(state.pane(split).unwrap().size, size(120, 40));
    assert!(state.move_pane_to_index(second, 0));
    assert_eq!(
        state.panes().iter().map(|pane| pane.pane_id).collect::<Vec<_>>(),
        vec![second, first, split]
    );

    assert!(state.focus_pane(first));
    assert_eq!(state.selected_dispatch_candidate(), Some(first));
    assert!(state.close_pane(first));
    assert_eq!(state.selected_pane(), Some(split));
    assert_eq!(state.pane(split).unwrap().split_from, None);
    assert_eq!(state.pane(split).unwrap().split_axis, None);

    assert!(state.close_pane(split));
    assert_eq!(state.selected_pane(), Some(second));
    assert!(state.close_pane(second));
    assert_eq!(state.selected_pane(), None);
    assert_eq!(state.selected_dispatch_candidate(), None);
}

#[test]
fn layout_and_display_mutations_never_change_canonical_associations() {
    let mut state = WorkbenchState::new();
    let pane = state.create_pane(
        "Presentation",
        Some("workspace-stable".to_owned()),
        Some("session-stable".to_owned()),
        size(80, 24),
    );
    let sibling = state.create_pane("sibling", None, None, size(70, 20));

    assert!(state.rename_pane(pane, "Renamed presentation only"));
    assert!(state.resize_pane(pane, size(132, 45)));
    assert!(state.move_pane_to_index(pane, 1));
    let split = state
        .split_pane(pane, SplitAxis::Vertical, "derived presentation")
        .expect("split pane");

    let original = state.pane(pane).unwrap();
    assert_eq!(original.display_title, "Renamed presentation only");
    assert_eq!(original.canonical_workspace_id.as_deref(), Some("workspace-stable"));
    assert_eq!(
        original.canonical_winds_session_id.as_deref(),
        Some("session-stable")
    );

    let derived = state.pane(split).unwrap();
    assert_eq!(derived.canonical_workspace_id, original.canonical_workspace_id);
    assert_eq!(
        derived.canonical_winds_session_id,
        original.canonical_winds_session_id
    );
    assert_eq!(state.pane(sibling).unwrap().canonical_workspace_id, None);
}

#[test]
fn restored_presentation_metadata_never_establishes_live_ownership() {
    let mut state = WorkbenchState::new();
    let restored = state.restore_presentation(PanePresentationMetadata {
        display_title: "LIVE shell from yesterday".to_owned(),
        canonical_workspace_id: Some("workspace-restored".to_owned()),
        canonical_winds_session_id: Some("session-restored".to_owned()),
        size: size(90, 28),
    });

    let pane = state.pane(restored).unwrap();
    assert_eq!(pane.lifecycle, PaneLifecycleView::OwnershipLost);
    assert_ne!(pane.lifecycle, PaneLifecycleView::Live);
    assert_eq!(pane.display_title, "LIVE shell from yesterday");
    assert_eq!(pane.canonical_workspace_id.as_deref(), Some("workspace-restored"));
    assert_eq!(
        pane.canonical_winds_session_id.as_deref(),
        Some("session-restored")
    );
}

#[test]
fn lifecycle_views_keep_live_and_non_live_truth_distinct() {
    assert_ne!(PaneLifecycleView::Live, PaneLifecycleView::Exited);
    assert_ne!(PaneLifecycleView::Live, PaneLifecycleView::Stopped);
    assert_ne!(PaneLifecycleView::Live, PaneLifecycleView::OwnershipLost);
    assert_ne!(PaneLifecycleView::Live, PaneLifecycleView::Error);
    assert_ne!(PaneLifecycleView::Exited, PaneLifecycleView::Stopped);
    assert_ne!(PaneLifecycleView::Stopped, PaneLifecycleView::OwnershipLost);
    assert_ne!(PaneLifecycleView::OwnershipLost, PaneLifecycleView::Error);

    let mut state = WorkbenchState::new();
    let pane = state.create_pane("inert", None, None, size(80, 24));
    assert_eq!(state.pane(pane).unwrap().lifecycle, PaneLifecycleView::Stopped);
}

#[test]
fn fifty_plus_inert_panes_remain_bounded_presentation_state_without_live_shells() {
    let mut first = WorkbenchState::new();
    for index in 0..64 {
        first.create_pane(
            format!("pane-{index}"),
            Some(format!("workspace-{}", index % 4)),
            Some(format!("session-{}", index % 8)),
            size(80 + (index % 7) as u16, 24 + (index % 5) as u16),
        );
    }
    assert_eq!(first.panes().len(), 64);
    assert!(
        first
            .panes()
            .iter()
            .all(|pane| pane.lifecycle != PaneLifecycleView::Live)
    );

    let mut second = first.clone();
    let moves = [63usize, 17, 31, 5, 48, 12];
    for (destination, source_index) in moves.into_iter().enumerate() {
        let left_id = first.panes()[source_index].pane_id;
        let right_id = second.panes()[source_index].pane_id;
        assert_eq!(left_id, right_id);
        assert!(first.move_pane_to_index(left_id, destination));
        assert!(second.move_pane_to_index(right_id, destination));
    }
    assert_eq!(first, second);
}

#[test]
fn unicode_case_and_colliding_titles_never_define_pane_identity() {
    let mut state = WorkbenchState::new();
    let titles = ["Pane", "pane", "Pane", "Cafe\u{301}", "Café", "终端", "终端"];
    let ids = titles
        .iter()
        .map(|title| state.create_pane(*title, None, None, size(80, 24)))
        .collect::<Vec<_>>();

    let unique = ids.iter().copied().collect::<HashSet<PaneId>>();
    assert_eq!(unique.len(), ids.len());
    assert_eq!(
        state
            .panes()
            .iter()
            .map(|pane| pane.display_title.as_str())
            .collect::<Vec<_>>(),
        titles
    );
}

#[test]
fn unknown_pane_operations_fail_without_changing_selection_or_topology() {
    let mut state = WorkbenchState::new();
    let known = state.create_pane("known", None, None, size(80, 24));
    let unknown = PaneId(u64::MAX);
    let before = state.clone();

    assert!(!state.focus_pane(unknown));
    assert!(!state.resize_pane(unknown, size(1, 1)));
    assert!(!state.rename_pane(unknown, "ignored"));
    assert!(!state.move_pane_to_index(unknown, 0));
    assert!(!state.close_pane(unknown));
    assert_eq!(state.split_pane(unknown, SplitAxis::Vertical, "ignored"), None);
    assert_eq!(state, before);
    assert_eq!(state.selected_dispatch_candidate(), Some(known));
}
