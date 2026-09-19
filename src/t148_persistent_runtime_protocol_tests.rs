use super::{
    MAX_INBOUND_CONTROL_FRAME_BYTES, MAX_OUTPUT_EVENT_CHUNK_BYTES, MessageAuthorityClass,
    MessageDirection, MessageKind, MutationOutcomeTracker, ProtocolMessage, ProtocolPayload,
    RequestSequenceGuard, decode_frame, encode_frame, read_frame, validate_event_binding,
    validate_response_binding, write_frame,
};
use crate::persistent_runtime::domain::{
    ClientAuthority, ClientConnectionId, ContinuityClass, EndpointAvailability, EventSequence,
    LifecycleProofClass, LocalControlErrorKind, OwnerGenerationId, OwnershipState, ProcessLiveness,
    RuntimeLifecycleEvent, RuntimeLifecycleEventKind, RuntimeNamespaceId, RuntimeTruth,
};
use std::io::Cursor;

fn namespace(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).expect("valid namespace")
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).expect("valid generation")
}

fn connection(value: &str) -> ClientConnectionId {
    ClientConnectionId::new(value).expect("valid connection id")
}

fn sequence(value: u64) -> EventSequence {
    EventSequence::new(value).expect("nonzero sequence")
}

fn client_message(
    value: u64,
    runtime_namespace_id: Option<RuntimeNamespaceId>,
    payload: ProtocolPayload,
) -> ProtocolMessage {
    ProtocolMessage::new(
        connection("client-a"),
        sequence(value),
        runtime_namespace_id,
        generation(2),
        None,
        payload,
    )
    .expect("valid client message")
}

fn framed_json(json: &str) -> Vec<u8> {
    let bytes = json.as_bytes();
    let mut frame = Vec::with_capacity(4 + bytes.len());
    frame.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    frame.extend_from_slice(bytes);
    frame
}

#[test]
fn t148_hello_golden_frame_is_u32_le_followed_by_deterministic_utf8_json() {
    let message = ProtocolMessage::hello(sequence(1), 1, 1, None).unwrap();
    let encoded = encode_frame(&message).unwrap();
    let payload = &encoded[4..];

    assert_eq!(
        u32::from_le_bytes(encoded[..4].try_into().unwrap()) as usize,
        payload.len()
    );
    assert_eq!(
        std::str::from_utf8(payload).unwrap(),
        r#"{"protocol_version":1,"message_kind":"HELLO","sequence":1,"body":{"maximum_protocol_version":1,"minimum_protocol_version":1}}"#
    );
    assert_eq!(decode_frame(&encoded).unwrap(), message);

    let mut sink = Vec::new();
    write_frame(&mut sink, &message).unwrap();
    assert_eq!(sink, encoded);
    assert_eq!(read_frame(&mut Cursor::new(sink)).unwrap(), message);
}

#[test]
fn t148_handshake_assigns_connection_identity_only_in_hello_ack_and_checks_expected_generation() {
    let fresh = ProtocolMessage::hello(sequence(1), 1, 1, None).unwrap();
    assert!(fresh.connection_id.is_none());
    assert!(fresh.owner_generation_id.is_none());

    let ack = ProtocolMessage::new(
        connection("owner-assigned"),
        sequence(2),
        None,
        generation(2),
        Some(sequence(1)),
        ProtocolPayload::HelloAck,
    )
    .unwrap();
    assert!(validate_response_binding(&fresh, &ack).is_ok());

    let reconnect = ProtocolMessage::hello(sequence(3), 1, 1, Some(generation(2))).unwrap();
    assert!(
        validate_response_binding(
            &reconnect,
            &ProtocolMessage::new(
                connection("owner-assigned-2"),
                sequence(4),
                None,
                generation(2),
                Some(sequence(3)),
                ProtocolPayload::HelloAck,
            )
            .unwrap()
        )
        .is_ok()
    );

    let replaced_owner = ProtocolMessage::new(
        connection("owner-assigned-3"),
        sequence(4),
        None,
        generation(9),
        Some(sequence(3)),
        ProtocolPayload::HelloAck,
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&reconnect, &replaced_owner).unwrap_err(),
        LocalControlErrorKind::StaleOwnerGeneration
    );

    assert_eq!(
        ProtocolMessage::hello(sequence(5), 2, 3, None).unwrap_err(),
        LocalControlErrorKind::ProtocolMismatch
    );
}

#[test]
fn t148_message_ceiling_is_closed_and_authority_classes_are_exact() {
    let kinds = [
        MessageKind::Hello,
        MessageKind::HelloAck,
        MessageKind::Ping,
        MessageKind::Pong,
        MessageKind::ListRuntimes,
        MessageKind::RuntimeSnapshot,
        MessageKind::AttachObserver,
        MessageKind::Detach,
        MessageKind::RuntimeEvent,
        MessageKind::OutputEvent,
        MessageKind::HistoryGap,
        MessageKind::OwnerStatus,
        MessageKind::Error,
        MessageKind::ControlState,
        MessageKind::RequestControl,
        MessageKind::ReleaseControl,
        MessageKind::Input,
        MessageKind::Resize,
        MessageKind::Interrupt,
        MessageKind::Stop,
    ];
    assert_eq!(kinds.len(), 20);

    for kind in [
        MessageKind::Hello,
        MessageKind::Ping,
        MessageKind::Pong,
        MessageKind::ListRuntimes,
        MessageKind::RuntimeSnapshot,
        MessageKind::AttachObserver,
        MessageKind::Detach,
        MessageKind::RuntimeEvent,
        MessageKind::OutputEvent,
        MessageKind::HistoryGap,
        MessageKind::OwnerStatus,
    ] {
        assert_eq!(
            kind.authority_class(),
            MessageAuthorityClass::ConnectionObserverSafe
        );
    }
    for kind in [
        MessageKind::HelloAck,
        MessageKind::Error,
        MessageKind::ControlState,
    ] {
        assert_eq!(
            kind.authority_class(),
            MessageAuthorityClass::OwnerToClientStateOnly
        );
    }
    assert_eq!(
        MessageKind::RequestControl.authority_class(),
        MessageAuthorityClass::BoundedAuthorityTransition
    );
    assert_eq!(
        MessageKind::ReleaseControl.authority_class(),
        MessageAuthorityClass::ActiveControllerRevocation
    );
    for kind in [
        MessageKind::Input,
        MessageKind::Resize,
        MessageKind::Interrupt,
        MessageKind::Stop,
    ] {
        assert_eq!(
            kind.authority_class(),
            MessageAuthorityClass::ControllerOnly
        );
    }

    assert!(serde_json::from_str::<MessageKind>(r#""SHUTDOWN_IF_IDLE""#).is_err());
    assert_eq!(
        MessageKind::Input.direction(),
        MessageDirection::ClientToOwner
    );
    assert_eq!(
        MessageKind::RuntimeEvent.direction(),
        MessageDirection::OwnerToClient
    );
}

#[test]
fn t148_runtime_and_generation_bindings_fail_closed() {
    let runtime = namespace(3);
    let event = RuntimeLifecycleEvent {
        runtime_namespace_id: runtime,
        owner_generation_id: generation(2),
        sequence: sequence(10),
        kind: RuntimeLifecycleEventKind::ProcessStateObserved,
        proof_class: LifecycleProofClass::WindsObserved,
        controller_client_id: None,
        observed_unix_ms: Some(1_789_780_000_000),
    };
    let message = ProtocolMessage::new(
        connection("client-a"),
        sequence(10),
        Some(runtime),
        generation(2),
        None,
        ProtocolPayload::RuntimeEvent {
            event: event.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        decode_frame(&encode_frame(&message).unwrap()).unwrap(),
        message
    );

    let wrong_event_sequence = RuntimeLifecycleEvent {
        sequence: sequence(8),
        ..event.clone()
    };
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(10),
            Some(runtime),
            generation(2),
            None,
            ProtocolPayload::RuntimeEvent {
                event: wrong_event_sequence,
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(10),
            Some(namespace(4)),
            generation(2),
            None,
            ProtocolPayload::RuntimeEvent {
                event: event.clone()
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(10),
            Some(runtime),
            generation(5),
            None,
            ProtocolPayload::RuntimeEvent { event },
        )
        .unwrap_err(),
        LocalControlErrorKind::StaleOwnerGeneration
    );

    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(1),
            None,
            generation(2),
            None,
            ProtocolPayload::Input { data: "x".into() },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(1),
            Some(runtime),
            generation(2),
            None,
            ProtocolPayload::Hello {
                minimum_protocol_version: 1,
                maximum_protocol_version: 1,
                expected_owner_generation_id: None,
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

#[test]
fn t148_frame_parser_rejects_oversized_truncated_invalid_utf8_and_trailing_bytes() {
    let oversized = ((MAX_INBOUND_CONTROL_FRAME_BYTES + 1) as u32).to_le_bytes();
    assert_eq!(
        read_frame(&mut Cursor::new(oversized)).unwrap_err(),
        LocalControlErrorKind::OversizedFrame
    );

    assert_eq!(
        read_frame(&mut Cursor::new(0_u32.to_le_bytes())).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let mut truncated = Vec::new();
    truncated.extend_from_slice(&10_u32.to_le_bytes());
    truncated.extend_from_slice(b"{}");
    assert_eq!(
        read_frame(&mut Cursor::new(truncated)).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let invalid_utf8 = [1_u32.to_le_bytes().as_slice(), &[0xff]].concat();
    assert_eq!(
        read_frame(&mut Cursor::new(invalid_utf8)).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let mut valid = encode_frame(&client_message(1, None, ProtocolPayload::Ping)).unwrap();
    valid.push(0);
    assert_eq!(
        decode_frame(&valid).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

#[test]
fn t148_json_schema_rejects_unknown_version_kind_fields_and_malformed_body() {
    let base = r#"{"protocol_version":1,"message_kind":"HELLO","sequence":1,"body":{"minimum_protocol_version":1,"maximum_protocol_version":1}}"#;
    assert!(decode_frame(&framed_json(base)).is_ok());

    let wrong_version = base.replace("\"protocol_version\":1", "\"protocol_version\":2");
    assert_eq!(
        decode_frame(&framed_json(&wrong_version)).unwrap_err(),
        LocalControlErrorKind::ProtocolMismatch
    );
    let unknown_kind = base.replace("\"HELLO\"", "\"GENERIC_DISPATCH\"");
    assert_eq!(
        decode_frame(&framed_json(&unknown_kind)).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    let unknown_outer = base.replace("\"sequence\":1,", "\"sequence\":1,\"extra\":1,");
    assert_eq!(
        decode_frame(&framed_json(&unknown_outer)).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    let unknown_body = base.replace(
        "\"body\":{\"minimum_protocol_version\":1,\"maximum_protocol_version\":1}",
        "\"body\":{\"minimum_protocol_version\":1,\"maximum_protocol_version\":1,\"extra\":1}",
    );
    assert_eq!(
        decode_frame(&framed_json(&unknown_body)).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    let missing_body = base.replace(
        ",\"body\":{\"minimum_protocol_version\":1,\"maximum_protocol_version\":1}",
        "",
    );
    assert_eq!(
        decode_frame(&framed_json(&missing_body)).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        decode_frame(&framed_json("{not-json}")).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

#[test]
fn t148_output_chunk_limit_is_measured_before_framing() {
    let runtime = namespace(3);
    let exact = "é".repeat(MAX_OUTPUT_EVENT_CHUNK_BYTES / 2);
    assert_eq!(exact.len(), MAX_OUTPUT_EVENT_CHUNK_BYTES);
    let accepted = ProtocolMessage::new(
        connection("client-a"),
        sequence(4),
        Some(runtime),
        generation(2),
        None,
        ProtocolPayload::OutputEvent { chunk: exact },
    )
    .unwrap();
    assert!(encode_frame(&accepted).is_ok());

    let too_large = "é".repeat(MAX_OUTPUT_EVENT_CHUNK_BYTES / 2 + 1);
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(4),
            Some(runtime),
            generation(2),
            None,
            ProtocolPayload::OutputEvent { chunk: too_large },
        )
        .unwrap_err(),
        LocalControlErrorKind::OversizedFrame
    );
}

#[test]
fn t148_request_sequence_guard_rejects_duplicate_and_lower_requests_before_execution() {
    let runtime = namespace(3);
    let mut guard = RequestSequenceGuard::new(connection("client-a"), generation(2));
    let first = client_message(
        7,
        Some(runtime),
        ProtocolPayload::Input {
            data: "first".into(),
        },
    );
    let duplicate = client_message(
        7,
        Some(runtime),
        ProtocolPayload::Input {
            data: "duplicate".into(),
        },
    );
    let lower = client_message(
        6,
        Some(runtime),
        ProtocolPayload::Resize {
            columns: 100,
            rows: 40,
        },
    );

    let mut executions = 0;
    if guard.accept(&first).is_ok() {
        executions += 1;
    }
    assert_eq!(
        guard.accept(&duplicate).unwrap_err(),
        LocalControlErrorKind::DuplicateOrOutOfOrderRequest
    );
    assert_eq!(
        guard.accept(&lower).unwrap_err(),
        LocalControlErrorKind::DuplicateOrOutOfOrderRequest
    );
    assert_eq!(executions, 1);

    let wrong_generation = ProtocolMessage::new(
        connection("client-a"),
        sequence(8),
        Some(runtime),
        generation(9),
        None,
        ProtocolPayload::Stop,
    )
    .unwrap();
    assert_eq!(
        guard.accept(&wrong_generation).unwrap_err(),
        LocalControlErrorKind::StaleOwnerGeneration
    );
}

#[test]
fn t148_response_correlation_cannot_be_reassigned_across_identity_boundaries() {
    let runtime = namespace(3);
    let request = client_message(11, Some(runtime), ProtocolPayload::RequestControl);
    let response = ProtocolMessage::new(
        connection("client-a"),
        sequence(12),
        Some(runtime),
        generation(2),
        Some(sequence(11)),
        ProtocolPayload::ControlState {
            authority: ClientAuthority::Controller,
            controller_client_id: Some(connection("client-a")),
        },
    )
    .unwrap();
    assert!(validate_response_binding(&request, &response).is_ok());

    let wrong_runtime = ProtocolMessage::new(
        connection("client-a"),
        sequence(12),
        Some(namespace(4)),
        generation(2),
        Some(sequence(11)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::ControllerConflict,
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&request, &wrong_runtime).unwrap_err(),
        LocalControlErrorKind::UnknownRuntime
    );

    let wrong_connection = ProtocolMessage::new(
        connection("client-b"),
        sequence(12),
        Some(runtime),
        generation(2),
        Some(sequence(11)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::ControllerConflict,
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&request, &wrong_connection).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let wrong_generation = ProtocolMessage::new(
        connection("client-a"),
        sequence(12),
        Some(runtime),
        generation(9),
        Some(sequence(11)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::ControllerConflict,
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&request, &wrong_generation).unwrap_err(),
        LocalControlErrorKind::StaleOwnerGeneration
    );

    let wrong_sequence = ProtocolMessage::new(
        connection("client-a"),
        sequence(12),
        Some(runtime),
        generation(2),
        Some(sequence(10)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::ControllerConflict,
        },
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&request, &wrong_sequence).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let ping = client_message(19, None, ProtocolPayload::Ping);
    let wrong_kind = ProtocolMessage::new(
        connection("client-a"),
        sequence(20),
        None,
        generation(2),
        Some(sequence(19)),
        ProtocolPayload::HelloAck,
    )
    .unwrap();
    assert_eq!(
        validate_response_binding(&ping, &wrong_kind).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let list = client_message(20, None, ProtocolPayload::ListRuntimes);
    let snapshot = ProtocolMessage::new(
        connection("client-a"),
        sequence(21),
        Some(runtime),
        generation(2),
        Some(sequence(20)),
        ProtocolPayload::RuntimeSnapshot {
            truth: RuntimeTruth {
                ownership: OwnershipState::OwnershipLost,
                process_liveness: ProcessLiveness::Unknown,
                endpoint_availability: EndpointAvailability::Unknown,
                continuity: ContinuityClass::Unknown,
            },
        },
    )
    .unwrap();
    assert!(validate_response_binding(&list, &snapshot).is_ok());
}

#[test]
fn t148_async_event_binding_cannot_be_reassigned_across_connection_generation_or_runtime() {
    let runtime = namespace(3);
    let event = ProtocolMessage::new(
        connection("client-a"),
        sequence(25),
        Some(runtime),
        generation(2),
        None,
        ProtocolPayload::OutputEvent {
            chunk: "bounded output".into(),
        },
    )
    .unwrap();
    assert!(
        validate_event_binding(
            &connection("client-a"),
            generation(2),
            Some(runtime),
            &event
        )
        .is_ok()
    );
    assert_eq!(
        validate_event_binding(
            &connection("client-b"),
            generation(2),
            Some(runtime),
            &event
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        validate_event_binding(
            &connection("client-a"),
            generation(9),
            Some(runtime),
            &event
        )
        .unwrap_err(),
        LocalControlErrorKind::StaleOwnerGeneration
    );
    assert_eq!(
        validate_event_binding(
            &connection("client-a"),
            generation(2),
            Some(namespace(4)),
            &event,
        )
        .unwrap_err(),
        LocalControlErrorKind::UnknownRuntime
    );
}

#[test]
fn t148_lost_mutation_response_becomes_outcome_unknown_until_state_reconciliation() {
    let runtime = namespace(3);
    let first = client_message(30, Some(runtime), ProtocolPayload::RequestControl);
    let second = client_message(
        31,
        Some(runtime),
        ProtocolPayload::Input {
            data: "after-loss".into(),
        },
    );
    let mut tracker = MutationOutcomeTracker::new();

    tracker.begin(&first).unwrap();
    tracker.connection_lost();
    assert!(tracker.is_outcome_unknown());
    assert_eq!(
        tracker.begin(&second).unwrap_err(),
        LocalControlErrorKind::OutcomeUnknown
    );

    let ping = client_message(31, None, ProtocolPayload::Ping);
    let pong = ProtocolMessage::new(
        connection("client-a"),
        sequence(32),
        None,
        generation(2),
        Some(sequence(31)),
        ProtocolPayload::Pong,
    )
    .unwrap();
    assert_eq!(
        tracker.reconcile_state(&ping, &pong).unwrap_err(),
        LocalControlErrorKind::UnsupportedOperation
    );
    assert!(tracker.is_outcome_unknown());

    let unauthenticated_clear = client_message(31, Some(runtime), ProtocolPayload::AttachObserver);
    let unrelated_state = ProtocolMessage::new(
        connection("client-a"),
        sequence(32),
        Some(namespace(4)),
        generation(2),
        Some(sequence(31)),
        ProtocolPayload::RuntimeSnapshot {
            truth: RuntimeTruth {
                ownership: OwnershipState::OwnershipLost,
                process_liveness: ProcessLiveness::Unknown,
                endpoint_availability: EndpointAvailability::Unknown,
                continuity: ContinuityClass::Unknown,
            },
        },
    )
    .unwrap();
    assert_eq!(
        tracker
            .reconcile_state(&unauthenticated_clear, &unrelated_state)
            .unwrap_err(),
        LocalControlErrorKind::UnknownRuntime
    );
    assert!(tracker.is_outcome_unknown());

    let state_query = client_message(32, Some(runtime), ProtocolPayload::AttachObserver);
    let state_response = ProtocolMessage::new(
        connection("client-a"),
        sequence(33),
        Some(runtime),
        generation(2),
        Some(sequence(32)),
        ProtocolPayload::RuntimeSnapshot {
            truth: RuntimeTruth {
                ownership: OwnershipState::OwnershipLost,
                process_liveness: ProcessLiveness::Unknown,
                endpoint_availability: EndpointAvailability::Unknown,
                continuity: ContinuityClass::Unknown,
            },
        },
    )
    .unwrap();
    tracker
        .reconcile_state(&state_query, &state_response)
        .unwrap();
    assert!(!tracker.is_outcome_unknown());
    tracker.begin(&second).unwrap();
    let wrong_kind_response = ProtocolMessage::new(
        connection("client-a"),
        sequence(32),
        Some(runtime),
        generation(2),
        Some(sequence(31)),
        ProtocolPayload::RuntimeSnapshot {
            truth: RuntimeTruth {
                ownership: OwnershipState::OwnershipLost,
                process_liveness: ProcessLiveness::Unknown,
                endpoint_availability: EndpointAvailability::Unknown,
                continuity: ContinuityClass::Unknown,
            },
        },
    )
    .unwrap();
    assert_eq!(
        tracker.acknowledge(&wrong_kind_response).unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );

    let wrong_response = ProtocolMessage::new(
        connection("client-a"),
        sequence(32),
        Some(namespace(4)),
        generation(2),
        Some(sequence(31)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::UnsupportedOperation,
        },
    )
    .unwrap();
    assert_eq!(
        tracker.acknowledge(&wrong_response).unwrap_err(),
        LocalControlErrorKind::UnknownRuntime
    );

    let response = ProtocolMessage::new(
        connection("client-a"),
        sequence(32),
        Some(runtime),
        generation(2),
        Some(sequence(31)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::UnsupportedOperation,
        },
    )
    .unwrap();
    tracker.acknowledge(&response).unwrap();

    let third = client_message(
        33,
        Some(runtime),
        ProtocolPayload::Resize {
            columns: 120,
            rows: 50,
        },
    );
    tracker.begin(&third).unwrap();
    let explicit_unknown = ProtocolMessage::new(
        connection("client-a"),
        sequence(34),
        Some(runtime),
        generation(2),
        Some(sequence(33)),
        ProtocolPayload::Error {
            kind: LocalControlErrorKind::OutcomeUnknown,
        },
    )
    .unwrap();
    assert_eq!(
        tracker.acknowledge(&explicit_unknown).unwrap_err(),
        LocalControlErrorKind::OutcomeUnknown
    );
    assert!(tracker.is_outcome_unknown());
    let fourth = client_message(35, Some(runtime), ProtocolPayload::Stop);
    assert_eq!(
        tracker.begin(&fourth).unwrap_err(),
        LocalControlErrorKind::OutcomeUnknown
    );
}

#[test]
fn t148_terminal_or_agent_text_shaped_like_control_json_remains_plain_data() {
    let runtime = namespace(3);
    let forged = r#"{"protocol_version":1,"message_kind":"INPUT","accepted":true,"VERIFIED":true}"#;
    let message = ProtocolMessage::new(
        connection("client-a"),
        sequence(40),
        Some(runtime),
        generation(2),
        None,
        ProtocolPayload::OutputEvent {
            chunk: forged.to_owned(),
        },
    )
    .unwrap();
    let decoded = decode_frame(&encode_frame(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);
    match decoded.payload {
        ProtocolPayload::OutputEvent { chunk } => assert_eq!(chunk, forged),
        _ => panic!("output text must remain an output-data field"),
    }
}

#[test]
fn t148_body_invariants_reject_ambiguous_control_state_history_and_resize() {
    let runtime = namespace(3);
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(50),
            Some(runtime),
            generation(2),
            Some(sequence(49)),
            ProtocolPayload::ControlState {
                authority: ClientAuthority::Controller,
                controller_client_id: None,
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(50),
            Some(runtime),
            generation(2),
            Some(sequence(49)),
            ProtocolPayload::ControlState {
                authority: ClientAuthority::Observer,
                controller_client_id: Some(connection("client-a")),
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(50),
            Some(runtime),
            generation(2),
            None,
            ProtocolPayload::HistoryGap {
                first_available_sequence: sequence(10),
                last_dropped_sequence: sequence(10),
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
    assert_eq!(
        ProtocolMessage::new(
            connection("client-a"),
            sequence(50),
            Some(runtime),
            generation(2),
            None,
            ProtocolPayload::Resize {
                columns: 0,
                rows: 40
            },
        )
        .unwrap_err(),
        LocalControlErrorKind::MalformedFrame
    );
}

#[test]
fn t148_implementation_contains_no_endpoint_listener_or_generic_dispatch_surface() {
    let source = include_str!("persistent_runtime/protocol.rs");
    for prohibited in [
        "TcpListener",
        "UnixListener",
        "NamedPipe",
        "SHUTDOWN_IF_IDLE",
        "generic_dispatch",
    ] {
        assert!(!source.contains(prohibited), "{prohibited}");
    }
}
