#![cfg(target_os = "linux")]

use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::terminal::TerminalSize;
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use crate::persistent_runtime::domain::{ClientConnectionId, RuntimeAlias};
use crate::persistent_runtime::owner::{OwnerError, PersistentOwner};
use crate::persistent_runtime::protocol::ProtocolPayload;
use crate::persistent_runtime::replay::{
    MAX_AGGREGATE_REPLAY_BYTES, MAX_OBSERVER_QUEUE_BYTES, MAX_RUNTIME_REPLAY_BYTES,
    MAX_RUNTIME_REPLAY_EVENTS, ObserverHandle,
};
use crate::persistent_runtime::runtime::PersistentTerminalAttachment;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const RECONNECT_CYCLES: usize = 500;
const OBSERVER_ATTACH_SAMPLES: usize = 500;
const CONTROLLER_INPUT_SAMPLES: usize = 1_000;
const RESIZE_SAMPLES: usize = 1_000;
const OBSERVER_COUNT: usize = 8;
const IDLE_RUNTIME_NAMESPACES: usize = 32;
const OUTPUT_MIN_BYTES: u64 = 10 * 1024 * 1024;
const OUTPUT_MIN_LINES: u64 = 100_000;
const OUTPUT_BATCH_LINES: usize = 1_000;
const OUTPUT_BATCH_COUNT: usize = 100;
const OUTPUT_DEADLINE: Duration = Duration::from_secs(60);

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
struct LatencySummary {
    sample_count: usize,
    raw_us: Vec<u64>,
    p50_us: u64,
    p95_us: u64,
    max_us: u64,
}

fn summarize(samples: Vec<Duration>) -> LatencySummary {
    assert!(!samples.is_empty());
    let raw_us: Vec<u64> = samples
        .into_iter()
        .map(|sample| u64::try_from(sample.as_micros()).unwrap_or(u64::MAX))
        .collect();
    let mut ordered = raw_us.clone();
    ordered.sort_unstable();
    LatencySummary {
        sample_count: ordered.len(),
        p50_us: percentile(&ordered, 50),
        p95_us: percentile(&ordered, 95),
        max_us: *ordered.last().expect("non-empty samples were proven"),
        raw_us,
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    let rank = sorted.len().saturating_mul(percentile).saturating_add(99) / 100;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn summary_json(summary: &LatencySummary) -> serde_json::Value {
    json!({
        "sample_count": summary.sample_count,
        "raw_us": summary.raw_us,
        "p50_us": summary.p50_us,
        "p95_us": summary.p95_us,
        "max_us": summary.max_us,
    })
}

fn require_release_profile() {
    let debug_assertions = std::hint::black_box(cfg!(debug_assertions));
    assert!(
        !debug_assertions,
        "T159 performance qualification requires cargo test --release"
    );
}

fn fixture_root(label: &str) -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "winds-t159-{label}-{}-{sequence}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("T159 fixture root must be creatable");
    path.canonicalize()
        .expect("T159 fixture root must canonicalize")
}

fn fixture_runtime_root() -> PathBuf {
    let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("w159r-{}-{sequence}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("T159 runtime root must be creatable");
    path.canonicalize()
        .expect("T159 runtime root must canonicalize")
}

fn fixture_shell_profile(root: &Path) -> ShellProfile {
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: root.to_string_lossy().into_owned(),
        git_common_dir: root.to_string_lossy().into_owned(),
        shell_candidates: vec!["/bin/sh".to_owned()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .expect("T159 shell discovery must succeed")
        .into_iter()
        .find(|profile| profile.executable == "/bin/sh")
        .expect("/bin/sh must qualify on the pinned Ubuntu reference runner")
}

fn start_owner(home: &Path, runtime_root: &Path) -> PersistentOwner {
    let runtime_directory = runtime_root.join("r");
    crate::persistent_runtime::transport::unix::prepare_runtime_directory(&runtime_directory)
        .expect("T159 runtime directory must qualify");
    PersistentOwner::start_for_test(home, &runtime_directory, 10)
        .expect("T159 persistent owner must start")
}

fn client(index: usize) -> ClientConnectionId {
    ClientConnectionId::new(&format!("t159-client-{index:04}"))
        .expect("T159 client id must be valid")
}

fn fd_count() -> usize {
    fs::read_dir("/proc/self/fd")
        .expect("/proc/self/fd must be readable on the pinned Ubuntu runner")
        .count()
}

fn start_runtime(
    owner: &mut PersistentOwner,
    profile: &ShellProfile,
    home: &Path,
    alias: &str,
    now_unix_ms: i64,
    now_monotonic_ms: u64,
) -> PersistentTerminalAttachment {
    owner
        .start_terminal_runtime(
            RuntimeAlias::new(alias).expect("T159 runtime alias must be valid"),
            profile,
            home,
            TerminalSize { rows: 24, cols: 80 },
            now_unix_ms,
            now_monotonic_ms,
        )
        .expect("T159 runtime must start")
}

fn contains_marker(tail: &mut Vec<u8>, chunk: &[u8], marker: &[u8]) -> bool {
    tail.extend_from_slice(chunk);
    let found = tail.windows(marker.len()).any(|window| window == marker);
    if tail.len() > marker.len().saturating_mul(2) {
        let keep = marker.len().saturating_sub(1);
        let start = tail.len().saturating_sub(keep);
        tail.drain(..start);
    }
    found
}

fn high_output_batch_command(batch_index: usize) -> (Vec<u8>, Vec<u8>) {
    let body = "x".repeat(112);
    let marker = format!("WINDS_T159_BATCH_{batch_index:03}_DONE");
    let command = format!(
        "awk 'BEGIN {{ for (i = 0; i < {OUTPUT_BATCH_LINES}; i++) printf \"T159-%03d-%06d-{body}\\n\", {batch_index}, i }}'; printf 'WINDS_T159_BATCH_%03d_DONE\\n' {batch_index}\n"
    )
    .into_bytes();
    (command, marker.into_bytes())
}

#[test]
fn t159_batch_completion_marker_cannot_be_satisfied_by_terminal_input_echo() {
    let (command, marker) = high_output_batch_command(7);
    assert!(
        !command
            .windows(marker.len())
            .any(|window| window == marker.as_slice()),
        "the exact completion marker must be emitted only by completed shell output, never by echoed input"
    );
}

fn drain_fast_observers(
    owner: &mut PersistentOwner,
    handles: &[ObserverHandle],
    fast_output_events: &mut u64,
) {
    for handle in handles.iter().take(OBSERVER_COUNT - 1) {
        owner
            .fill_terminal_observer_queue(handle)
            .expect("fast observer queue fill must remain available");
        let messages = owner
            .drain_terminal_observer(handle)
            .expect("fast observer drain must remain available");
        *fast_output_events += messages
            .iter()
            .filter(|message| matches!(&message.payload, ProtocolPayload::OutputEvent { .. }))
            .count() as u64;
    }
}

#[test]
#[ignore = "T159 release-profile persistent-runtime resource and latency campaign"]
fn t159_release_resource_and_latency_campaign() {
    require_release_profile();

    assert_eq!(MAX_RUNTIME_REPLAY_BYTES, 8 * 1024 * 1024);
    assert_eq!(MAX_RUNTIME_REPLAY_EVENTS, 10_000);
    assert_eq!(MAX_AGGREGATE_REPLAY_BYTES, 64 * 1024 * 1024);
    assert_eq!(MAX_OBSERVER_QUEUE_BYTES, 4 * 1024 * 1024);

    let fd_baseline = fd_count();
    let home = fixture_root("campaign-home");
    let runtime_root = fixture_runtime_root();
    let profile = fixture_shell_profile(&home);
    let mut owner = start_owner(&home, &runtime_root);
    let generation = owner.generation_id();

    let mut attachment = Some(start_runtime(
        &mut owner,
        &profile,
        &home,
        "t159-primary",
        11,
        100,
    ));
    let runtime_id = attachment
        .as_ref()
        .expect("T159 primary attachment must exist")
        .runtime_namespace_id();

    let mut reconnect_samples = Vec::with_capacity(RECONNECT_CYCLES);
    for index in 0..RECONNECT_CYCLES {
        let detached_identity = {
            let detached = attachment
                .take()
                .expect("T159 attachment must exist before detach");
            (
                detached.runtime_namespace_id(),
                detached.owner_generation_id(),
            )
        };
        assert_eq!(detached_identity.0, runtime_id);
        assert_eq!(detached_identity.1, generation);

        let started = Instant::now();
        let reattached = owner
            .reattach_terminal_runtime(
                runtime_id,
                generation,
                20 + index as i64,
                200 + index as u64,
            )
            .expect("exact-generation T159 reattach must succeed");
        let snapshot = owner
            .terminal_runtime_snapshot(&reattached, 20 + index as i64, 200 + index as u64)
            .expect("T159 reattach snapshot must succeed");
        reconnect_samples.push(started.elapsed());
        assert_eq!(snapshot.runtime_namespace_id, runtime_id);
        assert_eq!(snapshot.owner_generation_id, generation);
        attachment = Some(reattached);
    }
    let attachment = attachment.expect("T159 final attachment must exist");
    let reconnect = summarize(reconnect_samples);
    assert!(
        reconnect.p95_us <= 100_000,
        "T159 reattach + snapshot p95 exceeded 100 ms: {} us",
        reconnect.p95_us
    );

    let mut observer_attach_samples = Vec::with_capacity(OBSERVER_ATTACH_SAMPLES);
    for index in 0..OBSERVER_ATTACH_SAMPLES {
        let started = Instant::now();
        let handle = owner
            .attach_terminal_observer(client(1_000 + index), runtime_id)
            .expect("cached observer attach must succeed");
        owner
            .fill_terminal_observer_queue(&handle)
            .expect("cached observer snapshot fill must succeed");
        let _ = owner
            .drain_terminal_observer(&handle)
            .expect("cached observer snapshot drain must succeed");
        owner
            .detach_terminal_observer(&handle)
            .expect("cached observer detach must succeed");
        observer_attach_samples.push(started.elapsed());
    }
    let observer_attach = summarize(observer_attach_samples);
    assert!(
        observer_attach.p95_us <= 100_000,
        "T159 cached observer attach p95 exceeded 100 ms: {} us",
        observer_attach.p95_us
    );

    let controller = client(10);
    let state = owner
        .request_terminal_control(controller.clone(), runtime_id, 1_000, 1_000)
        .expect("T159 controller acquisition must succeed");
    assert_eq!(state.controller_client_id.as_ref(), Some(&controller));

    let mut input_samples = Vec::with_capacity(CONTROLLER_INPUT_SAMPLES);
    for index in 0..CONTROLLER_INPUT_SAMPLES {
        let started = Instant::now();
        owner
            .controller_send_terminal_input(
                &controller,
                runtime_id,
                b" ",
                1_001 + index as i64,
                1_001 + index as u64,
            )
            .expect("T159 controller input dispatch must succeed");
        input_samples.push(started.elapsed());
    }
    let input = summarize(input_samples);
    assert!(
        input.p95_us <= 10_000,
        "T159 controller input dispatch p95 exceeded 10 ms: {} us",
        input.p95_us
    );

    let mut resize_samples = Vec::with_capacity(RESIZE_SAMPLES);
    for index in 0..RESIZE_SAMPLES {
        let size = if index % 2 == 0 {
            TerminalSize { rows: 24, cols: 80 }
        } else {
            TerminalSize {
                rows: 40,
                cols: 120,
            }
        };
        let started = Instant::now();
        owner
            .controller_resize_terminal(
                &controller,
                runtime_id,
                size,
                3_000 + index as i64,
                3_000 + index as u64,
            )
            .expect("T159 controller resize dispatch must succeed");
        resize_samples.push(started.elapsed());
    }
    let resize = summarize(resize_samples);
    assert!(
        resize.p95_us <= 25_000,
        "T159 resize dispatch p95 exceeded 25 ms: {} us",
        resize.p95_us
    );

    let handles: Vec<_> = (0..OBSERVER_COUNT)
        .map(|index| {
            owner
                .attach_terminal_observer(client(20_000 + index), runtime_id)
                .expect("T159 concurrent observer attach must succeed")
        })
        .collect();
    assert_eq!(handles.len(), OBSERVER_COUNT);

    let deadline = Instant::now() + OUTPUT_DEADLINE;
    let mut observed_bytes = 0_u64;
    let mut observed_lines = 0_u64;
    let mut fast_output_events = 0_u64;
    let mut slow_fill_count = 0_u64;
    let mut slow_disconnect_count = 0_u64;
    let mut slow_disconnect_batch = None;

    for batch_index in 0..OUTPUT_BATCH_COUNT {
        let (command, marker) = high_output_batch_command(batch_index);
        owner
            .controller_send_terminal_input(
                &controller,
                runtime_id,
                &command,
                5_000 + batch_index as i64,
                5_000 + batch_index as u64,
            )
            .expect("T159 high-output batch command must dispatch");

        owner
            .controller_resize_terminal(
                &controller,
                runtime_id,
                TerminalSize {
                    rows: 24 + (batch_index % 2) as u16,
                    cols: 80 + (batch_index % 2) as u16,
                },
                6_000 + batch_index as i64,
                6_000 + batch_index as u64,
            )
            .expect("controller resize must remain correct under sustained output pressure");

        let mut tail = Vec::new();
        loop {
            let mut buffer = [0_u8; 64 * 1024];
            let count = owner
                .read_terminal_runtime_output(&attachment, &mut buffer)
                .expect(
                    "T159 direct owner drain must remain complete for each sub-ceiling output batch",
                );
            if count > 0 {
                let chunk = &buffer[..count];
                observed_bytes = observed_bytes.saturating_add(count as u64);
                observed_lines = observed_lines
                    .saturating_add(chunk.iter().filter(|byte| **byte == b'\n').count() as u64);
                if contains_marker(&mut tail, chunk, &marker) {
                    break;
                }
            } else {
                owner
                    .poll_terminal_runtimes(7_000, 7_000)
                    .expect("T159 output wait poll must remain valid");
            }
            assert!(
                Instant::now() < deadline,
                "T159 sustained high-output campaign did not complete inside the 60-second bound"
            );
        }

        drain_fast_observers(&mut owner, &handles, &mut fast_output_events);
        if slow_disconnect_count == 0 {
            let slow_handle = handles
                .last()
                .expect("T159 slow observer handle must exist");
            match owner.fill_terminal_observer_queue(slow_handle) {
                Ok(_) => {
                    slow_fill_count = slow_fill_count.saturating_add(1);
                }
                Err(OwnerError::Runtime(message))
                    if message
                        == "persistent terminal replay failed: replay observer disconnected for slow-client backpressure" =>
                {
                    slow_disconnect_count = slow_disconnect_count.saturating_add(1);
                    slow_disconnect_batch = Some(batch_index);
                }
                Err(error) => {
                    panic!("unexpected T159 slow-observer backpressure result: {error}");
                }
            }
        }
    }

    drain_fast_observers(&mut owner, &handles, &mut fast_output_events);
    assert_eq!(
        slow_disconnect_count, 1,
        "T159 slow observer must disconnect exactly once at the accepted backpressure boundary"
    );
    assert!(
        slow_disconnect_batch.is_some(),
        "T159 slow-observer disconnect batch must be recorded"
    );
    for handle in handles.iter().take(OBSERVER_COUNT - 1) {
        owner
            .fill_terminal_observer_queue(handle)
            .expect("T159 fast observer must remain available after slow-observer disconnect");
        let _ = owner
            .drain_terminal_observer(handle)
            .expect("T159 fast observer must remain drainable after slow-observer disconnect");
        owner
            .detach_terminal_observer(handle)
            .expect("T159 fast observer cleanup must succeed");
    }
    owner
        .detach_terminal_observer(
            handles
                .last()
                .expect("T159 slow observer handle must exist for cleanup"),
        )
        .expect("T159 disconnected slow observer cleanup must succeed");

    assert!(
        observed_bytes >= OUTPUT_MIN_BYTES,
        "T159 observed fewer than 10 MiB: {observed_bytes} bytes"
    );
    assert!(
        observed_lines >= OUTPUT_MIN_LINES,
        "T159 observed fewer than 100,000 logical lines: {observed_lines}"
    );
    assert!(
        fast_output_events > 0,
        "T159 fast observers received no output events during the high-output campaign"
    );
    assert!(
        slow_fill_count > 0,
        "T159 slow-observer pressure was not exercised during the high-output campaign"
    );

    let mut idle_attachments = Vec::with_capacity(IDLE_RUNTIME_NAMESPACES - 1);
    for index in 1..IDLE_RUNTIME_NAMESPACES {
        idle_attachments.push(start_runtime(
            &mut owner,
            &profile,
            &home,
            &format!("t159-idle-{index:02}"),
            8_000 + index as i64,
            8_000 + index as u64,
        ));
    }
    assert_eq!(owner.live_terminal_runtime_count(), IDLE_RUNTIME_NAMESPACES);

    for runtime in idle_attachments {
        owner
            .terminate_terminal_runtime(&runtime, 9_000, 9_000)
            .expect("T159 idle runtime cleanup must succeed");
    }
    owner
        .terminate_terminal_runtime(&attachment, 9_001, 9_001)
        .expect("T159 primary runtime cleanup must succeed");
    assert_eq!(owner.live_terminal_runtime_count(), 0);
    drop(owner);

    thread::sleep(Duration::from_millis(250));
    let fd_after = fd_count();
    assert!(
        fd_after <= fd_baseline.saturating_add(2),
        "T159 leaked file descriptors: baseline={fd_baseline}, after={fd_after}"
    );

    let evidence = json!({
        "schema": "WINDS_SPEC_011_T159_CORE_V1",
        "build_profile": "release",
        "reattach_snapshot": summary_json(&reconnect),
        "cached_observer_attach": summary_json(&observer_attach),
        "controller_input_dispatch": summary_json(&input),
        "resize_dispatch": summary_json(&resize),
        "reconnect_cycles": RECONNECT_CYCLES,
        "concurrent_observers": OBSERVER_COUNT,
        "idle_runtime_namespaces": IDLE_RUNTIME_NAMESPACES,
        "output_observed_bytes": observed_bytes,
        "output_observed_lines": observed_lines,
        "fast_observer_output_events": fast_output_events,
        "slow_observer_fill_count": slow_fill_count,
        "slow_observer_disconnect_count": slow_disconnect_count,
        "slow_observer_disconnect_batch": slow_disconnect_batch,
        "replay_bounds": {
            "per_runtime_bytes": MAX_RUNTIME_REPLAY_BYTES,
            "per_runtime_events": MAX_RUNTIME_REPLAY_EVENTS,
            "aggregate_bytes": MAX_AGGREGATE_REPLAY_BYTES,
            "per_observer_queue_bytes": MAX_OBSERVER_QUEUE_BYTES,
        },
        "fd_baseline": fd_baseline,
        "fd_after_cleanup": fd_after,
        "thresholds_relaxed": false,
        "correctness_or_security_checks_disabled": false,
    });
    println!(
        "T159_CORE_JSON={}",
        serde_json::to_string(&evidence).expect("T159 core evidence must serialize")
    );

    fs::remove_dir_all(home).expect("T159 home cleanup must succeed");
    let _ = fs::remove_dir_all(runtime_root);
}
