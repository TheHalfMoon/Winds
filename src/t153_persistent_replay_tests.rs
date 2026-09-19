use super::*;
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::persistent_runtime::domain::{
    LifecycleProofClass, RuntimeAlias, RuntimeLifecycleEventKind,
};
use crate::persistent_runtime::owner::PersistentOwner;
use crate::persistent_runtime::protocol::{MessageAuthorityClass, MessageKind, ProtocolPayload};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

fn runtime(byte: u8) -> RuntimeNamespaceId {
    RuntimeNamespaceId::from_entropy_bytes([byte; 16]).unwrap()
}

fn owner(byte: u8) -> OwnerGenerationId {
    OwnerGenerationId::from_entropy_bytes([byte; 16]).unwrap()
}

fn client(index: usize) -> ClientConnectionId {
    ClientConnectionId::new(&format!("observer-{index:03}")).unwrap()
}

static NEXT_OWNER_REPLAY_ROOT: AtomicU64 = AtomicU64::new(1);

fn owner_replay_root(label: &str) -> PathBuf {
    let sequence = NEXT_OWNER_REPLAY_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t153-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn owner_replay_runtime_root() -> PathBuf {
    let sequence = NEXT_OWNER_REPLAY_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("w153r-{sequence}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path.canonicalize().unwrap()
}

fn owner_replay_shell_profile(root: &Path) -> ShellProfile {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let candidate = "/bin/sh".to_owned();
    #[cfg(windows)]
    let candidate = std::env::var("COMSPEC").expect("Windows CI must provide COMSPEC");

    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.to_string_lossy().into_owned(),
        git_common_dir: root.to_string_lossy().into_owned(),
        shell_candidates: vec![candidate.clone()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .unwrap()
        .into_iter()
        .find(|profile| {
            #[cfg(windows)]
            {
                profile.executable.eq_ignore_ascii_case(&candidate)
            }
            #[cfg(not(windows))]
            {
                profile.executable == candidate
            }
        })
        .expect("accepted native shell profile must be discoverable")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn start_owner_replay_fixture(
    home: &Path,
    runtime_root: &Path,
    now_unix_ms: i64,
) -> PersistentOwner {
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .unwrap();
    PersistentOwner::start_for_test(home, &runtime_directory, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn start_owner_replay_fixture(
    home: &Path,
    _runtime_root: &Path,
    now_unix_ms: i64,
) -> PersistentOwner {
    PersistentOwner::start(home, now_unix_ms).unwrap()
}

#[cfg(windows)]
fn prime_owner_replay_windows_terminal(
    owner: &mut PersistentOwner,
    attachment: &crate::persistent_runtime::runtime::PersistentTerminalAttachment,
) {
    const CURSOR_QUERY: &[u8] = b"[6n";
    const CURSOR_RESPONSE: &[u8] = b"[1;1R";
    let mut observed = Vec::new();
    for _ in 0..32 {
        let mut buffer = [0_u8; 4096];
        let count = owner
            .read_terminal_runtime_output(attachment, &mut buffer)
            .unwrap();
        if count == 0 {
            break;
        }
        observed.extend_from_slice(&buffer[..count]);
        if observed
            .windows(CURSOR_QUERY.len())
            .any(|window| window == CURSOR_QUERY)
        {
            owner
                .send_terminal_runtime_input(attachment, CURSOR_RESPONSE)
                .unwrap();
            return;
        }
        assert!(observed.len() <= 128 * 1024);
    }
    panic!(
        "headless native-Windows ConPTY did not request cursor position; observed {:?}",
        String::from_utf8_lossy(&observed)
    );
}

#[cfg(not(windows))]
fn prime_owner_replay_windows_terminal(
    _owner: &mut PersistentOwner,
    _attachment: &crate::persistent_runtime::runtime::PersistentTerminalAttachment,
) {
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
fn t153_tail_source_gap_is_visible_without_waiting_for_a_future_record() {
    let runtime_id = runtime(8);
    let mut replay = ReplayCoordinator::new(owner(8));
    replay.register_runtime(runtime_id);
    replay.append_output(runtime_id, b"before-gap").unwrap();

    let handle = replay.attach_observer(client(80), runtime_id).unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    let first = replay.drain_observer(&handle).unwrap();
    assert_eq!(output_message_chunks(&first), vec![b"before-gap".to_vec()]);

    replay.note_source_gap(runtime_id).unwrap();
    replay.fill_observer_queue(&handle).unwrap();
    let second = replay.drain_observer(&handle).unwrap();
    assert_eq!(second.len(), 1);
    assert!(matches!(
        second[0].payload,
        ProtocolPayload::HistoryGap { .. }
    ));
}

#[test]
fn t153_owner_runtime_feeds_eight_observers_from_live_output_and_lifecycle() {
    let home = owner_replay_root("owner-integration");
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let runtime_root = owner_replay_runtime_root();
    #[cfg(windows)]
    let runtime_root = owner_replay_root("windows-runtime-placeholder");

    let profile = owner_replay_shell_profile(&home);
    let mut persistent_owner = start_owner_replay_fixture(&home, &runtime_root, 10);
    let attachment = persistent_owner
        .start_terminal_runtime(
            RuntimeAlias::new("observer-runtime").unwrap(),
            &profile,
            &home,
            TerminalSize { rows: 24, cols: 80 },
            11,
            100,
        )
        .unwrap();
    prime_owner_replay_windows_terminal(&mut persistent_owner, &attachment);
    let runtime_id = attachment.runtime_namespace_id();

    let handles: Vec<_> = (0..8)
        .map(|index| {
            persistent_owner
                .attach_terminal_observer(client(200 + index), runtime_id)
                .unwrap()
        })
        .collect();

    #[cfg(windows)]
    let marker_command = b"echo WINDS_T153_OWNER_REPLAY\r\n".as_slice();
    #[cfg(not(windows))]
    let marker_command = b"printf 'WINDS_T153_OWNER_REPLAY\n'\n".as_slice();
    persistent_owner
        .send_terminal_runtime_input(&attachment, marker_command)
        .unwrap();
    thread::sleep(Duration::from_millis(100));

    let mut sequences = None;
    for handle in &handles {
        persistent_owner
            .fill_terminal_observer_queue(handle)
            .unwrap();
        let messages = persistent_owner.drain_terminal_observer(handle).unwrap();
        assert!(messages.iter().any(|message| {
            matches!(
                &message.payload,
                ProtocolPayload::RuntimeEvent { event }
                    if event.kind == RuntimeLifecycleEventKind::OwnershipEstablished
                        && event.proof_class == LifecycleProofClass::WindsObserved
            )
        }));
        assert!(
            output_message_chunks(&messages).iter().any(|chunk| {
                String::from_utf8_lossy(chunk).contains("WINDS_T153_OWNER_REPLAY")
            })
        );
        let observed: Vec<_> = messages
            .iter()
            .map(|message| message.sequence.get())
            .collect();
        assert!(observed.windows(2).all(|pair| pair[0] < pair[1]));
        match &sequences {
            Some(expected) => assert_eq!(&observed, expected),
            None => sequences = Some(observed),
        }
    }

    persistent_owner
        .close_terminal_runtime(&attachment, 12, 101)
        .unwrap();
    for handle in &handles {
        persistent_owner
            .fill_terminal_observer_queue(handle)
            .unwrap();
        let messages = persistent_owner.drain_terminal_observer(handle).unwrap();
        assert!(messages.iter().any(|message| {
            matches!(
                &message.payload,
                ProtocolPayload::RuntimeEvent { event }
                    if event.kind == RuntimeLifecycleEventKind::RuntimeStopped
                        && event.proof_class == LifecycleProofClass::WindsObserved
            )
        }));
        persistent_owner.detach_terminal_observer(handle).unwrap();
    }

    drop(persistent_owner);
    fs::remove_dir_all(home).unwrap();
    let _ = fs::remove_dir_all(runtime_root);
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
