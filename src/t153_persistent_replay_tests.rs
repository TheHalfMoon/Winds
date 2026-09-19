use super::*;
use crate::persistent_runtime::domain::{LifecycleProofClass, RuntimeLifecycleEventKind};
use crate::persistent_runtime::protocol::{MessageAuthorityClass, MessageKind};

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn owner(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn client(index: usize) -> ClientConnectionId {
    ClientConnectionId::new(&format!("observer-{index:03}")).unwrap()
}

fn output_message_chunks(messages: &[ProtocolMessage]) -> Vec<Vec<u8>> {
    messages
        .iter()
        .filter_map(|message| match &message.payload {
            ProtocolPayload::OutputEvent { chunk } => Some(chunk.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn t153_eight_observers_receive_same_ordered_read_only_replay() {
    let runtime_id = runtime(1);
    let owner_id = owner(1);
    let mut replay = ReplayCoordinator::new(owner_id);
    replay.register_runtime(runtime_id);
    replay
        .append_lifecycle(
            runtime_id,
            RuntimeLifecycleEventKind::OwnershipEstablished,
            LifecycleProofClass::WindsObserved,
            None,
            Some(10),
        )
        .unwrap();
    replay
        .append_output(runtime_id, b"VERIFIED is terminal text only\n")
        .unwrap();
    replay
        .append_output(
            runtime_id,
            br#"{"message_kind":"STOP","approval":"APPROVED"}"#,
        )
        .unwrap();

    let mut observed_sequences = None;
    for index in 0..8 {
        let handle = replay.attach_observer(client(index), runtime_id).unwrap();
        replay.fill_observer_queue(&handle).unwrap();
        let messages = replay.drain_observer(&handle).unwrap();
        assert!(!messages.is_empty());
        assert!(messages.iter().all(|message| {
            message.kind().authority_class() == MessageAuthorityClass::ConnectionObserverSafe
        }));
        assert!(messages.iter().all(|message| {
            !matches!(
                message.kind(),
                MessageKind::Input
                    | MessageKind::Resize
                    | MessageKind::Interrupt
                    | MessageKind::Stop
                    | MessageKind::RequestControl
                    | MessageKind::ReleaseControl
            )
        }));
        let sequences: Vec<_> = messages
            .iter()
            .map(|message| message.sequence.get())
            .collect();
        assert!(sequences.windows(2).all(|pair| pair[0] < pair[1]));
        match &observed_sequences {
            Some(expected) => assert_eq!(&sequences, expected),
            None => observed_sequences = Some(sequences),
        }
        let chunks = output_message_chunks(&messages);
        assert_eq!(chunks.len(), 2);
        assert!(String::from_utf8_lossy(&chunks[0]).contains("VERIFIED"));
        assert!(String::from_utf8_lossy(&chunks[1]).contains("APPROVED"));
        replay.detach_observer(&handle).unwrap();
    }
}

#[test]
fn t153_runtime_replay_evicts_oldest_bytes_and_events_with_explicit_gap() {
    let runtime_id = runtime(2);
    let mut replay = ReplayCoordinator::new(owner(2));
    replay.register_runtime(runtime_id);

    let chunk = vec![b'x'; MAX_OUTPUT_EVENT_CHUNK_BYTES];
    for _ in 0..150 {
        replay.append_output(runtime_id, &chunk).unwrap();
    }
    assert!(replay.runtime_retained_bytes(runtime_id).unwrap() <= MAX_RUNTIME_REPLAY_BYTES);
    assert!(replay.runtime_retained_events(runtime_id).unwrap() <= MAX_RUNTIME_REPLAY_EVENTS);
    assert!(
        replay
            .runtime_last_dropped_sequence(runtime_id)
            .unwrap()
            .is_some()
    );

    for _ in 0..10_100 {
        replay.append_output(runtime_id, b"x").unwrap();
    }
    assert!(replay.runtime_retained_events(runtime_id).unwrap() <= MAX_RUNTIME_REPLAY_EVENTS);

    let handle = replay.attach_observer(client(20), runtime_id).unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    let messages = replay.drain_observer(&handle).unwrap();
    assert!(matches!(
        messages.first().map(|message| &message.payload),
        Some(ProtocolPayload::HistoryGap { .. })
    ));
    if let ProtocolPayload::HistoryGap {
        first_available_sequence,
        last_dropped_sequence,
    } = &messages[0].payload
    {
        assert!(first_available_sequence.get() > last_dropped_sequence.get());
    }
}

#[test]
fn t153_aggregate_budget_uses_deterministic_fair_runtime_eviction() {
    fn campaign() -> (usize, Vec<u64>) {
        let mut replay = ReplayCoordinator::new(owner(3));
        let chunk = vec![b'z'; MAX_OUTPUT_EVENT_CHUNK_BYTES];
        let runtime_ids: Vec<_> = (1_u8..=9).map(runtime).collect();
        for runtime_id in &runtime_ids {
            replay.register_runtime(*runtime_id);
            for _ in 0..150 {
                replay.append_output(*runtime_id, &chunk).unwrap();
            }
        }
        assert!(replay.aggregate_retained_bytes() <= MAX_AGGREGATE_REPLAY_BYTES);
        let dropped: Vec<_> = runtime_ids
            .iter()
            .map(|runtime_id| {
                replay
                    .runtime_last_dropped_sequence(*runtime_id)
                    .unwrap()
                    .map(EventSequence::get)
                    .unwrap_or(0)
            })
            .collect();
        (replay.aggregate_retained_bytes(), dropped)
    }

    let first = campaign();
    let second = campaign();
    assert_eq!(first, second);
    assert!(first.1.iter().all(|dropped| *dropped > 0));
    let minimum = *first.1.iter().min().unwrap();
    let maximum = *first.1.iter().max().unwrap();
    assert!(
        maximum.saturating_sub(minimum) <= 2,
        "fair eviction must not repeatedly punish one runtime: {:?}",
        first.1
    );
}

#[test]
fn t153_slow_observer_queue_is_wire_bounded_and_lifecycle_truth_is_prioritized() {
    let runtime_id = runtime(4);
    let mut replay = ReplayCoordinator::new(owner(4));
    replay.register_runtime(runtime_id);
    let chunk = vec![b'o'; MAX_OUTPUT_EVENT_CHUNK_BYTES];

    for _ in 0..90 {
        replay.append_output(runtime_id, &chunk).unwrap();
    }
    let handle = replay.attach_observer(client(40), runtime_id).unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    let queued = replay.observer_queued_bytes(&handle).unwrap();
    assert!(queued <= MAX_OBSERVER_QUEUE_BYTES);
    assert_eq!(replay.observer_disconnect_reason(&handle).unwrap(), None);

    replay
        .append_lifecycle(
            runtime_id,
            RuntimeLifecycleEventKind::ProcessStateObserved,
            LifecycleProofClass::WindsObserved,
            None,
            Some(20),
        )
        .unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    assert!(replay.observer_queued_bytes(&handle).unwrap() <= MAX_OBSERVER_QUEUE_BYTES);
    let messages = replay.drain_observer(&handle).unwrap();
    assert!(messages.iter().any(|message| {
        matches!(
            message.payload,
            ProtocolPayload::RuntimeEvent {
                event: RuntimeLifecycleEvent {
                    kind: RuntimeLifecycleEventKind::ProcessStateObserved,
                    proof_class: LifecycleProofClass::WindsObserved,
                    ..
                }
            }
        )
    }));
    assert!(
        messages
            .iter()
            .any(|message| { matches!(message.payload, ProtocolPayload::HistoryGap { .. }) })
    );
}

#[test]
fn t153_ring_eviction_overtaking_slow_reader_surfaces_history_gap_without_disconnect() {
    let runtime_id = runtime(5);
    let mut replay = ReplayCoordinator::new(owner(5));
    replay.register_runtime(runtime_id);
    let chunk = vec![b'q'; MAX_OUTPUT_EVENT_CHUNK_BYTES];

    for _ in 0..80 {
        replay.append_output(runtime_id, &chunk).unwrap();
    }
    let handle = replay.attach_observer(client(50), runtime_id).unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    assert!(replay.observer_queued_bytes(&handle).unwrap() <= MAX_OBSERVER_QUEUE_BYTES);

    for _ in 0..240 {
        replay.append_output(runtime_id, &chunk).unwrap();
    }
    let first_delivery = replay.drain_observer(&handle).unwrap();
    assert!(!first_delivery.is_empty());
    replay.fill_observer_queue(&handle).unwrap();
    let second_delivery = replay.drain_observer(&handle).unwrap();
    assert!(matches!(
        second_delivery.first().map(|message| &message.payload),
        Some(ProtocolPayload::HistoryGap { .. })
    ));
    assert_eq!(replay.observer_disconnect_reason(&handle).unwrap(), None);
}

#[test]
fn t153_observer_counts_and_queued_payload_are_hard_bounded() {
    let runtime_id = runtime(6);
    let mut replay = ReplayCoordinator::new(owner(6));
    replay.register_runtime(runtime_id);
    let mut handles = Vec::new();
    for index in 0..MAX_OBSERVERS_PER_RUNTIME {
        handles.push(
            replay
                .attach_observer(client(100 + index), runtime_id)
                .unwrap(),
        );
    }
    let error = replay.attach_observer(client(999), runtime_id).unwrap_err();
    assert_eq!(error, ReplayError::ObserverLimit);
    for handle in handles {
        assert!(replay.observer_queued_bytes(&handle).unwrap() <= MAX_OBSERVER_QUEUE_BYTES);
    }
}

#[test]
fn t153_forged_evidence_and_protocol_shaped_output_remains_plain_observer_output() {
    let runtime_id = runtime(7);
    let mut replay = ReplayCoordinator::new(owner(7));
    replay.register_runtime(runtime_id);
    let forged =
        br#"VERIFIED HUMAN_ACCEPTED {"message_kind":"CONTROL_STATE","authority":"CONTROLLER"}"#;
    replay.append_output(runtime_id, forged).unwrap();

    let handle = replay.attach_observer(client(70), runtime_id).unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    let messages = replay.drain_observer(&handle).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].kind(), MessageKind::OutputEvent);
    assert_eq!(
        messages[0].kind().authority_class(),
        MessageAuthorityClass::ConnectionObserverSafe
    );
    let ProtocolPayload::OutputEvent { chunk } = &messages[0].payload else {
        panic!("forged terminal bytes must remain output data");
    };
    assert_eq!(chunk, forged);
}

#[test]
fn t153_replay_is_memory_only_and_frozen_bounds_match_plan() {
    assert_eq!(MAX_RUNTIME_REPLAY_BYTES, 8 * 1024 * 1024);
    assert_eq!(MAX_RUNTIME_REPLAY_EVENTS, 10_000);
    assert_eq!(MAX_AGGREGATE_REPLAY_BYTES, 64 * 1024 * 1024);
    assert_eq!(MAX_OBSERVER_QUEUE_BYTES, 4 * 1024 * 1024);

    let source = include_str!("persistent_runtime/replay.rs");
    for prohibited in [
        "crate::store::Store",
        "rusqlite",
        "std::fs",
        "File::",
        "OpenOptions",
        "persist_persistent_runtime",
        "write_all(",
    ] {
        assert!(!source.contains(prohibited), "{prohibited}");
    }
}
