use super::{
    Direction, Edge, LayoutNode, LayoutTemplateNode, LayoutTemplateStructure, LayoutTemplateTab,
    MultiplexerTopology, PanePlacement, SplitAxis, SplitRatioBps,
};
use crate::multiplexer::domain::{
    MultiplexerErrorKind, MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration,
};

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).unwrap()
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).unwrap()
}

fn ratio(value: u16) -> SplitRatioBps {
    SplitRatioBps::new(value).unwrap()
}

fn topology() -> (MultiplexerTopology, MultiplexerWorkspaceId, TabId, PaneId) {
    let workspace_id = workspace(1);
    let tab_id = tab(2);
    let pane_id = pane(3);
    let mut topology = MultiplexerTopology::empty();
    topology
        .create_workspace(
            topology.generation(),
            workspace_id,
            "duplicate".into(),
            tab_id,
            "duplicate".into(),
            pane_id,
        )
        .unwrap();
    (topology, workspace_id, tab_id, pane_id)
}

fn panes(node: &LayoutNode, output: &mut Vec<PaneId>) {
    match node {
        LayoutNode::Pane(pane_id) => output.push(*pane_id),
        LayoutNode::Split { first, second, .. } => {
            panes(first, output);
            panes(second, output);
        }
    }
}

#[test]
fn t163_workspace_and_tab_identity_survive_duplicate_aliases_renames_moves_and_focus() {
    let (mut topology, first_workspace, first_tab, _) = topology();
    let second_workspace = workspace(4);
    let second_tab = tab(5);
    let second_pane = pane(6);

    let generation = topology
        .create_workspace(
            topology.generation(),
            second_workspace,
            "duplicate".into(),
            second_tab,
            "duplicate".into(),
            second_pane,
        )
        .unwrap();
    assert_eq!(topology.workspaces().len(), 2);
    assert_ne!(first_workspace, second_workspace);
    assert_eq!(topology.focused_workspace_id(), Some(second_workspace));

    let generation = topology
        .rename_workspace(generation, first_workspace, "renamed".into())
        .unwrap();
    let generation = topology
        .move_workspace(generation, second_workspace, 0)
        .unwrap();
    let generation = topology
        .focus_workspace(generation, first_workspace)
        .unwrap();

    let third_tab = tab(7);
    let third_pane = pane(8);
    let generation = topology
        .create_tab(
            generation,
            first_workspace,
            third_tab,
            "duplicate".into(),
            third_pane,
        )
        .unwrap();
    let generation = topology
        .rename_tab(generation, first_workspace, first_tab, "same".into())
        .unwrap();
    let generation = topology
        .rename_tab(generation, first_workspace, third_tab, "same".into())
        .unwrap();
    let generation = topology
        .move_tab(generation, first_workspace, third_tab, 0)
        .unwrap();
    topology
        .focus_tab(generation, first_workspace, first_tab)
        .unwrap();

    assert_eq!(topology.workspaces()[0].id, second_workspace);
    assert_eq!(
        topology.workspace(first_workspace).unwrap().focused_tab_id,
        first_tab
    );
    assert_eq!(
        topology.workspace(first_workspace).unwrap().tabs[0].id,
        third_tab
    );
}

#[test]
fn t163_stale_generation_rejects_without_partial_mutation_or_retargeting() {
    let (mut topology, workspace_id, tab_id, pane_id) = topology();
    let stale = topology.generation();
    let fresh = topology
        .split_pane(
            stale,
            workspace_id,
            tab_id,
            pane_id,
            pane(9),
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();
    let snapshot = topology.clone();

    assert_eq!(
        topology.rename_workspace(stale, workspace_id, "stale".into()),
        Err(MultiplexerErrorKind::StaleTopologyGeneration)
    );
    assert_eq!(topology, snapshot);
    assert_eq!(topology.generation(), fresh);
}

#[test]
fn t163_split_preserves_existing_pane_identity_and_rejects_retired_reuse() {
    let (mut topology, workspace_id, tab_id, first_pane) = topology();
    let second_pane = pane(10);
    let generation = topology
        .split_pane(
            topology.generation(),
            workspace_id,
            tab_id,
            first_pane,
            second_pane,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(6_000),
        )
        .unwrap();

    let tab = topology.tab(workspace_id, tab_id).unwrap();
    let mut observed = Vec::new();
    panes(&tab.root, &mut observed);
    assert_eq!(observed, vec![first_pane, second_pane]);

    let generation = topology
        .close_pane(generation, workspace_id, tab_id, second_pane)
        .unwrap();
    let snapshot = topology.clone();
    assert_eq!(
        topology.split_pane(
            generation,
            workspace_id,
            tab_id,
            first_pane,
            second_pane,
            SplitAxis::Vertical,
            PanePlacement::Before,
            ratio(5_000),
        ),
        Err(MultiplexerErrorKind::IdentityReuse)
    );
    assert_eq!(topology, snapshot);
}

#[test]
fn t163_closing_last_pane_or_last_tab_requires_explicit_parent_close() {
    let (mut topology, workspace_id, tab_id, pane_id) = topology();
    let generation = topology.generation();
    assert_eq!(
        topology.close_pane(generation, workspace_id, tab_id, pane_id),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
    assert_eq!(
        topology.close_tab(generation, workspace_id, tab_id),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
    assert_eq!(topology.generation(), generation);
}

#[test]
fn t163_swap_move_resize_zoom_and_focus_are_exact_and_deterministic() {
    let (mut topology, workspace_id, first_tab, first_pane) = topology();
    let second_pane = pane(11);
    let third_pane = pane(12);
    let second_tab = tab(13);
    let destination_pane = pane(14);

    let generation = topology
        .split_pane(
            topology.generation(),
            workspace_id,
            first_tab,
            first_pane,
            second_pane,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();
    let generation = topology
        .split_pane(
            generation,
            workspace_id,
            first_tab,
            second_pane,
            third_pane,
            SplitAxis::Vertical,
            PanePlacement::After,
            ratio(4_000),
        )
        .unwrap();
    let generation = topology
        .swap_panes(generation, workspace_id, first_tab, first_pane, third_pane)
        .unwrap();
    let generation = topology
        .resize_split_between(
            generation,
            workspace_id,
            first_tab,
            third_pane,
            first_pane,
            ratio(7_000),
        )
        .unwrap();
    let generation = topology
        .toggle_zoom(generation, workspace_id, first_tab, first_pane)
        .unwrap();
    let generation = topology
        .focus_pane(generation, workspace_id, first_tab, first_pane)
        .unwrap();
    let generation = topology
        .create_tab(
            generation,
            workspace_id,
            second_tab,
            "target".into(),
            destination_pane,
        )
        .unwrap();
    topology
        .move_pane(
            generation,
            workspace_id,
            first_tab,
            second_pane,
            second_tab,
            destination_pane,
            SplitAxis::Vertical,
            PanePlacement::Before,
            ratio(5_000),
        )
        .unwrap();

    let source = topology.tab(workspace_id, first_tab).unwrap();
    let destination = topology.tab(workspace_id, second_tab).unwrap();
    assert!(!super::contains_pane(&source.root, second_pane));
    assert!(super::contains_pane(&destination.root, second_pane));
}

#[test]
fn t163_move_refuses_to_empty_a_source_tab() {
    let (mut topology, workspace_id, first_tab, first_pane) = topology();
    let second_tab = tab(15);
    let second_pane = pane(16);
    let generation = topology
        .create_tab(
            topology.generation(),
            workspace_id,
            second_tab,
            "second".into(),
            second_pane,
        )
        .unwrap();

    assert_eq!(
        topology.move_pane(
            generation,
            workspace_id,
            first_tab,
            first_pane,
            second_tab,
            second_pane,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        ),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
}

#[test]
fn t163_split_geometry_uses_wide_intermediates_and_preserves_requested_ratios() {
    let (mut topology, workspace_id, tab_id, first) = topology();
    let second = pane(19);
    topology
        .split_pane(
            topology.generation(),
            workspace_id,
            tab_id,
            first,
            second,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();

    let tab = topology.tab(workspace_id, tab_id).unwrap();
    let rects = super::pane_rects(&tab.root);
    let first_rect = rects
        .iter()
        .find(|(pane_id, _)| *pane_id == first)
        .unwrap()
        .1;
    let second_rect = rects
        .iter()
        .find(|(pane_id, _)| *pane_id == second)
        .unwrap()
        .1;

    assert_eq!(first_rect.left, 0);
    assert_eq!(first_rect.right, 500_000);
    assert_eq!(second_rect.left, 500_000);
    assert_eq!(second_rect.right, 1_000_000);

    let third = pane(18);
    topology
        .split_pane(
            topology.generation(),
            workspace_id,
            tab_id,
            second,
            third,
            SplitAxis::Vertical,
            PanePlacement::After,
            ratio(9_000),
        )
        .unwrap();

    let tab = topology.tab(workspace_id, tab_id).unwrap();
    let rects = super::pane_rects(&tab.root);
    let second_rect = rects
        .iter()
        .find(|(pane_id, _)| *pane_id == second)
        .unwrap()
        .1;
    let third_rect = rects
        .iter()
        .find(|(pane_id, _)| *pane_id == third)
        .unwrap()
        .1;

    assert_eq!(second_rect.top, 0);
    assert_eq!(second_rect.bottom, 900_000);
    assert_eq!(third_rect.top, 900_000);
    assert_eq!(third_rect.bottom, 1_000_000);
}

#[test]
fn t163_directional_navigation_and_edges_are_geometry_based_with_stable_ties() {
    let (mut topology, workspace_id, tab_id, left_top) = topology();
    let right = pane(20);
    let left_bottom = pane(21);
    let generation = topology
        .split_pane(
            topology.generation(),
            workspace_id,
            tab_id,
            left_top,
            right,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();
    topology
        .split_pane(
            generation,
            workspace_id,
            tab_id,
            left_top,
            left_bottom,
            SplitAxis::Vertical,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();

    assert_eq!(
        topology
            .directional_neighbor(workspace_id, tab_id, left_top, Direction::Right)
            .unwrap(),
        Some(right)
    );
    // AD-012-31 minimizes primary-axis distance before orthogonal overlap.
    // The right pane is vertically closer to left_top than left_bottom is.
    assert_eq!(
        topology
            .directional_neighbor(workspace_id, tab_id, left_top, Direction::Down)
            .unwrap(),
        Some(right)
    );
    assert_eq!(
        topology
            .edge_pane(workspace_id, tab_id, Edge::Left)
            .unwrap(),
        left_top
    );
    assert_eq!(
        topology
            .edge_pane(workspace_id, tab_id, Edge::Right)
            .unwrap(),
        right
    );
    assert_eq!(
        topology
            .edge_pane(workspace_id, tab_id, Edge::Bottom)
            .unwrap(),
        left_bottom
    );
}

#[test]
fn t163_same_initial_state_and_operation_sequence_produce_identical_topology() {
    let (first, workspace_id, tab_id, pane_id) = topology();
    let mut left = first.clone();
    let mut right = first;

    for topology in [&mut left, &mut right] {
        let generation = topology
            .split_pane(
                topology.generation(),
                workspace_id,
                tab_id,
                pane_id,
                pane(30),
                SplitAxis::Horizontal,
                PanePlacement::After,
                ratio(5_500),
            )
            .unwrap();
        let generation = topology
            .focus_pane(generation, workspace_id, tab_id, pane(30))
            .unwrap();
        topology
            .toggle_zoom(generation, workspace_id, tab_id, pane(30))
            .unwrap();
    }

    assert_eq!(left, right);
}

#[test]
fn t163_delayed_close_cannot_hit_a_replacement_in_the_same_visual_position() {
    let (mut topology, workspace_id, tab_id, first_pane) = topology();
    let closed = pane(31);
    let generation_before_split = topology.generation();
    let generation = topology
        .split_pane(
            generation_before_split,
            workspace_id,
            tab_id,
            first_pane,
            closed,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();
    let generation = topology
        .close_pane(generation, workspace_id, tab_id, closed)
        .unwrap();
    let replacement = pane(32);
    let generation = topology
        .split_pane(
            generation,
            workspace_id,
            tab_id,
            first_pane,
            replacement,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();

    assert_eq!(
        topology.close_pane(generation_before_split, workspace_id, tab_id, closed),
        Err(MultiplexerErrorKind::StaleTopologyGeneration)
    );
    assert!(super::contains_pane(
        &topology.tab(workspace_id, tab_id).unwrap().root,
        replacement
    ));
    assert_eq!(topology.generation(), generation);
}

#[test]
fn t163_layout_template_structure_contains_no_live_identity_or_process_binding() {
    let template = LayoutTemplateStructure {
        tabs: vec![LayoutTemplateTab {
            alias: "work".into(),
            root: LayoutTemplateNode::Split {
                axis: SplitAxis::Horizontal,
                ratio_bps: ratio(5_000),
                first: Box::new(LayoutTemplateNode::Pane),
                second: Box::new(LayoutTemplateNode::Pane),
            },
        }],
    };

    assert_eq!(template.tabs.len(), 1);
    assert_eq!(template.tabs[0].alias, "work");
    assert!(matches!(
        template.tabs[0].root,
        LayoutTemplateNode::Split { .. }
    ));
}

#[test]
fn t163_split_ratio_is_integer_bounded_and_invalid_values_fail_closed() {
    assert_eq!(SplitRatioBps::new(1_000).unwrap().get(), 1_000);
    assert_eq!(SplitRatioBps::new(9_000).unwrap().get(), 9_000);
    assert_eq!(
        SplitRatioBps::new(999),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
    assert_eq!(
        SplitRatioBps::new(9_001),
        Err(MultiplexerErrorKind::UnsupportedOperation)
    );
}

#[test]
fn t163_workspace_close_retires_all_panes_and_focuses_deterministic_fallback() {
    let (mut topology, first_workspace, _, first_pane) = topology();
    let second_workspace = workspace(40);
    let second_tab = tab(41);
    let second_pane = pane(42);
    let generation = topology
        .create_workspace(
            topology.generation(),
            second_workspace,
            "second".into(),
            second_tab,
            "tab".into(),
            second_pane,
        )
        .unwrap();
    let generation = topology
        .close_workspace(generation, second_workspace)
        .unwrap();

    assert_eq!(topology.focused_workspace_id(), Some(first_workspace));
    assert_eq!(
        topology.create_workspace(
            generation,
            workspace(43),
            "third".into(),
            tab(44),
            "tab".into(),
            second_pane,
        ),
        Err(MultiplexerErrorKind::IdentityReuse)
    );
    assert!(
        topology
            .tab(
                first_workspace,
                topology.workspace(first_workspace).unwrap().focused_tab_id
            )
            .is_ok()
    );
    assert!(first_pane != second_pane);
}

#[test]
fn t163_closed_workspace_identity_cannot_be_resurrected() {
    let (mut topology, workspace_id, _, _) = topology();
    let generation = topology
        .close_workspace(topology.generation(), workspace_id)
        .unwrap();
    let snapshot = topology.clone();

    assert_eq!(
        topology.create_workspace(
            generation,
            workspace_id,
            "replacement".into(),
            tab(60),
            "tab".into(),
            pane(61),
        ),
        Err(MultiplexerErrorKind::IdentityReuse)
    );
    assert_eq!(topology, snapshot);
}

#[test]
fn t163_closed_tab_identity_cannot_be_reused_in_the_same_workspace() {
    let (mut topology, workspace_id, first_tab, _) = topology();
    let closed_tab = tab(62);
    let closed_pane = pane(63);
    let generation = topology
        .create_tab(
            topology.generation(),
            workspace_id,
            closed_tab,
            "closed".into(),
            closed_pane,
        )
        .unwrap();
    let generation = topology
        .close_tab(generation, workspace_id, closed_tab)
        .unwrap();
    let snapshot = topology.clone();

    assert_eq!(
        topology.create_tab(
            generation,
            workspace_id,
            closed_tab,
            "replacement".into(),
            pane(64),
        ),
        Err(MultiplexerErrorKind::IdentityReuse)
    );
    assert_eq!(topology, snapshot);

    assert_eq!(
        topology.focus_tab(generation, workspace_id, first_tab),
        Ok(TopologyGeneration::new(generation.get() + 1).unwrap())
    );
}

#[test]
fn t163_tab_identity_is_scoped_to_its_workspace() {
    let (mut topology, first_workspace, first_tab, _) = topology();
    let second_workspace = workspace(65);
    let second_pane = pane(66);

    topology
        .create_workspace(
            topology.generation(),
            second_workspace,
            "second".into(),
            first_tab,
            "same-scoped-tab-id".into(),
            second_pane,
        )
        .unwrap();

    assert!(topology.tab(first_workspace, first_tab).is_ok());
    assert!(topology.tab(second_workspace, first_tab).is_ok());
}

#[test]
fn t163_topology_generation_exhaustion_fails_before_mutation() {
    let (mut topology, workspace_id, _, _) = topology();
    topology.generation = TopologyGeneration::new(u64::MAX).unwrap();
    let snapshot = topology.clone();
    assert_eq!(
        topology.rename_workspace(
            TopologyGeneration::new(u64::MAX).unwrap(),
            workspace_id,
            "never".into(),
        ),
        Err(MultiplexerErrorKind::TopologyGenerationExhausted)
    );
    assert_eq!(topology, snapshot);
}

#[test]
fn t163_edge_top_is_stable_for_equal_extremes() {
    let (mut topology, workspace_id, tab_id, first) = topology();
    let second = pane(50);
    topology
        .split_pane(
            topology.generation(),
            workspace_id,
            tab_id,
            first,
            second,
            SplitAxis::Horizontal,
            PanePlacement::After,
            ratio(5_000),
        )
        .unwrap();
    let expected = first.min(second);
    assert_eq!(
        topology.edge_pane(workspace_id, tab_id, Edge::Top).unwrap(),
        expected
    );
}

#[test]
fn t163_unknown_exact_targets_fail_without_generation_change() {
    let (mut topology, workspace_id, tab_id, _) = topology();
    let generation = topology.generation();
    assert_eq!(
        topology.focus_pane(generation, workspace_id, tab_id, pane(99)),
        Err(MultiplexerErrorKind::UnknownPane)
    );
    assert_eq!(topology.generation(), generation);
}
