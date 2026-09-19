use super::*;
use crate::persistent_runtime::domain::{
    ClientAuthority, ClientConnectionId, ContinuityClass, EndpointAvailability, EventSequence,
    OwnerGenerationId, OwnershipState, ProcessLiveness, RuntimeNamespaceId, RuntimeTruth,
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
    truth: RuntimeTruth,
) -> ProtocolMessage {
    response(
        connection_id,
        owner_generation_id,
        Some(runtime_namespace_id),
        response_sequence,
        correlation_sequence,
        ProtocolPayload::RuntimeSnapshot { truth },
    )
}

fn control_response(
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
        ProtocolPayload::ControlState {
            authority: ClientAuthority::Controller,
            controller_client_id: Some(connection_id.clone()),
        },
    )
}

fn running_truth() -> RuntimeTruth {
    RuntimeTruth {
        ownership: OwnershipState::LiveOwned,
        process_liveness: ProcessLiveness::Running,
        endpoint_availability: EndpointAvailability::Available,
        continuity: ContinuityClass::RetainedLiveProcess,
    }
}

fn stopped_truth() -> RuntimeTruth {
    RuntimeTruth {
        ownership: OwnershipState::Unowned,
        process_liveness: ProcessLiveness::Exited,
        endpoint_availability: EndpointAvailability::Available,
        continuity: ContinuityClass::RetainedLiveProcess,
    }
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
                "scripted T157 local-control receive exhausted".to_owned(),
            ))
        })
    }
}

fn stop_send_count(sent: &Arc<Mutex<Vec<ProtocolMessage>>>) -> usize {
    sent.lock()
        .unwrap()
        .iter()
        .filter(|message| matches!(message.payload, ProtocolPayload::Stop))
        .count()
}

#[test]
fn t157_lost_stop_response_becomes_outcome_unknown_without_automatic_retry() {
    let owner_generation = generation(0x61);
    let runtime_id = runtime(0x61);
    let first_connection = connection("t157-stop-first");
    let (wire, sent) = ScriptedWire::new(vec![
        Ok(hello_ack(owner_generation, &first_connection)),
        Ok(snapshot_response(
            owner_generation,
            &first_connection,
            runtime_id,
            2,
            101,
            running_truth(),
        )),
        Ok(control_response(
            owner_generation,
            &first_connection,
            runtime_id,
            3,
            102,
        )),
        Err(WireError::Transport(
            "connection lost after exact STOP send".to_owned(),
        )),
    ]);
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(owner_generation))
            .unwrap();
    let target = ResolvedRuntimeTarget::exact(runtime_id);

    client.attach_observer(target).unwrap();
    client.request_control(target).unwrap();

    assert_eq!(
        client.stop(target).unwrap_err(),
        LocalControlClientError::OutcomeUnknown(runtime_id)
    );
    assert!(client.is_outcome_unknown());
    assert_eq!(stop_send_count(&sent), 1);

    assert_eq!(
        client.stop(target).unwrap_err(),
        LocalControlClientError::OutcomeUnknown(runtime_id)
    );
    assert_eq!(
        stop_send_count(&sent),
        1,
        "unknown STOP outcome must never trigger an automatic second STOP"
    );

    let replacement_connection = connection("t157-stop-reconcile");
    let (replacement_wire, replacement_sent) = ScriptedWire::new(vec![
        Ok(hello_ack(owner_generation, &replacement_connection)),
        Ok(snapshot_response(
            owner_generation,
            &replacement_connection,
            runtime_id,
            2,
            201,
            stopped_truth(),
        )),
    ]);
    client
        .reconnect_with_wire_for_test(Box::new(replacement_wire))
        .unwrap();
    assert!(
        client.is_outcome_unknown(),
        "reconnect alone cannot erase an unknown consequential outcome"
    );

    let projection = client.reconcile_runtime(target).unwrap();
    assert!(matches!(
        projection,
        ClientResponseProjection::RuntimeSnapshot {
            truth: RuntimeTruth {
                ownership: OwnershipState::Unowned,
                process_liveness: ProcessLiveness::Exited,
                ..
            },
            ..
        }
    ));
    assert!(!client.is_outcome_unknown());
    assert_eq!(stop_send_count(&replacement_sent), 0);
}

#[test]
fn t157_unknown_stop_outcome_cannot_be_reconciled_across_owner_generation_change() {
    let first_generation = generation(0x62);
    let runtime_id = runtime(0x62);
    let first_connection = connection("t157-generation-first");
    let (wire, sent) = ScriptedWire::new(vec![
        Ok(hello_ack(first_generation, &first_connection)),
        Ok(snapshot_response(
            first_generation,
            &first_connection,
            runtime_id,
            2,
            101,
            running_truth(),
        )),
        Ok(control_response(
            first_generation,
            &first_connection,
            runtime_id,
            3,
            102,
        )),
        Err(WireError::Transport(
            "connection lost after STOP during owner crash".to_owned(),
        )),
    ]);
    let mut client =
        RustLocalControlClient::connect_with_wire_for_test(Box::new(wire), Some(first_generation))
            .unwrap();
    let target = ResolvedRuntimeTarget::exact(runtime_id);
    client.attach_observer(target).unwrap();
    client.request_control(target).unwrap();
    assert_eq!(
        client.stop(target).unwrap_err(),
        LocalControlClientError::OutcomeUnknown(runtime_id)
    );
    assert_eq!(stop_send_count(&sent), 1);

    let replacement_generation = generation(0x63);
    let replacement_connection = connection("t157-generation-replacement");
    let (replacement_wire, replacement_sent) = ScriptedWire::new(vec![Ok(hello_ack(
        replacement_generation,
        &replacement_connection,
    ))]);
    assert_eq!(
        client
            .reconnect_with_wire_for_test(Box::new(replacement_wire))
            .unwrap_err(),
        LocalControlClientError::StaleOwnerGeneration
    );
    assert!(client.is_outcome_unknown());
    assert_eq!(stop_send_count(&replacement_sent), 0);
}
