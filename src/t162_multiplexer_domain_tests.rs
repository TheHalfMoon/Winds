use crate::multiplexer::domain::{
    AgentObservationId, AgentObservationTopologyBinding, ClientSurfaceCapability, LayoutTemplateId,
    MultiplexerAuthority, MultiplexerErrorKind, MultiplexerWorkspaceId, PaneId, TabId,
    TopologyGeneration, validate_replacement_pane_id,
};
use crate::persistent_runtime::domain::{ClientAuthority, RuntimeNamespaceId};
use std::any::TypeId;

fn workspace(byte: u8) -> MultiplexerWorkspaceId {
    MultiplexerWorkspaceId::from_entropy_bytes([byte; 16]).expect("valid workspace entropy")
}

fn tab(byte: u8) -> TabId {
    TabId::from_entropy_bytes([byte; 16]).expect("valid tab entropy")
}

fn pane(byte: u8) -> PaneId {
    PaneId::from_entropy_bytes([byte; 16]).expect("valid pane entropy")
}

fn observation(byte: u8) -> AgentObservationId {
    AgentObservationId::from_entropy_bytes([byte; 16]).expect("valid observation entropy")
}

#[test]
fn t162_exact_multiplexer_ids_round_trip_as_lowercase_fixed_width_hex() {
    let id = MultiplexerWorkspaceId::from_entropy_bytes([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc,
        0xfe,
    ])
    .unwrap();

    let encoded = id.as_hex();
    assert_eq!(encoded, "0123456789abcdef1032547698badcfe");
    assert_eq!(encoded.len(), 32);
    assert_eq!(MultiplexerWorkspaceId::parse(&encoded).unwrap(), id);
    assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{encoded}\""));

    let layout = LayoutTemplateId::from_entropy_bytes([0x0a; 16]).unwrap();
    let json = serde_json::to_string(&layout).unwrap();
    assert_eq!(
        serde_json::from_str::<LayoutTemplateId>(&json).unwrap(),
        layout
    );
}

#[test]
fn t162_multiplexer_ids_reject_unproven_or_noncanonical_values() {
    assert!(MultiplexerWorkspaceId::from_entropy_bytes([0; 16]).is_err());
    assert!(TabId::from_entropy_bytes([0; 16]).is_err());
    assert!(PaneId::from_entropy_bytes([0; 16]).is_err());
    assert!(LayoutTemplateId::from_entropy_bytes([0; 16]).is_err());
    assert!(AgentObservationId::from_entropy_bytes([0; 16]).is_err());

    for value in [
        "",
        "00",
        "00000000000000000000000000000000",
        "0123456789ABCDEF1032547698BADCFE",
        "g123456789abcdef1032547698badcfe",
        "0123456789abcdef1032547698badcf",
        "0123456789abcdef1032547698badcfee",
        "workspace-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ] {
        assert!(MultiplexerWorkspaceId::parse(value).is_err(), "{value}");
    }
}

#[test]
fn t162_identity_domains_remain_structurally_distinct_from_runtime_and_generation() {
    assert_ne!(
        TypeId::of::<MultiplexerWorkspaceId>(),
        TypeId::of::<RuntimeNamespaceId>()
    );
    assert_ne!(
        TypeId::of::<MultiplexerWorkspaceId>(),
        TypeId::of::<TopologyGeneration>()
    );
    assert_ne!(TypeId::of::<PaneId>(), TypeId::of::<TabId>());
    assert_ne!(TypeId::of::<PaneId>(), TypeId::of::<AgentObservationId>());

    let first_label = "duplicate";
    let second_label = "duplicate";
    let first = workspace(1);
    let second = workspace(2);
    assert_eq!(first_label, second_label);
    assert_ne!(first, second);
}

#[test]
fn t162_replacement_panes_must_receive_a_fresh_identity() {
    let closed = pane(3);
    assert_eq!(
        validate_replacement_pane_id(closed, closed),
        Err(MultiplexerErrorKind::IdentityReuse)
    );
    assert_eq!(validate_replacement_pane_id(closed, pane(4)), Ok(()));
}

#[test]
fn t162_topology_generation_is_nonzero_monotonic_and_not_identity() {
    assert_eq!(TopologyGeneration::initial().get(), 1);
    assert!(TopologyGeneration::new(0).is_err());

    let current = TopologyGeneration::new(41).unwrap();
    assert_eq!(current.checked_next().unwrap().get(), 42);
    assert_eq!(serde_json::to_string(&current).unwrap(), "41");
    assert_eq!(
        serde_json::from_str::<TopologyGeneration>("41").unwrap(),
        current
    );
    assert!(serde_json::from_str::<TopologyGeneration>("0").is_err());

    let max = TopologyGeneration::new(u64::MAX).unwrap();
    assert_eq!(
        max.checked_next(),
        Err(MultiplexerErrorKind::TopologyGenerationExhausted)
    );
}

#[test]
fn t162_multiplexer_write_and_runtime_controller_authority_are_nontransitive() {
    assert_eq!(
        serde_json::to_string(&MultiplexerAuthority::Observer).unwrap(),
        r#""OBSERVER""#
    );
    assert_eq!(
        serde_json::to_string(&MultiplexerAuthority::MultiplexerWrite).unwrap(),
        r#""MULTIPLEXER_WRITE""#
    );
    assert_eq!(
        serde_json::to_string(&ClientAuthority::Controller).unwrap(),
        r#""CONTROLLER""#
    );
    assert_ne!(
        serde_json::to_string(&MultiplexerAuthority::MultiplexerWrite).unwrap(),
        serde_json::to_string(&ClientAuthority::Controller).unwrap()
    );
}

#[test]
fn t162_client_capabilities_distinguish_terminal_and_desktop_surfaces() {
    let capabilities = [
        ClientSurfaceCapability::NonInteractiveObserver,
        ClientSurfaceCapability::ControllingTerminal,
        ClientSurfaceCapability::TrustedDesktopTerminalSurface,
    ];
    let serialized = capabilities
        .iter()
        .map(|capability| serde_json::to_string(capability).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(serialized.len(), 3);
    assert!(serialized.contains(&r#""CONTROLLING_TERMINAL""#.to_owned()));
    assert!(serialized.contains(&r#""TRUSTED_DESKTOP_TERMINAL_SURFACE""#.to_owned()));
}

#[test]
fn t162_closed_error_vocabulary_covers_required_fail_closed_outcomes() {
    let required = [
        MultiplexerErrorKind::StaleTopologyGeneration,
        MultiplexerErrorKind::UnknownWorkspace,
        MultiplexerErrorKind::UnknownTab,
        MultiplexerErrorKind::UnknownPane,
        MultiplexerErrorKind::ClosedTarget,
        MultiplexerErrorKind::IdentityReuse,
        MultiplexerErrorKind::DuplicateOrReplayedMutation,
        MultiplexerErrorKind::SnapshotLimitExceeded,
        MultiplexerErrorKind::CapabilityUnavailable,
        MultiplexerErrorKind::MultiplexerWriteRequired,
        MultiplexerErrorKind::RuntimeControllerRequired,
        MultiplexerErrorKind::UnsupportedOperation,
        MultiplexerErrorKind::UnsupportedPlatform,
        MultiplexerErrorKind::OwnershipLost,
        MultiplexerErrorKind::OutcomeUnknown,
        MultiplexerErrorKind::TopologyGenerationExhausted,
    ];
    let serialized = required
        .iter()
        .map(|kind| serde_json::to_string(kind).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(serialized.len(), 16);
    assert!(serialized.contains(&r#""STALE_TOPOLOGY_GENERATION""#.to_owned()));
    assert!(serialized.contains(&r#""SNAPSHOT_LIMIT_EXCEEDED""#.to_owned()));
    assert!(serialized.contains(&r#""CAPABILITY_UNAVAILABLE""#.to_owned()));
    assert!(serialized.contains(&r#""OUTCOME_UNKNOWN""#.to_owned()));
}

#[test]
fn t162_agent_observation_binding_names_the_multiplexer_workspace_domain_explicitly() {
    let binding = AgentObservationTopologyBinding {
        observation_id: observation(5),
        multiplexer_workspace_id: workspace(6),
        tab_id: tab(7),
        pane_id: pane(8),
        runtime_namespace_id: Some(
            RuntimeNamespaceId::from_entropy_bytes([9; 16]).expect("valid runtime entropy"),
        ),
    };

    let value = serde_json::to_value(&binding).unwrap();
    let object = value.as_object().unwrap();

    assert!(object.contains_key("observation_id"));
    assert!(object.contains_key("multiplexer_workspace_id"));
    assert!(object.contains_key("tab_id"));
    assert!(object.contains_key("pane_id"));
    assert!(object.contains_key("runtime_namespace_id"));
    assert!(!object.contains_key("workspace_id"));
    assert!(!object.contains_key("git_workspace_id"));

    let round_trip: AgentObservationTopologyBinding =
        serde_json::from_value(value.clone()).expect("binding round trip");
    assert_eq!(round_trip, binding);

    let mut ambiguous = value;
    ambiguous
        .as_object_mut()
        .unwrap()
        .insert("workspace_id".to_owned(), serde_json::json!("workspace-ambiguous"));
    assert!(serde_json::from_value::<AgentObservationTopologyBinding>(ambiguous).is_err());
}
