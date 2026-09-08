#![cfg(target_os = "linux")]

use super::interaction::{InteractionContext, search_terminal_transcript};
use super::output::WorkbenchOutput;
use super::screen::WorkbenchScreen;
use super::terminal::WorkbenchTerminals;
use super::{PaneId, PaneLifecycleView, PaneSize, WorkbenchState};
use crate::git::shell_profiles::{ShellProfile, discover_native_shell_profiles};
use crate::git::workspace_inventory::WorkspaceEnvironmentInventory;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const DISPATCH_ITERATIONS: usize = 1_000;
const TOPOLOGY_ITERATIONS: usize = 1_000;
const HISTORY_LINES: usize = 100_000;
const HISTORY_SEARCHES: usize = 200;
const OUTPUT_LINES: usize = 100_001;
const OUTPUT_BODY_BYTES: usize = 111;
const RESIZE_ITERATIONS: usize = 1_000;
const IDLE_PANES: usize = 10;
const IDLE_DURATION: Duration = Duration::from_secs(60);
const HIGH_VOLUME_DEADLINE: Duration = Duration::from_secs(60);
const HIGH_VOLUME_DONE_MARKER: &str = "WINDS_T097_HIGH_VOLUME_DONE";

#[derive(Debug, Clone, Copy)]
struct Summary {
    sample_count: usize,
    p50_us: u64,
    p95_us: u64,
    max_us: u64,
}

#[derive(Debug, Clone, Copy)]
struct HighVolumeSummary {
    navigation: Summary,
    processed_lines: u64,
    processed_bytes: u64,
    retained_lines: usize,
    retained_bytes: usize,
    evicted_lines: u64,
    evicted_bytes: u64,
    truncated: bool,
    lifecycle_live_after_campaign: bool,
    owned_terminal_after_campaign: bool,
}

fn summarize(samples: &[Duration]) -> Summary {
    assert!(!samples.is_empty());
    let mut micros: Vec<u64> = samples
        .iter()
        .map(|sample| u64::try_from(sample.as_micros()).unwrap_or(u64::MAX))
        .collect();
    micros.sort_unstable();
    Summary {
        sample_count: micros.len(),
        p50_us: percentile(&micros, 50),
        p95_us: percentile(&micros, 95),
        max_us: *micros.last().expect("non-empty samples were just proven"),
    }
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    let rank = sorted.len().saturating_mul(percentile).saturating_add(99) / 100;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn canonical_cwd() -> PathBuf {
    std::env::current_dir()
        .expect("benchmark cwd must exist")
        .canonicalize()
        .expect("benchmark cwd must be canonicalizable")
}

fn require_release_profile() {
    let debug_assertions = std::hint::black_box(cfg!(debug_assertions));
    assert!(
        !debug_assertions,
        "T097 performance qualification requires cargo test --release"
    );
}

fn fixture_shell_profile(cwd: &Path) -> ShellProfile {
    let shell = "/bin/sh";
    let inventory = WorkspaceEnvironmentInventory {
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        canonical_worktree_root: cwd.to_string_lossy().into_owned(),
        git_common_dir: cwd.to_string_lossy().into_owned(),
        shell_candidates: vec![shell.to_owned()],
        detected_manifests: Vec::new(),
    };
    discover_native_shell_profiles(&inventory)
        .expect("fixture shell discovery must succeed")
        .into_iter()
        .find(|profile| profile.executable == shell)
        .expect("/bin/sh must qualify on the pinned Ubuntu reference runner")
}

fn close_all(terminals: &mut WorkbenchTerminals, state: &mut WorkbenchState, pane_ids: &[PaneId]) {
    for pane_id in pane_ids.iter().copied() {
        terminals
            .close_pane(state, pane_id)
            .expect("benchmark fixture terminal cleanup must remain proven");
    }
}

#[test]
fn t097_nonblocking_output_pump_drains_live_pane_into_its_screen() {
    let cwd = canonical_cwd();
    let profile = fixture_shell_profile(&cwd);
    let mut state = WorkbenchState::new();
    let pane_id = state.create_pane(
        "output-pump",
        Some("t097-workspace".to_owned()),
        None,
        PaneSize::new(80, 24),
    );
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane_id, &profile, &cwd)
        .expect("output-pump fixture shell must start");
    let mut output = WorkbenchOutput::new();
    output
        .attach_live_pane(&mut terminals, &mut state, pane_id)
        .expect("live pane must transfer its output reader to the bounded pump");
    terminals
        .dispatch_selected_input(&mut state, b"printf 'WINDS_T097_OUTPUT_PUMP\\n'\n")
        .expect("fixture marker command must dispatch to the selected owned pane");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        output
            .drain_tick(&mut terminals, &mut state)
            .expect("output pump must drain without blocking the host tick");
        if output
            .screen_contents(pane_id)
            .is_some_and(|contents| contents.contains("WINDS_T097_OUTPUT_PUMP"))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "T097 output pump did not project the terminal marker inside the fixture deadline"
        );
        thread::sleep(Duration::from_millis(10));
    }

    close_all(&mut terminals, &mut state, &[pane_id]);
}

#[test]
#[ignore = "T097 release-profile benchmark campaign; run explicitly in t097-performance"]
fn t097_release_benchmark_campaign() {
    require_release_profile();

    let cwd = canonical_cwd();
    let profile = fixture_shell_profile(&cwd);

    let dispatch = benchmark_dispatch(&cwd, &profile);
    assert!(
        dispatch.p95_us <= 16_000,
        "FR-046 p95 exceeded 16 ms: {} us",
        dispatch.p95_us
    );

    let topology = benchmark_topology();
    assert!(
        topology.p95_us <= 16_000,
        "FR-047 p95 exceeded 16 ms: {} us",
        topology.p95_us
    );

    let history_search = benchmark_history_search();
    assert!(
        history_search.p95_us <= 100_000,
        "FR-048 p95 exceeded 100 ms: {} us",
        history_search.p95_us
    );

    let high_volume = benchmark_large_output_and_navigation(&cwd, &profile);
    assert!(
        high_volume.navigation.p95_us <= 100_000,
        "FR-049 navigation p95 exceeded 100 ms: {} us",
        high_volume.navigation.p95_us
    );
    assert!(
        high_volume.processed_lines >= u64::try_from(OUTPUT_LINES).unwrap(),
        "FR-049 processed fewer than the required logical lines"
    );
    assert!(
        high_volume.processed_bytes >= 10 * 1024 * 1024,
        "FR-049 processed less than the required 10 MiB terminal payload"
    );
    assert!(
        high_volume.lifecycle_live_after_campaign && high_volume.owned_terminal_after_campaign,
        "FR-049 high-volume campaign corrupted live terminal lifecycle or ownership"
    );
    assert!(
        high_volume.retained_lines <= 100_000,
        "FR-050 line bound exceeded"
    );
    assert!(
        high_volume.retained_bytes <= 32 * 1024 * 1024,
        "FR-050 payload bound exceeded"
    );
    assert!(high_volume.truncated, "FR-050 eviction state must be visible");
    assert!(
        high_volume.evicted_lines > 0 || high_volume.evicted_bytes > 0,
        "FR-050 over-bound campaign must record eviction"
    );

    let (resize, final_size_correct) = benchmark_resize(&cwd, &profile);
    assert!(
        resize <= Duration::from_secs(10),
        "FR-052 1000 resize requests exceeded ten seconds: {resize:?}"
    );
    assert!(
        final_size_correct,
        "FR-052 final live terminal size did not match the final accepted size"
    );

    println!(
        "T097_CORE_JSON={}",
        serde_json::to_string(&json!({
            "fr046_dispatch": summary_json(dispatch),
            "fr047_topology": summary_json(topology),
            "fr048_history_search": summary_json(history_search),
            "fr049_navigation": summary_json(high_volume.navigation),
            "fr049_high_volume": {
                "processed_lines": high_volume.processed_lines,
                "processed_bytes": high_volume.processed_bytes,
                "source": "live_fixture_shell_via_owned_pty_and_bounded_workbench_output_pump",
                "lifecycle_live_after_campaign": high_volume.lifecycle_live_after_campaign,
                "owned_terminal_after_campaign": high_volume.owned_terminal_after_campaign,
            },
            "fr050_retention": {
                "retained_lines": high_volume.retained_lines,
                "retained_bytes": high_volume.retained_bytes,
                "evicted_lines": high_volume.evicted_lines,
                "evicted_bytes": high_volume.evicted_bytes,
                "truncated": high_volume.truncated,
                "line_limit": 100_000,
                "byte_limit": 32 * 1024 * 1024,
            },
            "fr052_resize": {
                "sample_count": RESIZE_ITERATIONS,
                "total_us": u64::try_from(resize.as_micros()).unwrap_or(u64::MAX),
                "final_size_correct": final_size_correct,
                "verification_source": "owned TerminalSession::current_size",
            },
        }))
        .expect("T097 core evidence must serialize")
    );
}

fn benchmark_dispatch(cwd: &Path, profile: &ShellProfile) -> Summary {
    let mut state = WorkbenchState::new();
    let pane_id = state.create_pane(
        "dispatch",
        Some("t097-workspace".to_owned()),
        None,
        PaneSize::new(80, 24),
    );
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, pane_id, profile, cwd)
        .expect("dispatch fixture shell must start");

    for _ in 0..100 {
        terminals
            .dispatch_selected_input(&mut state, b":\n")
            .expect("dispatch warmup must succeed");
    }

    let mut samples = Vec::with_capacity(DISPATCH_ITERATIONS);
    for _ in 0..DISPATCH_ITERATIONS {
        let start = Instant::now();
        terminals
            .dispatch_selected_input(&mut state, b":\n")
            .expect("dispatch benchmark must preserve exact live ownership");
        samples.push(start.elapsed());
    }

    close_all(&mut terminals, &mut state, &[pane_id]);
    summarize(&samples)
}

fn benchmark_topology() -> Summary {
    let mut state = WorkbenchState::new();
    let mut pane_ids = Vec::with_capacity(50);
    for index in 0..50 {
        pane_ids.push(state.create_pane(
            format!("pane-{index:02}"),
            Some("t097-workspace".to_owned()),
            None,
            PaneSize::new(80, 24),
        ));
    }

    let mut samples = Vec::with_capacity(TOPOLOGY_ITERATIONS);
    for index in 0..TOPOLOGY_ITERATIONS {
        let pane_id = pane_ids[index % pane_ids.len()];
        let start = Instant::now();
        assert!(state.focus_pane(pane_id));
        assert!(state.resize_pane(
            pane_id,
            PaneSize::new(80 + u16::try_from(index % 8).unwrap(), 24)
        ));
        samples.push(start.elapsed());
    }
    summarize(&samples)
}

fn benchmark_history_search() -> Summary {
    let mut screen = WorkbenchScreen::new(PaneSize::new(80, 24))
        .expect("history benchmark screen must initialize");
    let mut payload = Vec::with_capacity(HISTORY_LINES * 28);
    for index in 0..HISTORY_LINES {
        payload.extend_from_slice(format!("entry-{index:06} marker-{index:06}\n").as_bytes());
    }
    screen.process_observed_bytes(&payload);
    let snapshot = screen.transcript_snapshot();
    assert_eq!(snapshot.lines.len(), HISTORY_LINES);
    let context = InteractionContext::new(
        None,
        Some("t097-workspace".to_owned()),
        Some("t097-session".to_owned()),
    );

    for index in 0..10 {
        let query = format!("entry-{:06}", index * 499);
        let result = search_terminal_transcript(&snapshot, &context, &query);
        assert!(!result.matches.is_empty());
    }

    let mut samples = Vec::with_capacity(HISTORY_SEARCHES);
    for index in 0..HISTORY_SEARCHES {
        let query = format!("entry-{:06}", index * 499);
        let start = Instant::now();
        let result = search_terminal_transcript(&snapshot, &context, &query);
        samples.push(start.elapsed());
        assert!(!result.matches.is_empty());
    }
    summarize(&samples)
}

fn benchmark_large_output_and_navigation(cwd: &Path, profile: &ShellProfile) -> HighVolumeSummary {
    let mut state = WorkbenchState::new();
    let mut pane_ids = Vec::with_capacity(50);
    for index in 0..50 {
        pane_ids.push(state.create_pane(
            format!("output-pane-{index:02}"),
            Some("t097-workspace".to_owned()),
            None,
            PaneSize::new(80, 24),
        ));
    }
    let output_pane = pane_ids[0];
    let mut terminals = WorkbenchTerminals::new();
    terminals
        .start_native(&mut state, output_pane, profile, cwd)
        .expect("high-volume fixture shell must start");
    let mut output = WorkbenchOutput::new();
    output
        .attach_live_pane(&mut terminals, &mut state, output_pane)
        .expect("high-volume live pane must attach the bounded output pump");

    let line_body = format!("output {}", "x".repeat(104));
    assert_eq!(line_body.len(), OUTPUT_BODY_BYTES);
    let payload_bytes = (OUTPUT_BODY_BYTES + 1) * OUTPUT_LINES;
    assert!(payload_bytes >= 10 * 1024 * 1024);
    let command = format!(
        "i=0; while [ \"$i\" -lt {OUTPUT_LINES} ]; do printf '%s\\n' '{line_body}'; i=$((i + 1)); done; printf '{HIGH_VOLUME_DONE_MARKER}\\n'\n"
    );
    terminals
        .dispatch_selected_input(&mut state, command.as_bytes())
        .expect("high-volume fixture command must dispatch to the exact owned pane");

    let deadline = Instant::now() + HIGH_VOLUME_DEADLINE;
    let mut samples = Vec::with_capacity(TOPOLOGY_ITERATIONS);
    let mut output_observed = false;
    loop {
        output
            .drain_tick(&mut terminals, &mut state)
            .expect("high-volume output pump must drain without lifecycle corruption");
        let snapshot = output
            .transcript_snapshot(output_pane)
            .expect("high-volume pane must retain a bounded transcript");
        output_observed |= snapshot.retained_bytes > 0 || snapshot.evicted_bytes > 0;
        let marker_seen = output
            .screen_contents(output_pane)
            .is_some_and(|contents| contents.contains(HIGH_VOLUME_DONE_MARKER));

        if output_observed && !marker_seen {
            for _ in 0..25 {
                if samples.len() == TOPOLOGY_ITERATIONS {
                    break;
                }
                let other = pane_ids[1 + (samples.len() % (pane_ids.len() - 1))];
                assert!(state.focus_pane(other));
                let start = Instant::now();
                assert!(state.focus_pane(output_pane));
                samples.push(start.elapsed());
            }
        }

        if marker_seen {
            assert_eq!(
                samples.len(),
                TOPOLOGY_ITERATIONS,
                "FR-049 output campaign completed before 1000 in-campaign navigation samples"
            );
            break;
        }
        assert!(
            Instant::now() < deadline,
            "FR-049 live high-volume terminal campaign exceeded its 60-second fixture deadline"
        );
        thread::yield_now();
    }

    let snapshot = output
        .transcript_snapshot(output_pane)
        .expect("high-volume pane must retain its final bounded transcript");
    let processed_lines = u64::try_from(snapshot.lines.len()).unwrap_or(u64::MAX)
        .saturating_add(snapshot.evicted_lines);
    let processed_bytes = u64::try_from(snapshot.retained_bytes)
        .unwrap_or(u64::MAX)
        .saturating_add(snapshot.evicted_bytes);
    let lifecycle_live_after_campaign = state
        .pane(output_pane)
        .is_some_and(|pane| pane.lifecycle == PaneLifecycleView::Live);
    let owned_terminal_after_campaign = terminals.has_owned_terminal(output_pane);

    let result = HighVolumeSummary {
        navigation: summarize(&samples),
        processed_lines,
        processed_bytes,
        retained_lines: snapshot.lines.len(),
        retained_bytes: snapshot.retained_bytes,
        evicted_lines: snapshot.evicted_lines,
        evicted_bytes: snapshot.evicted_bytes,
        truncated: snapshot.truncated,
        lifecycle_live_after_campaign,
        owned_terminal_after_campaign,
    };
    close_all(&mut terminals, &mut state, &[output_pane]);
    result
}

fn benchmark_resize(cwd: &Path, profile: &ShellProfile) -> (Duration, bool) {
    let mut state = WorkbenchState::new();
    let mut terminals = WorkbenchTerminals::new();
    let mut pane_ids = Vec::with_capacity(10);
    let mut final_sizes = [PaneSize::new(80, 24); 10];

    for index in 0..10 {
        let pane_id = state.create_pane(
            format!("resize-{index}"),
            Some("t097-workspace".to_owned()),
            None,
            PaneSize::new(80, 24),
        );
        terminals
            .start_native(&mut state, pane_id, profile, cwd)
            .expect("resize fixture shell must start");
        pane_ids.push(pane_id);
    }

    let start = Instant::now();
    for index in 0..RESIZE_ITERATIONS {
        let slot = index % pane_ids.len();
        let size = PaneSize::new(
            80 + u16::try_from(index % 20).unwrap(),
            24 + u16::try_from(index % 5).unwrap(),
        );
        terminals
            .resize(&mut state, pane_ids[slot], size)
            .expect("resize benchmark must preserve owned terminal truth");
        final_sizes[slot] = size;
    }
    let elapsed = start.elapsed();

    let final_size_correct =
        pane_ids
            .iter()
            .copied()
            .zip(final_sizes)
            .all(|(pane_id, expected)| {
                state
                    .pane(pane_id)
                    .is_some_and(|pane| pane.size == expected)
                    && terminals
                        .current_size(pane_id)
                        .is_ok_and(|actual| actual == expected)
            });
    close_all(&mut terminals, &mut state, &pane_ids);
    (elapsed, final_size_correct)
}

#[test]
#[ignore = "T097 sixty-second idle resource campaign; run explicitly in t097-performance"]
fn t097_idle_resource_campaign() {
    require_release_profile();

    let cwd = canonical_cwd();
    let profile = fixture_shell_profile(&cwd);
    let page_size = required_env_u64("WINDS_T097_PAGE_SIZE");
    let clock_ticks = required_env_u64("WINDS_T097_CLK_TCK");
    let baseline_rss = process_rss_bytes(page_size);

    let mut state = WorkbenchState::new();
    let mut terminals = WorkbenchTerminals::new();
    let mut output = WorkbenchOutput::new();
    let mut pane_ids = Vec::with_capacity(IDLE_PANES);
    for index in 0..IDLE_PANES {
        let pane_id = state.create_pane(
            format!("idle-{index}"),
            Some("t097-workspace".to_owned()),
            None,
            PaneSize::new(80, 24),
        );
        terminals
            .start_native(&mut state, pane_id, &profile, &cwd)
            .expect("idle fixture shell must start");
        output
            .attach_live_pane(&mut terminals, &mut state, pane_id)
            .expect("idle live pane must attach its bounded output pump");
        pane_ids.push(pane_id);
    }
    let editor = super::terminal::input::WorkbenchShellEditor::new();
    let backend = TestBackend::new(160, 50);
    let mut host_terminal = Terminal::new(backend).expect("idle render backend must initialize");

    let settle = Instant::now();
    while settle.elapsed() < Duration::from_secs(1) {
        output
            .drain_tick(&mut terminals, &mut state)
            .expect("idle settling output must drain");
        host_terminal
            .draw(|frame| super::render_workbench(frame, &state, &editor, &output))
            .expect("idle settling render must succeed");
        thread::sleep(Duration::from_millis(10));
    }

    let start_ticks = process_cpu_ticks();
    let started = Instant::now();
    let mut max_rss = process_rss_bytes(page_size);
    while started.elapsed() < IDLE_DURATION {
        output
            .drain_tick(&mut terminals, &mut state)
            .expect("idle workbench output tick must remain nonblocking");
        host_terminal
            .draw(|frame| super::render_workbench(frame, &state, &editor, &output))
            .expect("idle workbench render tick must succeed");
        thread::sleep(Duration::from_millis(250));
        max_rss = max_rss.max(process_rss_bytes(page_size));
    }
    let elapsed = started.elapsed();
    let end_ticks = process_cpu_ticks();
    let cpu_seconds = (end_ticks.saturating_sub(start_ticks)) as f64 / clock_ticks as f64;
    let cpu_percent_one_core = cpu_seconds / elapsed.as_secs_f64() * 100.0;
    let rss_overhead = max_rss.saturating_sub(baseline_rss);

    assert!(
        cpu_percent_one_core <= 2.0,
        "FR-051 idle CPU exceeded 2% of one logical core: {cpu_percent_one_core:.4}%"
    );
    assert!(
        rss_overhead <= 256 * 1024 * 1024,
        "FR-051 workbench RSS overhead exceeded 256 MiB: {rss_overhead} bytes"
    );

    close_all(&mut terminals, &mut state, &pane_ids);
    println!(
        "T097_IDLE_JSON={}",
        serde_json::to_string(&json!({
            "pane_count": IDLE_PANES,
            "duration_ms": u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
            "cpu_percent_one_logical_core": cpu_percent_one_core,
            "cpu_scope": "winds_workbench_process_only",
            "child_cpu_excluded": true,
            "rss_baseline_bytes": baseline_rss,
            "rss_max_bytes": max_rss,
            "rss_overhead_bytes": rss_overhead,
            "rss_scope": "winds_workbench_process_only",
            "child_memory_excluded": true,
            "idle_tick_includes_output_pumps_and_render": true,
        }))
        .expect("T097 idle evidence must serialize")
    );
}

fn required_env_u64(name: &str) -> u64 {
    std::env::var(name)
        .unwrap_or_else(|_| panic!("{name} must be provided by the pinned T097 workflow"))
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{name} must be an unsigned integer"))
}

fn process_cpu_ticks() -> u64 {
    let stat = fs::read_to_string("/proc/self/stat").expect("/proc/self/stat must be readable");
    let end_comm = stat
        .rfind(')')
        .expect("/proc/self/stat process name must terminate");
    let fields: Vec<&str> = stat[end_comm + 1..].split_whitespace().collect();
    let user = fields
        .get(11)
        .expect("/proc/self/stat utime must exist")
        .parse::<u64>()
        .expect("utime must be numeric");
    let system = fields
        .get(12)
        .expect("/proc/self/stat stime must exist")
        .parse::<u64>()
        .expect("stime must be numeric");
    user.saturating_add(system)
}

fn process_rss_bytes(page_size: u64) -> u64 {
    let statm = fs::read_to_string("/proc/self/statm").expect("/proc/self/statm must be readable");
    let resident_pages = statm
        .split_whitespace()
        .nth(1)
        .expect("/proc/self/statm resident pages must exist")
        .parse::<u64>()
        .expect("resident pages must be numeric");
    resident_pages.saturating_mul(page_size)
}

fn summary_json(summary: Summary) -> serde_json::Value {
    json!({
        "sample_count": summary.sample_count,
        "p50_us": summary.p50_us,
        "p95_us": summary.p95_us,
        "max_us": summary.max_us,
    })
}
