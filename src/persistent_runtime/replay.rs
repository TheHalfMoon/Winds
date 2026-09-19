use crate::persistent_runtime::domain::{
    ClientConnectionId, EventSequence, LifecycleProofClass, LocalControlErrorKind,
    OwnerGenerationId, RuntimeLifecycleEvent, RuntimeLifecycleEventKind, RuntimeNamespaceId,
};
use crate::persistent_runtime::protocol::{
    MAX_OUTPUT_EVENT_CHUNK_BYTES, ProtocolMessage, ProtocolPayload, encode_frame,
};
use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;

pub(crate) const MAX_RUNTIME_REPLAY_BYTES: usize = 8 * 1024 * 1024;
pub(crate) const MAX_RUNTIME_REPLAY_EVENTS: usize = 10_000;
pub(crate) const MAX_AGGREGATE_REPLAY_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_OBSERVER_QUEUE_BYTES: usize = 4 * 1024 * 1024;
pub(crate) const MAX_OBSERVERS_PER_RUNTIME: usize = 64;
pub(crate) const MAX_OBSERVERS_PER_OWNER: usize = 256;
const REPLAY_RECORD_ACCOUNTING_OVERHEAD: usize = 64;

pub(crate) type ReplayResult<T> = Result<T, ReplayError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplayError {
    UnknownRuntime,
    UnknownObserver,
    ObserverLimit,
    SequenceExhausted,
    SlowClientBackpressure,
    Protocol(LocalControlErrorKind),
}

impl fmt::Display for ReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRuntime => formatter.write_str("replay runtime is unknown"),
            Self::UnknownObserver => formatter.write_str("replay observer is unknown"),
            Self::ObserverLimit => formatter.write_str("replay observer limit reached"),
            Self::SequenceExhausted => formatter.write_str("replay event sequence exhausted"),
            Self::SlowClientBackpressure => {
                formatter.write_str("replay observer disconnected for slow-client backpressure")
            }
            Self::Protocol(kind) => write!(formatter, "replay protocol message rejected: {kind:?}"),
        }
    }
}

impl Error for ReplayError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReplayPayload {
    Output(Vec<u8>),
    Lifecycle(RuntimeLifecycleEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayRecord {
    sequence: EventSequence,
    payload: ReplayPayload,
    retained_bytes: usize,
}

impl ReplayRecord {
    fn output(sequence: EventSequence, chunk: Vec<u8>) -> Self {
        let retained_bytes = chunk
            .len()
            .saturating_add(REPLAY_RECORD_ACCOUNTING_OVERHEAD);
        Self {
            sequence,
            payload: ReplayPayload::Output(chunk),
            retained_bytes,
        }
    }

    fn lifecycle(event: RuntimeLifecycleEvent) -> ReplayResult<Self> {
        let encoded = serde_json::to_vec(&event)
            .map_err(|_| ReplayError::Protocol(LocalControlErrorKind::MalformedFrame))?;
        Ok(Self {
            sequence: event.sequence,
            payload: ReplayPayload::Lifecycle(event),
            retained_bytes: encoded
                .len()
                .saturating_add(REPLAY_RECORD_ACCOUNTING_OVERHEAD),
        })
    }

    fn is_output(&self) -> bool {
        matches!(self.payload, ReplayPayload::Output(_))
    }

    fn to_message(
        &self,
        connection_id: &ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
        owner_generation_id: OwnerGenerationId,
    ) -> ReplayResult<ProtocolMessage> {
        let payload = match &self.payload {
            ReplayPayload::Output(chunk) => ProtocolPayload::OutputEvent {
                chunk: chunk.clone(),
            },
            ReplayPayload::Lifecycle(event) => ProtocolPayload::RuntimeEvent {
                event: event.clone(),
            },
        };
        ProtocolMessage::new(
            connection_id.clone(),
            self.sequence,
            Some(runtime_namespace_id),
            owner_generation_id,
            None,
            payload,
        )
        .map_err(ReplayError::Protocol)
    }
}

#[derive(Debug)]
struct RuntimeReplay {
    records: VecDeque<ReplayRecord>,
    retained_bytes: usize,
    next_sequence: u64,
    last_dropped_sequence: Option<EventSequence>,
    last_lifecycle_kind: Option<RuntimeLifecycleEventKind>,
}

impl RuntimeReplay {
    fn new() -> Self {
        Self {
            records: VecDeque::new(),
            retained_bytes: 0,
            next_sequence: 1,
            last_dropped_sequence: None,
            last_lifecycle_kind: None,
        }
    }

    fn allocate_sequence(&mut self) -> ReplayResult<EventSequence> {
        let value = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(ReplayError::SequenceExhausted)?;
        EventSequence::new(value).map_err(|_| ReplayError::SequenceExhausted)
    }

    fn earliest_available_value(&self) -> u64 {
        self.records
            .front()
            .map(|record| record.sequence.get())
            .unwrap_or(self.next_sequence)
    }

    fn drop_front(&mut self) -> Option<ReplayRecord> {
        let record = self.records.pop_front()?;
        self.retained_bytes = self.retained_bytes.saturating_sub(record.retained_bytes);
        self.note_dropped(record.sequence);
        Some(record)
    }

    fn note_dropped(&mut self, sequence: EventSequence) {
        if self
            .last_dropped_sequence
            .is_none_or(|current| sequence.get() > current.get())
        {
            self.last_dropped_sequence = Some(sequence);
        }
    }
}

#[derive(Debug, Clone)]
struct QueuedMessage {
    message: ProtocolMessage,
    frame_bytes: usize,
    sequence: EventSequence,
    is_output: bool,
}

#[derive(Debug)]
struct ObserverState {
    cursor: u64,
    queue: VecDeque<QueuedMessage>,
    queued_bytes: usize,
    disconnect_reason: Option<LocalControlErrorKind>,
}

impl ObserverState {
    fn new() -> Self {
        Self {
            cursor: 1,
            queue: VecDeque::new(),
            queued_bytes: 0,
            disconnect_reason: None,
        }
    }

    fn connected(&self) -> bool {
        self.disconnect_reason.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObserverHandle {
    connection_id: ClientConnectionId,
    runtime_namespace_id: RuntimeNamespaceId,
    owner_generation_id: OwnerGenerationId,
}

impl ObserverHandle {
    pub(crate) fn connection_id(&self) -> &ClientConnectionId {
        &self.connection_id
    }

    pub(crate) fn runtime_namespace_id(&self) -> RuntimeNamespaceId {
        self.runtime_namespace_id
    }

    pub(crate) fn owner_generation_id(&self) -> OwnerGenerationId {
        self.owner_generation_id
    }
}

pub(crate) struct ReplayCoordinator {
    owner_generation_id: OwnerGenerationId,
    runtimes: BTreeMap<RuntimeNamespaceId, RuntimeReplay>,
    observers: BTreeMap<(RuntimeNamespaceId, ClientConnectionId), ObserverState>,
    aggregate_retained_bytes: usize,
    fair_eviction_cursor: Option<RuntimeNamespaceId>,
}

impl ReplayCoordinator {
    pub(crate) fn new(owner_generation_id: OwnerGenerationId) -> Self {
        Self {
            owner_generation_id,
            runtimes: BTreeMap::new(),
            observers: BTreeMap::new(),
            aggregate_retained_bytes: 0,
            fair_eviction_cursor: None,
        }
    }

    pub(crate) fn register_runtime(&mut self, runtime_namespace_id: RuntimeNamespaceId) {
        self.runtimes
            .entry(runtime_namespace_id)
            .or_insert_with(RuntimeReplay::new);
    }

    pub(crate) fn unregister_runtime(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<()> {
        let runtime = self
            .runtimes
            .remove(&runtime_namespace_id)
            .ok_or(ReplayError::UnknownRuntime)?;
        self.aggregate_retained_bytes = self
            .aggregate_retained_bytes
            .saturating_sub(runtime.retained_bytes);
        self.observers
            .retain(|(runtime_id, _), _| *runtime_id != runtime_namespace_id);
        if self.fair_eviction_cursor == Some(runtime_namespace_id) {
            self.fair_eviction_cursor = None;
        }
        Ok(())
    }

    pub(crate) fn append_output(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        bytes: &[u8],
    ) -> ReplayResult<()> {
        if !self.runtimes.contains_key(&runtime_namespace_id) {
            return Err(ReplayError::UnknownRuntime);
        }
        if bytes.is_empty() {
            return Ok(());
        }
        for chunk in bytes.chunks(MAX_OUTPUT_EVENT_CHUNK_BYTES) {
            let sequence = self.allocate_sequence(runtime_namespace_id)?;
            let record = ReplayRecord::output(sequence, chunk.to_vec());
            self.push_record(runtime_namespace_id, record)?;
        }
        Ok(())
    }

    pub(crate) fn append_lifecycle(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        kind: RuntimeLifecycleEventKind,
        proof_class: LifecycleProofClass,
        controller_client_id: Option<ClientConnectionId>,
        observed_unix_ms: Option<u64>,
    ) -> ReplayResult<EventSequence> {
        let sequence = self.allocate_sequence(runtime_namespace_id)?;
        let event = RuntimeLifecycleEvent {
            runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
            sequence,
            kind,
            proof_class,
            controller_client_id,
            observed_unix_ms,
        };
        let record = ReplayRecord::lifecycle(event)?;
        self.push_record(runtime_namespace_id, record)?;
        if let Some(runtime) = self.runtimes.get_mut(&runtime_namespace_id) {
            runtime.last_lifecycle_kind = Some(kind);
        }
        Ok(sequence)
    }

    pub(crate) fn append_lifecycle_if_changed(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        kind: RuntimeLifecycleEventKind,
        proof_class: LifecycleProofClass,
        observed_unix_ms: Option<u64>,
    ) -> ReplayResult<Option<EventSequence>> {
        let runtime = self
            .runtimes
            .get(&runtime_namespace_id)
            .ok_or(ReplayError::UnknownRuntime)?;
        if runtime.last_lifecycle_kind == Some(kind) {
            return Ok(None);
        }
        self.append_lifecycle(
            runtime_namespace_id,
            kind,
            proof_class,
            None,
            observed_unix_ms,
        )
        .map(Some)
    }

    pub(crate) fn note_source_gap(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<EventSequence> {
        let sequence = self.allocate_sequence(runtime_namespace_id)?;
        let runtime = self
            .runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(ReplayError::UnknownRuntime)?;
        runtime.note_dropped(sequence);
        Ok(sequence)
    }

    pub(crate) fn attach_observer(
        &mut self,
        connection_id: ClientConnectionId,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<ObserverHandle> {
        if !self.runtimes.contains_key(&runtime_namespace_id) {
            return Err(ReplayError::UnknownRuntime);
        }
        let key = (runtime_namespace_id, connection_id.clone());
        if self.observers.contains_key(&key) {
            return Ok(ObserverHandle {
                connection_id,
                runtime_namespace_id,
                owner_generation_id: self.owner_generation_id,
            });
        }
        let runtime_observers = self
            .observers
            .keys()
            .filter(|(runtime, _)| *runtime == runtime_namespace_id)
            .count();
        if runtime_observers >= MAX_OBSERVERS_PER_RUNTIME
            || self.observers.len() >= MAX_OBSERVERS_PER_OWNER
        {
            return Err(ReplayError::ObserverLimit);
        }
        self.observers.insert(key, ObserverState::new());
        Ok(ObserverHandle {
            connection_id,
            runtime_namespace_id,
            owner_generation_id: self.owner_generation_id,
        })
    }

    pub(crate) fn detach_observer(&mut self, handle: &ObserverHandle) -> ReplayResult<()> {
        self.validate_handle(handle)?;
        self.observers
            .remove(&(handle.runtime_namespace_id, handle.connection_id.clone()))
            .ok_or(ReplayError::UnknownObserver)?;
        Ok(())
    }

    pub(crate) fn fill_observer_queue(&mut self, handle: &ObserverHandle) -> ReplayResult<usize> {
        self.validate_handle(handle)?;
        let runtime = self
            .runtimes
            .get(&handle.runtime_namespace_id)
            .ok_or(ReplayError::UnknownRuntime)?;
        let earliest = runtime.earliest_available_value();
        let last_dropped = runtime.last_dropped_sequence;
        let next_sequence = runtime.next_sequence;
        let records: Vec<_> = runtime.records.iter().cloned().collect();

        let key = (handle.runtime_namespace_id, handle.connection_id.clone());
        let observer = self
            .observers
            .get_mut(&key)
            .ok_or(ReplayError::UnknownObserver)?;
        if !observer.connected() {
            return Err(ReplayError::SlowClientBackpressure);
        }

        if observer.cursor < earliest {
            let last = last_dropped
                .filter(|value| value.get() >= observer.cursor)
                .or_else(|| EventSequence::new(earliest.saturating_sub(1)).ok())
                .ok_or(ReplayError::SequenceExhausted)?;
            enqueue_gap(
                observer,
                &handle.connection_id,
                handle.runtime_namespace_id,
                self.owner_generation_id,
                earliest,
                last,
            )?;
            observer.cursor = earliest;
        }

        let mut added = 0_usize;
        let mut index = 0_usize;
        while index < records.len() {
            let record = &records[index];
            if record.sequence.get() < observer.cursor {
                index += 1;
                continue;
            }
            if record.sequence.get() > observer.cursor {
                let last = EventSequence::new(record.sequence.get().saturating_sub(1))
                    .map_err(|_| ReplayError::SequenceExhausted)?;
                enqueue_gap(
                    observer,
                    &handle.connection_id,
                    handle.runtime_namespace_id,
                    self.owner_generation_id,
                    record.sequence.get(),
                    last,
                )?;
                observer.cursor = record.sequence.get();
            }

            let message = record.to_message(
                &handle.connection_id,
                handle.runtime_namespace_id,
                self.owner_generation_id,
            )?;
            let frame_bytes = encode_frame(&message).map_err(ReplayError::Protocol)?.len();

            if observer.queued_bytes.saturating_add(frame_bytes) <= MAX_OBSERVER_QUEUE_BYTES {
                observer.queue.push_back(QueuedMessage {
                    message,
                    frame_bytes,
                    sequence: record.sequence,
                    is_output: record.is_output(),
                });
                observer.queued_bytes = observer.queued_bytes.saturating_add(frame_bytes);
                observer.cursor = record.sequence.get().saturating_add(1);
                added = added.saturating_add(1);
                index += 1;
                continue;
            }

            if record.is_output() {
                let next_lifecycle = records[index + 1..]
                    .iter()
                    .find(|candidate| !candidate.is_output());
                let Some(lifecycle) = next_lifecycle else {
                    break;
                };
                prioritize_lifecycle(
                    observer,
                    &handle.connection_id,
                    handle.runtime_namespace_id,
                    self.owner_generation_id,
                    lifecycle,
                )?;
                observer.cursor = lifecycle.sequence.get().saturating_add(1);
                added = added.saturating_add(2);
                index = records
                    .iter()
                    .position(|candidate| candidate.sequence == lifecycle.sequence)
                    .map(|value| value + 1)
                    .unwrap_or(records.len());
                continue;
            }

            disconnect_slow_observer(observer);
            return Err(ReplayError::SlowClientBackpressure);
        }

        if let Some(last) = last_dropped.filter(|value| value.get() >= observer.cursor) {
            let first_available = last
                .get()
                .checked_add(1)
                .ok_or(ReplayError::SequenceExhausted)?;
            if first_available > next_sequence {
                return Err(ReplayError::SequenceExhausted);
            }
            enqueue_gap(
                observer,
                &handle.connection_id,
                handle.runtime_namespace_id,
                self.owner_generation_id,
                first_available,
                last,
            )?;
            observer.cursor = first_available;
            added = added.saturating_add(1);
        }

        Ok(added)
    }

    pub(crate) fn drain_observer(
        &mut self,
        handle: &ObserverHandle,
    ) -> ReplayResult<Vec<ProtocolMessage>> {
        self.validate_handle(handle)?;
        let key = (handle.runtime_namespace_id, handle.connection_id.clone());
        let observer = self
            .observers
            .get_mut(&key)
            .ok_or(ReplayError::UnknownObserver)?;
        if !observer.connected() {
            return Err(ReplayError::SlowClientBackpressure);
        }
        let mut messages = Vec::with_capacity(observer.queue.len());
        while let Some(queued) = observer.queue.pop_front() {
            observer.queued_bytes = observer.queued_bytes.saturating_sub(queued.frame_bytes);
            messages.push(queued.message);
        }
        Ok(messages)
    }

    pub(crate) fn observer_queued_bytes(&self, handle: &ObserverHandle) -> ReplayResult<usize> {
        self.validate_handle(handle)?;
        self.observers
            .get(&(handle.runtime_namespace_id, handle.connection_id.clone()))
            .map(|observer| observer.queued_bytes)
            .ok_or(ReplayError::UnknownObserver)
    }

    pub(crate) fn observer_disconnect_reason(
        &self,
        handle: &ObserverHandle,
    ) -> ReplayResult<Option<LocalControlErrorKind>> {
        self.validate_handle_generation(handle)?;
        self.observers
            .get(&(handle.runtime_namespace_id, handle.connection_id.clone()))
            .map(|observer| observer.disconnect_reason)
            .ok_or(ReplayError::UnknownObserver)
    }

    pub(crate) fn aggregate_retained_bytes(&self) -> usize {
        self.aggregate_retained_bytes
    }

    pub(crate) fn runtime_retained_bytes(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<usize> {
        self.runtimes
            .get(&runtime_namespace_id)
            .map(|runtime| runtime.retained_bytes)
            .ok_or(ReplayError::UnknownRuntime)
    }

    pub(crate) fn runtime_retained_events(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<usize> {
        self.runtimes
            .get(&runtime_namespace_id)
            .map(|runtime| runtime.records.len())
            .ok_or(ReplayError::UnknownRuntime)
    }

    pub(crate) fn runtime_last_dropped_sequence(
        &self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<Option<EventSequence>> {
        self.runtimes
            .get(&runtime_namespace_id)
            .map(|runtime| runtime.last_dropped_sequence)
            .ok_or(ReplayError::UnknownRuntime)
    }

    fn allocate_sequence(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<EventSequence> {
        self.runtimes
            .get_mut(&runtime_namespace_id)
            .ok_or(ReplayError::UnknownRuntime)?
            .allocate_sequence()
    }

    fn push_record(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
        record: ReplayRecord,
    ) -> ReplayResult<()> {
        let bytes = record.retained_bytes;
        {
            let runtime = self
                .runtimes
                .get_mut(&runtime_namespace_id)
                .ok_or(ReplayError::UnknownRuntime)?;
            runtime.records.push_back(record);
            runtime.retained_bytes = runtime.retained_bytes.saturating_add(bytes);
        }
        self.aggregate_retained_bytes = self.aggregate_retained_bytes.saturating_add(bytes);
        self.enforce_runtime_bounds(runtime_namespace_id)?;
        self.enforce_aggregate_bound();
        Ok(())
    }

    fn enforce_runtime_bounds(
        &mut self,
        runtime_namespace_id: RuntimeNamespaceId,
    ) -> ReplayResult<()> {
        loop {
            let over = {
                let runtime = self
                    .runtimes
                    .get(&runtime_namespace_id)
                    .ok_or(ReplayError::UnknownRuntime)?;
                runtime.retained_bytes > MAX_RUNTIME_REPLAY_BYTES
                    || runtime.records.len() > MAX_RUNTIME_REPLAY_EVENTS
            };
            if !over {
                return Ok(());
            }
            self.evict_one(runtime_namespace_id);
        }
    }

    fn enforce_aggregate_bound(&mut self) {
        while self.aggregate_retained_bytes > MAX_AGGREGATE_REPLAY_BYTES {
            let Some(runtime_namespace_id) = self.next_fair_evictable_runtime() else {
                self.aggregate_retained_bytes = 0;
                return;
            };
            self.evict_one(runtime_namespace_id);
            self.fair_eviction_cursor = Some(runtime_namespace_id);
        }
    }

    fn next_fair_evictable_runtime(&self) -> Option<RuntimeNamespaceId> {
        let maximum_bytes = self
            .runtimes
            .values()
            .filter(|runtime| !runtime.records.is_empty())
            .map(|runtime| runtime.retained_bytes)
            .max()?;
        let candidates: Vec<_> = self
            .runtimes
            .iter()
            .filter_map(|(runtime_id, runtime)| {
                (!runtime.records.is_empty() && runtime.retained_bytes == maximum_bytes)
                    .then_some(*runtime_id)
            })
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let start = self
            .fair_eviction_cursor
            .and_then(|cursor| candidates.iter().position(|key| *key == cursor))
            .map(|position| (position + 1) % candidates.len())
            .unwrap_or(0);
        candidates
            .get(start)
            .copied()
            .or_else(|| candidates.first().copied())
    }

    fn evict_one(&mut self, runtime_namespace_id: RuntimeNamespaceId) {
        if let Some(runtime) = self.runtimes.get_mut(&runtime_namespace_id)
            && let Some(record) = runtime.drop_front()
        {
            self.aggregate_retained_bytes = self
                .aggregate_retained_bytes
                .saturating_sub(record.retained_bytes);
        }
    }

    fn validate_handle_generation(&self, handle: &ObserverHandle) -> ReplayResult<()> {
        if handle.owner_generation_id != self.owner_generation_id {
            return Err(ReplayError::Protocol(
                LocalControlErrorKind::StaleOwnerGeneration,
            ));
        }
        Ok(())
    }

    fn validate_handle(&self, handle: &ObserverHandle) -> ReplayResult<()> {
        self.validate_handle_generation(handle)?;
        if !self
            .observers
            .contains_key(&(handle.runtime_namespace_id, handle.connection_id.clone()))
        {
            return Err(ReplayError::UnknownObserver);
        }
        Ok(())
    }
}

fn enqueue_gap(
    observer: &mut ObserverState,
    connection_id: &ClientConnectionId,
    runtime_namespace_id: RuntimeNamespaceId,
    owner_generation_id: OwnerGenerationId,
    first_available_value: u64,
    last_dropped_sequence: EventSequence,
) -> ReplayResult<()> {
    let first_available_sequence =
        EventSequence::new(first_available_value).map_err(|_| ReplayError::SequenceExhausted)?;
    if first_available_sequence.get() <= last_dropped_sequence.get() {
        return Err(ReplayError::Protocol(LocalControlErrorKind::MalformedFrame));
    }
    let message = ProtocolMessage::new(
        connection_id.clone(),
        last_dropped_sequence,
        Some(runtime_namespace_id),
        owner_generation_id,
        None,
        ProtocolPayload::HistoryGap {
            first_available_sequence,
            last_dropped_sequence,
        },
    )
    .map_err(ReplayError::Protocol)?;
    let frame_bytes = encode_frame(&message).map_err(ReplayError::Protocol)?.len();
    make_room_for_priority(observer, frame_bytes)?;
    observer.queue.push_back(QueuedMessage {
        message,
        frame_bytes,
        sequence: last_dropped_sequence,
        is_output: false,
    });
    observer.queued_bytes = observer.queued_bytes.saturating_add(frame_bytes);
    Ok(())
}

fn prioritize_lifecycle(
    observer: &mut ObserverState,
    connection_id: &ClientConnectionId,
    runtime_namespace_id: RuntimeNamespaceId,
    owner_generation_id: OwnerGenerationId,
    lifecycle: &ReplayRecord,
) -> ReplayResult<()> {
    let lifecycle_message =
        lifecycle.to_message(connection_id, runtime_namespace_id, owner_generation_id)?;
    let lifecycle_bytes = encode_frame(&lifecycle_message)
        .map_err(ReplayError::Protocol)?
        .len();

    let mut dropped_from_queue = None;
    while observer.queued_bytes.saturating_add(lifecycle_bytes) > MAX_OBSERVER_QUEUE_BYTES {
        let Some(back) = observer.queue.back() else {
            disconnect_slow_observer(observer);
            return Err(ReplayError::SlowClientBackpressure);
        };
        if !back.is_output {
            disconnect_slow_observer(observer);
            return Err(ReplayError::SlowClientBackpressure);
        }
        let removed = observer
            .queue
            .pop_back()
            .expect("observer queue back was checked");
        observer.queued_bytes = observer.queued_bytes.saturating_sub(removed.frame_bytes);
        dropped_from_queue = Some(
            dropped_from_queue.map_or(removed.sequence.get(), |current: u64| {
                current.min(removed.sequence.get())
            }),
        );
    }

    let first_dropped = dropped_from_queue.unwrap_or(observer.cursor);
    let last_dropped_value = lifecycle.sequence.get().saturating_sub(1);
    if first_dropped <= last_dropped_value && last_dropped_value > 0 {
        let last_dropped =
            EventSequence::new(last_dropped_value).map_err(|_| ReplayError::SequenceExhausted)?;
        let gap_message = ProtocolMessage::new(
            connection_id.clone(),
            last_dropped,
            Some(runtime_namespace_id),
            owner_generation_id,
            None,
            ProtocolPayload::HistoryGap {
                first_available_sequence: lifecycle.sequence,
                last_dropped_sequence: last_dropped,
            },
        )
        .map_err(ReplayError::Protocol)?;
        let gap_bytes = encode_frame(&gap_message)
            .map_err(ReplayError::Protocol)?
            .len();
        make_room_for_priority(observer, gap_bytes.saturating_add(lifecycle_bytes))?;
        observer.queue.push_back(QueuedMessage {
            message: gap_message,
            frame_bytes: gap_bytes,
            sequence: last_dropped,
            is_output: false,
        });
        observer.queued_bytes = observer.queued_bytes.saturating_add(gap_bytes);
    }

    make_room_for_priority(observer, lifecycle_bytes)?;
    observer.queue.push_back(QueuedMessage {
        message: lifecycle_message,
        frame_bytes: lifecycle_bytes,
        sequence: lifecycle.sequence,
        is_output: false,
    });
    observer.queued_bytes = observer.queued_bytes.saturating_add(lifecycle_bytes);
    Ok(())
}

fn make_room_for_priority(observer: &mut ObserverState, required_bytes: usize) -> ReplayResult<()> {
    if required_bytes > MAX_OBSERVER_QUEUE_BYTES {
        disconnect_slow_observer(observer);
        return Err(ReplayError::SlowClientBackpressure);
    }
    while observer.queued_bytes.saturating_add(required_bytes) > MAX_OBSERVER_QUEUE_BYTES {
        let Some(back) = observer.queue.back() else {
            disconnect_slow_observer(observer);
            return Err(ReplayError::SlowClientBackpressure);
        };
        if !back.is_output {
            disconnect_slow_observer(observer);
            return Err(ReplayError::SlowClientBackpressure);
        }
        let removed = observer
            .queue
            .pop_back()
            .expect("observer queue back was checked");
        observer.queued_bytes = observer.queued_bytes.saturating_sub(removed.frame_bytes);
    }
    Ok(())
}

fn disconnect_slow_observer(observer: &mut ObserverState) {
    observer.queue.clear();
    observer.queued_bytes = 0;
    observer.disconnect_reason = Some(LocalControlErrorKind::SlowClientBackpressure);
}

#[cfg(test)]
#[path = "../t153_persistent_replay_tests.rs"]
mod t153_persistent_replay_tests;
