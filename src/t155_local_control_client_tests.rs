use super::*;
use crate::persistent_runtime::domain::{
    ContinuityClass, EndpointAvailability, OwnershipState, ProcessLiveness,
};
use crate::persistent_runtime::protocol::{ProtocolMessage, ProtocolPayload};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn generation(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn sequence(value: u64) -> EventSequence {
    EventSequence::new(value).unwrap()
}

fn connection(value: &str) -> ClientConnectionId {
    ClientConnectionId::new(value).unwrap()
}

fn running_truth() -> RuntimeTruth {
    RuntimeTruth {
        ownership: OwnershipState::LiveOwned,
        process_liveness: ProcessLiveness::Running,
        endpoint_availability: EndpointAvailability::Available,
        continuity: ContinuityClass::RetainedLiveProcess,
    }
}

fn response(
    connection_id: &ClientConnectionId,
    owner_generation_id: OwnerGenerationId,
    runtime_namespace_id: Option<RuntimeNamespaceId>,
    response_sequence: u64,
    correlation_sequence: u64,
    payload: ProtocolPayload,
) -> ProtocolMessage {
    ProtocolMessage::new(
        connection_id.clone(),
        sequence(response_sequence),
        runtime_namespace_id,
        owner_generation_id,
        Some(sequence(correlation_sequence)),
        payload,
    )
    .unwrap()
}

fn event(
    connection_id: &ClientConnectionId,
    owner_generation_id: OwnerGenerationId,
    runtime_namespace_id: Option<RuntimeNamespaceId>,
    event_sequence: u64,
    payload: ProtocolPayload,
) -> ProtocolMessage {
    ProtocolMessage::new(
        connection_id.clone(),
        sequence(event_sequence),
        runtime_namespace_id,
        owner_generation_id,
        None,
        payload,
    )
    .unwrap()
}

fn hello_ack(
    owner_generation_id: OwnerGenerationId,
    connection_id: &ClientConnectionId,
) -> ProtocolMessage {
    response(
        connection_id,
        owner_generation_id,
        None,
        100,
        1,
        ProtocolPayload::HelloAck,
    )
}

fn snapshot_response(
    owner_generation_id: OwnerGenerationId,
    connection_id: &ClientConnectionId,
    runtime_namespace_id: RuntimeNamespaceId,
    correlation_sequence: u64,
    response_sequence: u64,
) -> ProtocolMessage {
    response(
        connection_id,
        owner_generation_id,
        Some(runtime_namespace_id),
        response_sequence,
        correlation_sequence,
        ProtocolPayload::RuntimeSnapshot {
            truth: running_truth(),
        },
    )
}

fn control_response(
    owner_generation_id: OwnerGenerationId,
    connection_id: &ClientConnectionId,
    runtime_namespace_id: RuntimeNamespaceId,
    correlation_sequence: u64,
    response_sequence: u64,
    authority: ClientAuthority,
) -> ProtocolMessage {
    response(
        connection_id,
        owner_generation_id,
        Some(runtime_namespace_id),
        response_sequence,
        correlation_sequence,
        ProtocolPayload::ControlState {
            authority,
            controller_client_id: match authority {
                ClientAuthority::Controller => Some(connection_id.clone()),
                ClientAuthority::Observer => None,
            },
        },
    )
}

struct ScriptedWire {
    sent: Arc<Mutex<Vec<ProtocolMessage>>>,
    receive: VecDeque<Result<ProtocolMessage, WireError>>,
}

impl ScriptedWire {
    fn new(
        receive: Vec<Result<ProtocolMessage, WireError>>,
    ) -> (Self, Arc<Mutex<Vec<ProtocolMessage>>>) {
        let sent = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                sent: Arc::clone(&sent),
                receive: receive.into(),
            },
            sent,
        )
    }
}

impl LocalControlWire for ScriptedWire {
    fn send(&mut self, message: &ProtocolMessage) -> Result<(), WireError> {
        self.sent.lock().unwrap().push(message.clone());
        Ok(())
    }

    fn receive(&mut self) -> Result<ProtocolMessage, WireError> {
        self.receive.pop_front().unwrap_or_else(|| {
            Err(WireError::Transport(
                "scripted local-control receive exhausted".to_owned(),
            ))
        })
    }
}

#[test]
fn t155_exact_generation_handshake_binds_connection_and_rejects_stale_owner() {
    let expected = generation(1);
    let accepted_connection = connection("accepted-client");
    let (wire, sent) = ScriptedWire::new(vec![Ok(hello_ack(expected, &accepted_connection))]);
    let client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(expected)).unwrap();

    assert_eq!(client.owner_generation_id(), expected);
    assert_eq!(client.connection_id(), &accepted_connection);
    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert!(matches!(
        sent[0].payload,
        ProtocolPayload::Hello {
            minimum_protocol_version: PROTOCOL_VERSION,
            maximum_protocol_version: PROTOCOL_VERSION,
            expected_owner_generation_id: Some(value),
        } if value == expected
    ));

    let replacement = generation(2);
    let (wire, _) = ScriptedWire::new(vec![Ok(hello_ack(
        replacement,
        &connection("replacement-client"),
    ))]);
    let error = RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(expected))
        .err()
        .unwrap();
    assert_eq!(error, LocalControlClientError::StaleOwnerGeneration);
}

#[test]
fn t155_protocol_mismatch_is_distinct_from_transport_and_generation_failure() {
    let (wire, _) = ScriptedWire::new(vec![Err(WireError::Protocol(
        LocalControlErrorKind::ProtocolMismatch,
    ))]);
    let error = RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), None)
        .err()
        .unwrap();
    assert_eq!(error, LocalControlClientError::ProtocolMismatch);
}

#[test]
fn t155_duplicate_alias_requires_explicit_id_disambiguation_before_consequential_action() {
    let alias = RuntimeAlias::new("same-name").unwrap();
    let first = runtime(3);
    let second = runtime(4);
    let inventory = vec![
        RuntimeInventoryEntry {
            runtime_namespace_id: first,
            runtime_alias: alias.clone(),
        },
        RuntimeInventoryEntry {
            runtime_namespace_id: second,
            runtime_alias: alias.clone(),
        },
    ];

    assert_eq!(
        resolve_runtime_target(&inventory, &RuntimeTargetSelector::Alias(alias.clone()))
            .unwrap_err(),
        LocalControlClientError::AmbiguousAlias {
            alias: "same-name".to_owned(),
            matches: 2,
        }
    );
    assert_eq!(
        resolve_runtime_target(&inventory, &RuntimeTargetSelector::Exact(second))
            .unwrap()
            .runtime_namespace_id(),
        second
    );
    assert_eq!(
        resolve_runtime_target(
            &inventory,
            &RuntimeTargetSelector::Alias(RuntimeAlias::new("missing").unwrap()),
        )
        .unwrap_err(),
        LocalControlClientError::UnknownAlias("missing".to_owned())
    );
}

#[test]
fn t155_observer_attach_and_controller_transition_keep_exact_runtime_and_queue_interleaved_event() {
    let owner_generation = generation(5);
    let client_id = connection("observer-controller");
    let runtime_id = runtime(5);
    let output = event(
        &client_id,
        owner_generation,
        Some(runtime_id),
        103,
        ProtocolPayload::OutputEvent {
            chunk: b"ordered-output".to_vec(),
        },
    );
    let (wire, sent) = ScriptedWire::new(vec![
        Ok(hello_ack(owner_generation, &client_id)),
        Ok(snapshot_response(
            owner_generation,
            &client_id,
            runtime_id,
            2,
            101,
        )),
        Ok(output),
        Ok(control_response(
            owner_generation,
            &client_id,
            runtime_id,
            3,
            104,
            ClientAuthority::Controller,
        )),
        Ok(control_response(
            owner_generation,
            &client_id,
            runtime_id,
            4,
            105,
            ClientAuthority::Observer,
        )),
    ]);
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(owner_generation))
            .unwrap();
    let target = ResolvedRuntimeTarget::exact(runtime_id);

    assert!(matches!(
        client.attach_observer(target).unwrap(),
        ClientResponseProjection::RuntimeSnapshot {
            runtime_namespace_id,
            ..
        } if runtime_namespace_id == runtime_id
    ));
    assert!(matches!(
        client.request_control(target).unwrap(),
        ClientResponseProjection::ControlState {
            runtime_namespace_id,
            authority: ClientAuthority::Controller,
            ..
        } if runtime_namespace_id == runtime_id
    ));
    assert!(matches!(
        client.pop_event().unwrap(),
        Some(ClientEventProjection::Output {
            runtime_namespace_id,
            chunk,
        }) if runtime_namespace_id == runtime_id && chunk == b"ordered-output"
    ));
    assert!(matches!(
        client.release_control(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Observer,
            ..
        }
    ));

    let sent = sent.lock().unwrap();
    assert_eq!(sent.len(), 4);
    assert!(matches!(sent[1].payload, ProtocolPayload::AttachObserver));
    assert!(matches!(sent[2].payload, ProtocolPayload::RequestControl));
    assert!(matches!(sent[3].payload, ProtocolPayload::ReleaseControl));
    assert!(
        sent[1..]
            .iter()
            .all(|message| message.runtime_namespace_id == Some(runtime_id))
    );
}

#[test]
fn t155_lost_mutation_response_becomes_outcome_unknown_until_same_generation_reconciliation() {
    let owner_generation = generation(6);
    let runtime_id = runtime(6);
    let first_connection = connection("first-connection");
    let (wire, _) = ScriptedWire::new(vec![
        Ok(hello_ack(owner_generation, &first_connection)),
        Ok(snapshot_response(
            owner_generation,
            &first_connection,
            runtime_id,
            2,
            101,
        )),
        Err(WireError::Transport(
            "connection lost after mutation send".to_owned(),
        )),
    ]);
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(owner_generation))
            .unwrap();
    let target = ResolvedRuntimeTarget::exact(runtime_id);
    client.attach_observer(target).unwrap();

    assert_eq!(
        client
            .send_input(target, b"possibly-applied".to_vec())
            .unwrap_err(),
        LocalControlClientError::OutcomeUnknown(runtime_id)
    );
    assert!(client.is_outcome_unknown());
    assert_eq!(
        client.request_control(target).unwrap_err(),
        LocalControlClientError::OutcomeUnknown(runtime_id)
    );

    let replacement_connection = connection("reconnected-client");
    let (replacement_wire, _) = ScriptedWire::new(vec![
        Ok(hello_ack(owner_generation, &replacement_connection)),
        Ok(snapshot_response(
            owner_generation,
            &replacement_connection,
            runtime_id,
            2,
            201,
        )),
        Ok(control_response(
            owner_generation,
            &replacement_connection,
            runtime_id,
            3,
            202,
            ClientAuthority::Controller,
        )),
    ]);
    client
        .reconnect_with_wire_for_test(Box::new(replacement_wire))
        .unwrap();
    assert!(client.is_outcome_unknown());
    client.reconcile_runtime(target).unwrap();
    assert!(!client.is_outcome_unknown());
    assert!(matches!(
        client.request_control(target).unwrap(),
        ClientResponseProjection::ControlState {
            authority: ClientAuthority::Controller,
            ..
        }
    ));
}

#[test]
fn t155_reconnect_never_treats_replacement_owner_generation_as_continuation() {
    let first_generation = generation(7);
    let first_connection = connection("generation-one");
    let (wire, _) = ScriptedWire::new(vec![Ok(hello_ack(first_generation, &first_connection))]);
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(first_generation))
            .unwrap();

    let second_generation = generation(8);
    let (replacement_wire, _) = ScriptedWire::new(vec![Ok(hello_ack(
        second_generation,
        &connection("generation-two"),
    ))]);
    assert_eq!(
        client
            .reconnect_with_wire_for_test(Box::new(replacement_wire))
            .unwrap_err(),
        LocalControlClientError::StaleOwnerGeneration
    );
    assert_eq!(client.owner_generation_id(), first_generation);
}

#[test]
fn t155_remote_and_arbitrary_endpoint_syntax_is_rejected_before_transport_access() {
    for rejected in [
        "tcp://127.0.0.1:9999",
        "http://localhost",
        "https://example.test",
        "ws://localhost",
        "wss://localhost",
        "\\\\server\\pipe\\winds",
        "/tmp/arbitrary.sock",
    ] {
        assert_eq!(
            validate_local_endpoint_hint(Some(rejected)).unwrap_err(),
            LocalControlClientError::RemoteEndpointRejected,
            "{rejected}"
        );
    }
    for accepted in [None, Some(""), Some("default"), Some("local")] {
        validate_local_endpoint_hint(accepted).unwrap();
    }
}

#[test]
fn t155_client_pending_event_queue_is_hard_bounded_and_never_becomes_unbounded_buffering() {
    let owner_generation = generation(9);
    let client_id = connection("bounded-event-client");
    let runtime_id = runtime(9);
    let (wire, _) = ScriptedWire::new(vec![Ok(hello_ack(owner_generation, &client_id))]);
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(owner_generation))
            .unwrap();
    client.attached_runtimes.insert(runtime_id);

    let mut saw_backpressure = false;
    for index in 0..128_u64 {
        let message = event(
            &client_id,
            owner_generation,
            Some(runtime_id),
            index.saturating_add(10),
            ProtocolPayload::OutputEvent {
                chunk: vec![
                    b'x';
                    crate::persistent_runtime::protocol::MAX_OUTPUT_EVENT_CHUNK_BYTES
                ],
            },
        );
        match client.queue_event(message) {
            Ok(()) => assert!(client.queued_event_bytes() <= MAX_CLIENT_QUEUED_EVENT_BYTES),
            Err(LocalControlClientError::ClientEventBackpressure) => {
                saw_backpressure = true;
                break;
            }
            Err(error) => panic!("unexpected event queue result: {error}"),
        }
    }
    assert!(saw_backpressure);
    assert!(client.queued_event_bytes() <= MAX_CLIENT_QUEUED_EVENT_BYTES);
    assert!(client.wire.is_none());
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn t155_real_unix_private_endpoint_performs_same_user_generation_handshake() {
    use crate::persistent_runtime::protocol::{read_frame, write_frame};
    use crate::persistent_runtime::transport::unix::BoundUnixListener;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);
    let root = std::env::temp_dir().join(format!(
        "winds-t155-unix-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    let listener = BoundUnixListener::bind(&root).unwrap();
    let owner_generation = generation(10);
    let expected_connection = connection("real-unix-client");
    let client_root = root.clone();

    let client_thread = thread::spawn(move || {
        let client = RustLocalControlClient::connect_runtime_directory_for_test(
            &client_root,
            Some(owner_generation),
        )
        .unwrap();
        (
            client.owner_generation_id(),
            client.connection_id().as_str().to_owned(),
        )
    });

    let mut stream = listener.accept_same_user().unwrap();
    let hello = read_frame(&mut stream).unwrap();
    assert!(matches!(hello.payload, ProtocolPayload::Hello { .. }));
    let ack = ProtocolMessage::new(
        expected_connection.clone(),
        sequence(100),
        None,
        owner_generation,
        Some(hello.sequence),
        ProtocolPayload::HelloAck,
    )
    .unwrap();
    write_frame(&mut stream, &ack).unwrap();

    let (actual_generation, actual_connection) = client_thread.join().unwrap();
    assert_eq!(actual_generation, owner_generation);
    assert_eq!(actual_connection, expected_connection.as_str());
    drop(stream);
    drop(listener);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn t155_real_windows_private_named_pipe_performs_principal_and_generation_handshake() {
    use crate::persistent_runtime::protocol::{decode_frame, encode_frame};
    use crate::persistent_runtime::transport::windows::WindowsNamedPipeServer;
    use std::thread;

    fn read_server_message(server: &WindowsNamedPipeServer) -> ProtocolMessage {
        let mut length_bytes = [0_u8; 4];
        server.read_exact(&mut length_bytes).unwrap();
        let claimed = u32::from_le_bytes(length_bytes) as usize;
        assert!(claimed > 0 && claimed <= MAX_INBOUND_CONTROL_FRAME_BYTES);
        let mut payload = vec![0_u8; claimed];
        server.read_exact(&mut payload).unwrap();
        let mut frame = Vec::with_capacity(4 + claimed);
        frame.extend_from_slice(&length_bytes);
        frame.extend_from_slice(&payload);
        decode_frame(&frame).unwrap()
    }

    let owner_generation = generation(11);
    let expected_connection = connection("real-windows-client");
    let expected_connection_for_thread = expected_connection.clone();
    let mut server = WindowsNamedPipeServer::bind(owner_generation).unwrap();
    let client_thread = thread::spawn(move || {
        let client = RustLocalControlClient::connect(None, Some(owner_generation)).unwrap();
        (
            client.owner_generation_id(),
            client.connection_id().as_str().to_owned(),
        )
    });

    server.accept_same_user().unwrap();
    let hello = read_server_message(&server);
    assert!(matches!(hello.payload, ProtocolPayload::Hello { .. }));
    let ack = ProtocolMessage::new(
        expected_connection_for_thread,
        sequence(100),
        None,
        owner_generation,
        Some(hello.sequence),
        ProtocolPayload::HelloAck,
    )
    .unwrap();
    server.write_all(&encode_frame(&ack).unwrap()).unwrap();

    let (actual_generation, actual_connection) = client_thread.join().unwrap();
    assert_eq!(actual_generation, owner_generation);
    assert_eq!(actual_connection, expected_connection.as_str());
}

#[test]
fn t155_client_surface_contains_no_generic_remote_filesystem_git_sql_shell_or_process_dispatcher() {
    let source = include_str!("persistent_runtime/client.rs");
    for prohibited in [
        "TcpStream",
        "UdpSocket",
        "reqwest",
        "WebSocket",
        "std::fs",
        "rusqlite",
        "std::process",
        "Command::new",
        "generic_dispatch",
        "invoke_shell",
        "execute_git",
    ] {
        assert!(!source.contains(prohibited), "{prohibited}");
    }
}
