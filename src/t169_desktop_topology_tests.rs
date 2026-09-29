use super::*;
use crate::persistent_runtime::protocol::{ProtocolTabSnapshotV2, ProtocolWorkspaceSnapshotV2};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HOME: AtomicU64 = AtomicU64::new(0);

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
    assert!(!capability.renderer_supplied_owner_generation);
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
    assert_eq!(projected.tabs[0].focused_pane_id, first_pane_id.to_string());
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
fn t169_target_ids_fail_closed_on_malformed_input() {
    assert!(MultiplexerWorkspaceId::parse("not-a-workspace").is_err());
    assert!(TabId::parse("not-a-tab").is_err());
    assert!(PaneId::parse("not-a-pane").is_err());
}

#[test]
fn t169_renderer_cannot_supply_an_owner_generation_or_any_untyped_request_field() {
    assert!(
        serde_json::from_value::<DesktopTopologySnapshotRequest>(serde_json::json!({
            "expectedOwnerGenerationId": "00112233445566778899aabbccddeeff",
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<DesktopTopologyBindRequest>(serde_json::json!({
            "expectedOwnerGenerationId": "00112233445566778899aabbccddeeff",
            "expectedTopologyGeneration": 1,
            "multiplexerWorkspaceId": workspace_id(1).to_string(),
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<DesktopTopologyBindRequest>(serde_json::json!({
            "expectedTopologyGeneration": 1,
            "multiplexerWorkspaceId": workspace_id(1).to_string(),
            "paneOwnerGenerationId": "00112233445566778899aabbccddeeff",
        }))
        .is_err()
    );

    let snapshot: DesktopTopologySnapshotRequest =
        serde_json::from_value(serde_json::json!({})).expect("closed empty snapshot request");
    assert_eq!(snapshot, DesktopTopologySnapshotRequest {});
}

fn test_home(name: &str) -> std::path::PathBuf {
    let sequence = NEXT_HOME.fetch_add(1, Ordering::Relaxed);
    let home = std::env::temp_dir().join(format!(
        "winds-t169-{name}-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir_all(&home).unwrap();
    home
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).expect("valid owner generation")
}

#[test]
fn t169_trusted_owner_generation_is_empty_absent_ambiguous_or_unusable_and_never_renderer_supplied()
{
    let empty = test_home("empty");
    let store = Store::open(&empty).unwrap();
    assert_eq!(
        store.latest_persistent_runtime_owner_generation().unwrap(),
        None
    );

    store
        .record_persistent_runtime_owner_generation(generation(1), 1_000)
        .unwrap();
    store
        .record_persistent_runtime_owner_generation(generation(2), 2_000)
        .unwrap();
    assert_eq!(
        store.latest_persistent_runtime_owner_generation().unwrap(),
        Some(generation(2))
    );

    store
        .record_persistent_runtime_owner_generation(generation(3), 2_000)
        .unwrap();
    assert!(
        store
            .latest_persistent_runtime_owner_generation()
            .is_err_and(|error| error.to_string().contains("ambiguous")),
        "an ambiguous recorded generation start time must fail closed"
    );

    store
        .record_persistent_runtime_owner_generation(generation(4), 3_000)
        .unwrap();
    assert_eq!(
        store.latest_persistent_runtime_owner_generation().unwrap(),
        Some(generation(4))
    );

    let _ = std::fs::remove_dir_all(&empty);
}
