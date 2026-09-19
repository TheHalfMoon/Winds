use crate::persistent_runtime::domain::{
    ClientAuthority, ClientConnectionId, ContinuityClass, ControllerLeaseIdentity,
    EndpointAvailability, EventSequence, LifecycleProofClass, LocalControlErrorKind,
    OwnerGenerationId, OwnershipState, ProcessLiveness, RuntimeAlias, RuntimeLifecycleEvent,
    RuntimeLifecycleEventKind, RuntimeNamespaceId, RuntimeTruth,
};

fn namespace(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).expect("valid namespace entropy")
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).expect("valid generation entropy")
}

#[test]
fn t146_exact_persistent_ids_round_trip_as_lowercase_fixed_width_hex() {
    let id = RuntimeNamespaceId::from_entropy_bytes([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc,
        0xfe,
    ])
    .unwrap();

    let encoded = id.as_hex();
    assert_eq!(encoded, "0123456789abcdef1032547698badcfe");
    assert_eq!(encoded.len(), 32);
    assert_eq!(RuntimeNamespaceId::parse(&encoded).unwrap(), id);

    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, r#""0123456789abcdef1032547698badcfe""#);
    assert_eq!(
        serde_json::from_str::<RuntimeNamespaceId>(&json).unwrap(),
        id
    );

    let owner = generation(0x0a);
    let owner_json = serde_json::to_string(&owner).unwrap();
    assert_eq!(owner_json, r#""0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a""#);
    assert_eq!(
        serde_json::from_str::<OwnerGenerationId>(&owner_json).unwrap(),
        owner
    );
}

#[test]
fn t146_persistent_ids_reject_noncanonical_or_unproven_entropy() {
    assert!(RuntimeNamespaceId::from_entropy_bytes([0; 16]).is_err());
    assert!(OwnerGenerationId::from_entropy_bytes([0; 16]).is_err());

    for value in [
        "",
        "00",
        "00000000000000000000000000000000",
        "0123456789ABCDEF1032547698BADCFE",
        "g123456789abcdef1032547698badcfe",
        "0123456789abcdef1032547698badcf",
        "0123456789abcdef1032547698badcfee",
    ] {
        assert!(RuntimeNamespaceId::parse(value).is_err(), "{value}");
    }
}

#[test]
fn t146_duplicate_aliases_never_create_runtime_identity_equivalence() {
    let alias_a = RuntimeAlias::new("  worker  ").unwrap();
    let alias_b = RuntimeAlias::new("worker").unwrap();
    assert_eq!(alias_a, alias_b);
    assert_eq!(alias_a.as_str(), "worker");

    let first = namespace(1);
    let second = namespace(2);
    assert_ne!(first, second);
    assert_eq!(alias_a, alias_b);

    for invalid in ["", "   ", "\0alias"] {
        assert!(RuntimeAlias::new(invalid).is_err());
    }
    assert!(RuntimeAlias::new(&"x".repeat(257)).is_err());
    assert!(serde_json::from_str::<RuntimeAlias>(r#""   ""#).is_err());
    assert!(serde_json::from_str::<RuntimeAlias>(&format!(r#""{}""#, "x".repeat(257))).is_err());
}

#[test]
fn t146_ownership_liveness_endpoint_and_continuity_remain_separate_truth_dimensions() {
    let truth = RuntimeTruth {
        ownership: OwnershipState::OwnershipLost,
        process_liveness: ProcessLiveness::Unknown,
        endpoint_availability: EndpointAvailability::Unavailable,
        continuity: ContinuityClass::Unknown,
    };

    let json = serde_json::to_value(&truth).unwrap();
    assert_eq!(json["ownership"], "OWNERSHIP_LOST");
    assert_eq!(json["process_liveness"], "UNKNOWN");
    assert_eq!(json["endpoint_availability"], "UNAVAILABLE");
    assert_eq!(json["continuity"], "UNKNOWN");

    let still_running_without_ownership = RuntimeTruth {
        ownership: OwnershipState::OwnershipLost,
        process_liveness: ProcessLiveness::Running,
        endpoint_availability: EndpointAvailability::Unknown,
        continuity: ContinuityClass::Unavailable,
    };
    assert_ne!(truth, still_running_without_ownership);

    assert_eq!(
        serde_json::to_string(&ContinuityClass::RetainedLiveProcess).unwrap(),
        r#""RETAINED_LIVE_PROCESS""#
    );
    assert_eq!(
        serde_json::to_string(&ContinuityClass::ProviderNativeResume).unwrap(),
        r#""PROVIDER_NATIVE_RESUME""#
    );
    assert_eq!(
        serde_json::to_string(&ContinuityClass::WindsReconstruction).unwrap(),
        r#""WINDS_RECONSTRUCTION""#
    );
    assert_eq!(
        serde_json::to_string(&ContinuityClass::FreshProcess).unwrap(),
        r#""FRESH_PROCESS""#
    );
}

#[test]
fn t146_observer_controller_and_error_vocabulary_are_explicit() {
    assert_eq!(
        serde_json::to_string(&ClientAuthority::Observer).unwrap(),
        r#""OBSERVER""#
    );
    assert_eq!(
        serde_json::to_string(&ClientAuthority::Controller).unwrap(),
        r#""CONTROLLER""#
    );

    let required_errors = [
        LocalControlErrorKind::ProtocolMismatch,
        LocalControlErrorKind::PrincipalDenied,
        LocalControlErrorKind::StaleOwnerGeneration,
        LocalControlErrorKind::UnknownRuntime,
        LocalControlErrorKind::OwnershipLost,
        LocalControlErrorKind::ControllerConflict,
        LocalControlErrorKind::MalformedFrame,
        LocalControlErrorKind::OversizedFrame,
        LocalControlErrorKind::DuplicateOrOutOfOrderRequest,
        LocalControlErrorKind::SlowClientBackpressure,
        LocalControlErrorKind::UnsupportedOperation,
        LocalControlErrorKind::UnsupportedPlatform,
        LocalControlErrorKind::OutcomeUnknown,
    ];
    let serialized = required_errors
        .iter()
        .map(|error| serde_json::to_string(error).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(serialized.len(), 13);
    assert!(serialized.contains(&r#""OUTCOME_UNKNOWN""#.to_owned()));
    assert!(serialized.contains(&r#""DUPLICATE_OR_OUT_OF_ORDER_REQUEST""#.to_owned()));
}

#[test]
fn t146_controller_lease_identity_binds_owner_runtime_and_client_without_lease_mechanics() {
    let identity = ControllerLeaseIdentity {
        owner_generation_id: generation(3),
        runtime_namespace_id: namespace(4),
        controller_client_id: ClientConnectionId::new("client-7").unwrap(),
    };
    let json = serde_json::to_string(&identity).unwrap();
    let decoded: ControllerLeaseIdentity = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded, identity);
    assert_eq!(decoded.controller_client_id.as_str(), "client-7");
    assert!(ClientConnectionId::new("").is_err());
    assert!(ClientConnectionId::new("\0client").is_err());
    assert!(ClientConnectionId::new(&"x".repeat(129)).is_err());
    assert!(serde_json::from_str::<ClientConnectionId>(r#""""#).is_err());
    assert!(
        serde_json::from_str::<ClientConnectionId>(&format!(r#""{}""#, "x".repeat(129))).is_err()
    );
}

#[test]
fn t146_event_sequence_is_nonzero_and_round_trips_as_a_number() {
    assert!(EventSequence::new(0).is_err());
    let sequence = EventSequence::new(9).unwrap();
    assert_eq!(sequence.get(), 9);
    assert_eq!(serde_json::to_string(&sequence).unwrap(), "9");
    assert_eq!(
        serde_json::from_str::<EventSequence>("9").unwrap(),
        sequence
    );
    assert!(serde_json::from_str::<EventSequence>("0").is_err());
}

#[test]
fn t146_lifecycle_event_serialization_is_deterministic_and_authority_bounded() {
    let event = RuntimeLifecycleEvent {
        runtime_namespace_id: namespace(5),
        owner_generation_id: generation(6),
        sequence: EventSequence::new(7).unwrap(),
        kind: RuntimeLifecycleEventKind::OwnershipLost,
        proof_class: LifecycleProofClass::WindsObserved,
        controller_client_id: Some(ClientConnectionId::new("desktop-rust-host").unwrap()),
        observed_unix_ms: Some(1_789_778_000_000),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert_eq!(
        json,
        r#"{"runtime_namespace_id":"05050505050505050505050505050505","owner_generation_id":"06060606060606060606060606060606","sequence":7,"kind":"OWNERSHIP_LOST","proof_class":"WINDS_OBSERVED","controller_client_id":"desktop-rust-host","observed_unix_ms":1789778000000}"#
    );
    let decoded: RuntimeLifecycleEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, event);

    let value = serde_json::to_value(&event).unwrap();
    let object = value.as_object().unwrap();
    for forbidden in [
        "credential",
        "credentials",
        "secret",
        "environment",
        "process_environment",
        "agent_prose",
        "verification",
        "accepted",
        "approval",
        "candidate",
        "git",
    ] {
        assert!(!object.contains_key(forbidden), "{forbidden}");
    }
}

#[test]
fn t146_lifecycle_proof_class_preserves_agent_winds_human_distinctions() {
    assert_eq!(
        serde_json::to_string(&LifecycleProofClass::AgentReported).unwrap(),
        r#""AGENT_REPORTED""#
    );
    assert_eq!(
        serde_json::to_string(&LifecycleProofClass::WindsObserved).unwrap(),
        r#""WINDS_OBSERVED""#
    );
    assert_eq!(
        serde_json::to_string(&LifecycleProofClass::HumanDecided).unwrap(),
        r#""HUMAN_DECIDED""#
    );
}

#[test]
fn t146_event_kind_is_typed_and_has_no_arbitrary_text_authority() {
    let kinds = [
        RuntimeLifecycleEventKind::NamespaceCreated,
        RuntimeLifecycleEventKind::OwnershipEstablished,
        RuntimeLifecycleEventKind::OwnershipLost,
        RuntimeLifecycleEventKind::ProcessStateObserved,
        RuntimeLifecycleEventKind::ContinuityClassified,
        RuntimeLifecycleEventKind::ControllerChanged,
        RuntimeLifecycleEventKind::RuntimeStopped,
    ];

    assert_eq!(kinds.len(), 7);
    assert_eq!(
        serde_json::to_string(&RuntimeLifecycleEventKind::ControllerChanged).unwrap(),
        r#""CONTROLLER_CHANGED""#
    );
}
