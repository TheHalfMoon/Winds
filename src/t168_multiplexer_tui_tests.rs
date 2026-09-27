use super::*;
use crate::multiplexer::domain::{MultiplexerWorkspaceId, PaneId, TabId, TopologyGeneration};
use crate::persistent_runtime::domain::OwnerGenerationId;
use crate::persistent_runtime::protocol::{
    MultiplexerEventSubscriptionAckV2, MultiplexerEventV2, MultiplexerSnapshotV2,
    MultiplexerSubscriptionBoundaryV2, MultiplexerSubscriptionStreamV2, ProtocolLayoutNodeV2,
    ProtocolPaneClosePolicy, ProtocolPanePlacement, ProtocolSplitAxis, ProtocolTabSnapshotV2,
    ProtocolWorkspaceSnapshotV2, SubscribeMultiplexerEventsV2, TopologyOperationV2,
};

fn owner_generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).unwrap()
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).unwrap()
}

fn generation(value: u64) -> TopologyGeneration {
    TopologyGeneration::new(value).unwrap()
}

fn pane_node(pane_id: PaneId) -> ProtocolLayoutNodeV2 {
    ProtocolLayoutNodeV2::Pane { pane_id }
}

fn split_node(first: PaneId, second: PaneId) -> ProtocolLayoutNodeV2 {
    ProtocolLayoutNodeV2::Split {
        axis: ProtocolSplitAxis::Horizontal,
        ratio_basis_points: 5_000,
        first: Box::new(pane_node(first)),
        second: Box::new(pane_node(second)),
    }
}

fn tab_snapshot(
    tab_id: TabId,
    alias: &str,
    root: ProtocolLayoutNodeV2,
    focused_pane_id: PaneId,
    zoomed_pane_id: Option<PaneId>,
) -> ProtocolTabSnapshotV2 {
    ProtocolTabSnapshotV2 {
        tab_id,
        alias: alias.to_owned(),
        root,
        focused_pane_id,
        zoomed_pane_id,
    }
}

fn workspace_snapshot(
    topology_generation: u64,
    multiplexer_workspace_id: MultiplexerWorkspaceId,
    alias: &str,
    focused_tab_id: TabId,
    tabs: Vec<ProtocolTabSnapshotV2>,
) -> MultiplexerSnapshotV2 {
    MultiplexerSnapshotV2::Workspace {
        snapshot: ProtocolWorkspaceSnapshotV2 {
            multiplexer_workspace_id,
            alias: alias.to_owned(),
            topology_generation: generation(topology_generation),
            focused_tab_id,
            tabs,
        },
    }
}

fn detailed_snapshot(topology_generation: u64) -> MultiplexerSnapshotV2 {
    let workspace_id = workspace(1);
    let first_tab = tab(11);
    let second_tab = tab(12);
    workspace_snapshot(
        topology_generation,
        workspace_id,
        "workspace-alpha",
        first_tab,
        vec![
            tab_snapshot(
                first_tab,
                "shell",
                split_node(pane(21), pane(22)),
                pane(21),
                Some(pane(22)),
            ),
            tab_snapshot(
                second_tab,
                "logs",
                pane_node(pane(23)),
                pane(23),
                None,
            ),
        ],
    )
}

fn subscription(workspace_id: MultiplexerWorkspaceId) -> SubscribeMultiplexerEventsV2 {
    SubscribeMultiplexerEventsV2 {
        stream: MultiplexerSubscriptionStreamV2::Topology,
        multiplexer_workspace_id: Some(workspace_id),
    }
}

fn subscription_ack(
    workspace_id: MultiplexerWorkspaceId,
    topology_generation: u64,
) -> MultiplexerEventSubscriptionAckV2 {
    MultiplexerEventSubscriptionAckV2 {
        stream: MultiplexerSubscriptionStreamV2::Topology,
        multiplexer_workspace_id: Some(workspace_id),
        boundary: MultiplexerSubscriptionBoundaryV2::Topology {
            topology_generation: generation(topology_generation),
        },
    }
}

fn pane_target(tab_id: TabId, pane_id: PaneId) -> TuiTopologyTarget {
    TuiTopologyTarget::Pane {
        multiplexer_workspace_id: workspace(1),
        tab_id,
        pane_id,
    }
}

#[test]
fn t168_tui_requires_a_current_trusted_projection_and_invalidates_after_event() {
    let workspace_id = workspace(1);
    let mut source = TopologyProjection::new(owner_generation(1));
    assert_eq!(
        TuiTopologyProjection::from_trusted_projection(&source),
        Err(TuiTopologyProjectionError::UntrustedSnapshot)
    );

    let subscription = subscription(workspace_id);
    source
        .accept_subscription_ack(&subscription, &subscription_ack(workspace_id, 7))
        .unwrap();
    let snapshot = detailed_snapshot(7);
    source.accept_snapshot(&snapshot).unwrap();

    let tui = TuiTopologyProjection::from_trusted_projection(&source).unwrap();
    assert_eq!(tui.topology_generation(), generation(7));
    assert!(!tui.changes_canonical_authority());
    assert_eq!(tui.authority_label(), "READ_ONLY_TOPOLOGY");

    source
        .accept_event(&MultiplexerEventV2::TopologyChanged {
            multiplexer_workspace_id: workspace_id,
            topology_generation: generation(8),
        })
        .unwrap();
    assert_eq!(
        TuiTopologyProjection::from_trusted_projection(&source),
        Err(TuiTopologyProjectionError::UntrustedSnapshot)
    );
}

#[test]
fn t168_duplicate_labels_never_override_immutable_canonical_ids() {
    let workspace_id = workspace(2);
    let tab_id = tab(31);
    let pane_id = pane(41);
    let snapshot = workspace_snapshot(
        4,
        workspace_id,
        "duplicate",
        tab_id,
        vec![tab_snapshot(
            tab_id,
            "duplicate",
            pane_node(pane_id),
            pane_id,
            None,
        )],
    );
    let tui = TuiTopologyProjection::from_snapshot(&snapshot).unwrap();

    let TuiTopologyFindResolution::Ambiguous(matches) = tui.find("duplicate") else {
        panic!("duplicate labels must remain explicit ambiguity");
    };
    assert_eq!(matches.len(), 2);

    assert_eq!(
        tui.find(&workspace_id.to_string()),
        TuiTopologyFindResolution::Unique(TuiTopologyTarget::Workspace {
            multiplexer_workspace_id: workspace_id,
        })
    );
    assert_eq!(
        tui.find(&tab_id.to_string()),
        TuiTopologyFindResolution::Unique(TuiTopologyTarget::Tab {
            multiplexer_workspace_id: workspace_id,
            tab_id,
        })
    );
    assert_eq!(
        tui.find(&pane_id.to_string()),
        TuiTopologyFindResolution::Unique(TuiTopologyTarget::Pane {
            multiplexer_workspace_id: workspace_id,
            tab_id,
            pane_id,
        })
    );
}

#[test]
fn t168_pointer_and_keyboard_focus_resolve_to_the_same_exact_target() {
    let tui = TuiTopologyProjection::from_snapshot(&detailed_snapshot(7)).unwrap();
    let target = pane_target(tab(11), pane(22));
    let regions = vec![
        TuiTopologyHitRegion {
            target: pane_target(tab(11), pane(21)),
            column: 0,
            row: 0,
            width: 10,
            height: 10,
        },
        TuiTopologyHitRegion {
            target,
            column: 10,
            row: 0,
            width: 10,
            height: 10,
        },
    ];

    let keyboard = tui
        .keyboard_intent(TuiTopologyCommand::Focus(target))
        .unwrap();
    let pointer = tui.pointer_focus_intent(&regions, 12, 3).unwrap();
    assert_eq!(pointer, keyboard);
    assert_eq!(pointer.expected_topology_generation, generation(7));

    let overlapping = vec![
        TuiTopologyHitRegion {
            target: pane_target(tab(11), pane(21)),
            column: 0,
            row: 0,
            width: 20,
            height: 10,
        },
        TuiTopologyHitRegion {
            target,
            column: 0,
            row: 0,
            width: 20,
            height: 10,
        },
    ];
    assert_eq!(tui.pointer_target(&overlapping, 5, 5), None);
    assert_eq!(
        tui.pointer_focus_intent(&overlapping, 5, 5),
        Err(TuiTopologyProjectionError::UnknownTarget)
    );
}

#[test]
fn t168_keyboard_primary_topology_actions_are_generation_bound_intents_only() {
    let tui = TuiTopologyProjection::from_snapshot(&detailed_snapshot(7)).unwrap();
    let workspace_id = workspace(1);
    let tab_one = tab(11);
    let tab_two = tab(12);
    let pane_one = pane(21);
    let pane_two = pane(22);
    let pane_three = pane(23);
    let new_workspace = workspace(9);
    let new_tab = tab(19);
    let new_pane = pane(29);

    let commands = vec![
        TuiTopologyCommand::CreateWorkspace {
            multiplexer_workspace_id: new_workspace,
            alias: "new-workspace".to_owned(),
            first_tab_id: new_tab,
            first_tab_alias: "new-tab".to_owned(),
            first_pane_id: new_pane,
        },
        TuiTopologyCommand::CreateTab {
            multiplexer_workspace_id: workspace_id,
            tab_id: new_tab,
            alias: "new-tab".to_owned(),
            first_pane_id: new_pane,
        },
        TuiTopologyCommand::SplitPane {
            multiplexer_workspace_id: workspace_id,
            tab_id: tab_one,
            target_pane_id: pane_one,
            new_pane_id: new_pane,
            axis: ProtocolSplitAxis::Vertical,
            placement: ProtocolPanePlacement::After,
            ratio_basis_points: 5_000,
        },
        TuiTopologyCommand::Focus(pane_target(tab_one, pane_two)),
        TuiTopologyCommand::MoveWorkspace {
            multiplexer_workspace_id: workspace_id,
            new_index: 0,
        },
        TuiTopologyCommand::MoveTab {
            multiplexer_workspace_id: workspace_id,
            tab_id: tab_two,
            new_index: 0,
        },
        TuiTopologyCommand::MovePane {
            multiplexer_workspace_id: workspace_id,
            source_tab_id: tab_one,
            pane_id: pane_one,
            destination_tab_id: tab_two,
            destination_pane_id: pane_three,
            axis: ProtocolSplitAxis::Horizontal,
            placement: ProtocolPanePlacement::Before,
            ratio_basis_points: 4_000,
        },
        TuiTopologyCommand::ResizeSplit {
            multiplexer_workspace_id: workspace_id,
            tab_id: tab_one,
            first_pane_id: pane_one,
            second_pane_id: pane_two,
            ratio_basis_points: 6_000,
        },
        TuiTopologyCommand::ToggleZoom {
            multiplexer_workspace_id: workspace_id,
            tab_id: tab_one,
            pane_id: pane_two,
        },
        TuiTopologyCommand::Close(TuiTopologyTarget::Workspace {
            multiplexer_workspace_id: workspace_id,
        }),
        TuiTopologyCommand::Close(TuiTopologyTarget::Tab {
            multiplexer_workspace_id: workspace_id,
            tab_id: tab_two,
        }),
        TuiTopologyCommand::Close(pane_target(tab_one, pane_two)),
    ];

    let intents: Vec<_> = commands
        .into_iter()
        .map(|command| tui.keyboard_intent(command).unwrap())
        .collect();
    assert!(
        intents
            .iter()
            .all(|intent| intent.expected_topology_generation == generation(7))
    );
    assert!(matches!(
        intents.last().unwrap().operation,
        TopologyOperationV2::ClosePane {
            policy: ProtocolPaneClosePolicy::DetachView,
            ..
        }
    ));
    assert!(!tui.changes_canonical_authority());
}

#[test]
fn t168_focus_race_keeps_the_original_immutable_target_and_generation() {
    let old = TuiTopologyProjection::from_snapshot(&detailed_snapshot(7)).unwrap();
    let new = TuiTopologyProjection::from_snapshot(&detailed_snapshot(8)).unwrap();
    let target = pane_target(tab(11), pane(21));

    let old_intent = old
        .keyboard_intent(TuiTopologyCommand::Focus(target))
        .unwrap();
    let new_intent = new
        .keyboard_intent(TuiTopologyCommand::Focus(target))
        .unwrap();

    assert_eq!(old_intent.expected_topology_generation, generation(7));
    assert_eq!(new_intent.expected_topology_generation, generation(8));
    assert_eq!(old_intent.operation, new_intent.operation);
}

#[test]
fn t168_large_topology_and_accessibility_rendering_are_deterministic() {
    let workspace_id = workspace(7);
    let mut tabs = Vec::new();
    for index in 0_u8..32 {
        let tab_id = tab(64 + index);
        let pane_id = pane(96 + index);
        tabs.push(tab_snapshot(
            tab_id,
            "duplicate-tab-label",
            pane_node(pane_id),
            pane_id,
            None,
        ));
    }
    let focused_tab_id = tabs[0].tab_id;
    let snapshot = workspace_snapshot(11, workspace_id, "large", focused_tab_id, tabs);
    let tui = TuiTopologyProjection::from_snapshot(&snapshot).unwrap();
    assert_eq!(tui.rows().len(), 65);

    let accessibility = TuiTopologyAccessibility {
        reduced_motion: true,
        high_contrast: true,
        scaled_text: true,
    };
    let first = tui.render_lines(accessibility);
    let second = tui.render_lines(accessibility);
    assert_eq!(first, second);
    assert!(first[0].contains("authority=READ_ONLY_TOPOLOGY"));
    assert!(first[0].contains("reduced_motion=true"));
    assert!(first[0].contains("high_contrast=true"));
    assert!(first[0].contains("scaled_text=true"));
    assert!(first.iter().any(|line| line.contains(&workspace_id.to_string())));
    assert!(first.iter().any(|line| line.contains(&focused_tab_id.to_string())));

    let TuiTopologyFindResolution::Ambiguous(matches) = tui.find("duplicate-tab-label") else {
        panic!("large duplicate-label topology must remain explicit ambiguity");
    };
    assert_eq!(matches.len(), 32);
}

#[test]
fn t168_projection_rejects_duplicate_canonical_pane_identity() {
    let workspace_id = workspace(5);
    let first_tab = tab(51);
    let second_tab = tab(52);
    let duplicate_pane = pane(61);
    let snapshot = workspace_snapshot(
        3,
        workspace_id,
        "collision",
        first_tab,
        vec![
            tab_snapshot(
                first_tab,
                "one",
                pane_node(duplicate_pane),
                duplicate_pane,
                None,
            ),
            tab_snapshot(
                second_tab,
                "two",
                pane_node(duplicate_pane),
                duplicate_pane,
                None,
            ),
        ],
    );

    assert_eq!(
        TuiTopologyProjection::from_snapshot(&snapshot),
        Err(TuiTopologyProjectionError::IdentityCollision)
    );
}
