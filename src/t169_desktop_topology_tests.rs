use super::*;
use crate::persistent_runtime::protocol::{ProtocolTabSnapshotV2, ProtocolWorkspaceSnapshotV2};

fn workspace_id(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).expect("valid workspace id")
}

fn tab_id(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).expect("valid tab id")
}

fn pane_id(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).expect("valid pane id")
}

#[test]
fn t169_desktop_capability_is_tty_independent_and_has_no_renderer_authority() {
    let capability = desktop_topology_capability();
    assert_eq!(capability.authority, "OWNER_AUTHORITATIVE_TOPOLOGY");
    assert!(capability.trusted_rust_host);
    assert!(capability.terminal_surface_capable);
    assert!(!capability.controlling_tty_required);
    assert!(!capability.renderer_direct_owner_access);
    assert!(!capability.generic_invoke_surface);
}

#[test]
fn t169_recursive_projection_retains_immutable_ids_under_duplicate_labels() {
    let first_tab_id = tab_id(2);
    let second_tab_id = tab_id(3);
    let first_pane_id = pane_id(4);
    let second_pane_id = pane_id(5);
    let third_pane_id = pane_id(6);
    let snapshot = ProtocolWorkspaceSnapshotV2 {
        multiplexer_workspace_id: workspace_id(1),
        alias: "duplicate".to_owned(),
        topology_generation: TopologyGeneration::new(7).unwrap(),
        focused_tab_id: first_tab_id,
        tabs: vec![
            ProtocolTabSnapshotV2 {
                tab_id: first_tab_id,
                alias: "duplicate".to_owned(),
                root: ProtocolLayoutNodeV2::Split {
                    axis: ProtocolSplitAxis::Horizontal,
                    ratio_basis_points: 4200,
                    first: Box::new(ProtocolLayoutNodeV2::Pane {
                        pane_id: first_pane_id,
                    }),
                    second: Box::new(ProtocolLayoutNodeV2::Pane {
                        pane_id: second_pane_id,
                    }),
                },
                focused_pane_id: first_pane_id,
                zoomed_pane_id: None,
            },
            ProtocolTabSnapshotV2 {
                tab_id: second_tab_id,
                alias: "duplicate".to_owned(),
                root: ProtocolLayoutNodeV2::Pane {
                    pane_id: third_pane_id,
                },
                focused_pane_id: third_pane_id,
                zoomed_pane_id: Some(third_pane_id),
            },
        ],
    };

    let projected = workspace_projection(&snapshot);
    assert_eq!(
        projected.multiplexer_workspace_id,
        workspace_id(1).to_string()
    );
    assert_eq!(
        projected.tabs[0].display_label,
        projected.tabs[1].display_label
    );
    assert_ne!(projected.tabs[0].tab_id, projected.tabs[1].tab_id);
    assert_eq!(projected.focused_tab_id, first_tab_id.to_string());
    assert_eq!(
        projected.tabs[0].focused_pane_id,
        first_pane_id.to_string()
    );
    let third_pane_label = third_pane_id.to_string();
    assert_eq!(
        projected.tabs[1].zoomed_pane_id.as_deref(),
        Some(third_pane_label.as_str())
    );

    let DesktopTopologyLayoutNode::Split {
        axis,
        ratio_basis_points,
        first,
        second,
    } = &projected.tabs[0].root
    else {
        panic!("recursive split must remain a split");
    };
    assert_eq!(axis, "HORIZONTAL");
    assert_eq!(*ratio_basis_points, 4200);
    assert!(matches!(
        first.as_ref(),
        DesktopTopologyLayoutNode::Pane { pane_id } if pane_id == &first_pane_id.to_string()
    ));
    assert!(matches!(
        second.as_ref(),
        DesktopTopologyLayoutNode::Pane { pane_id } if pane_id == &second_pane_id.to_string()
    ));
}

#[test]
fn t169_owner_generation_and_target_ids_fail_closed_on_malformed_input() {
    assert!(parse_owner_generation(Some("not-a-generation")).is_err());
    assert!(MultiplexerWorkspaceId::parse("not-a-workspace").is_err());
    assert!(TabId::parse("not-a-tab").is_err());
    assert!(PaneId::parse("not-a-pane").is_err());
}
