import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const desktopRoot = fileURLToPath(new URL('..', import.meta.url));
const repoRoot = join(desktopRoot, '..');
const workflow = readFileSync(join(repoRoot, '.github/workflows/t143-performance.yml'), 'utf8');

const t142aWorkflow = readFileSync(join(repoRoot, '.github/workflows/t142a-winds-identity.yml'), 'utf8');
const host = readFileSync(join(desktopRoot, 'src-tauri/src/main.rs'), 'utf8');
const hostManifest = readFileSync(join(desktopRoot, 'src-tauri/Cargo.toml'), 'utf8');
const main = readFileSync(join(desktopRoot, 'src/main.tsx'), 'utf8');
const fixture = readFileSync(join(desktopRoot, 'src/leftDock/fixture.ts'), 'utf8');
const bridge = readFileSync(join(desktopRoot, 'src/leftDock/bridge.ts'), 'utf8');
const nativeHarness = readFileSync(join(desktopRoot, 'tests/performance/t143_native.py'), 'utf8');
const webkitHarness = readFileSync(join(desktopRoot, 'tests/performance/t143_webkit.py'), 'utf8');

test('T143 native readiness is renderer-only and adds no host feature or production Tauri command', () => {
  assert.doesNotMatch(hostManifest, /t143-benchmark/);
  assert.equal((host.match(/#\[tauri::command\]/g) ?? []).length, 22);
  assert.doesNotMatch(host, /PageLoadEvent|WINDS_T143_READY_MS|t143-ready/);
  assert.match(main, /VITE_WINDS_T143_NATIVE_READY === "1"/);
  assert.match(main, /document\.title = "Winds \[T143 Ready\]"/);
  assert.doesNotMatch(main, /searchParams\.set\("t143-ready"|location\.replace/);
  assert.match(bridge, /VITE_WINDS_T143_NATIVE_READY === "1"/);
  assert.doesNotMatch(main, /@tauri-apps\/api|invoke\(/);
});

test('T143 large performance fixture is benchmark-only and exceeds the frozen scale floor', () => {
  assert.match(fixture, /T143_PROJECT_COUNT = 100/);
  assert.match(fixture, /T143_SESSIONS_PER_PROJECT = 10/);
  assert.match(fixture, /VITE_WINDS_T143_BENCHMARK !== "1"/);
  assert.match(bridge, /VITE_WINDS_T143_BENCHMARK === "1"/);
  assert.match(fixture, /t143-large/);
});

test('T143 native harness directly enforces cold-launch and idle CPU RSS ceilings', () => {
  assert.match(nativeHarness, /--launches.*default=20/);
  assert.match(nativeHarness, /--idle-seconds.*default=60\.0/);
  assert.match(nativeHarness, /cold_launch_p95_le_1500_ms/);
  assert.match(nativeHarness, /idle_cpu_le_2_percent_one_core/);
  assert.match(nativeHarness, /renderer_host_idle_rss_le_300_mib/);
  assert.match(nativeHarness, /rss_max_by_role_mib/);
  assert.match(nativeHarness, /process_tree_rss_max_mib/);
  assert.match(nativeHarness, /renderer_host_rss_bytes/);
  assert.match(nativeHarness, /role in \{\"host\", \"renderer\"\}/);
  assert.match(nativeHarness, /\"process_rss\"/);
  assert.match(nativeHarness, /xdotool/);
  assert.match(nativeHarness, /renderer_present_every_idle_sample/);
  assert.match(nativeHarness, /renderer_host_process_count_ge_2_every_idle_sample/);
  assert.match(nativeHarness, /renderer_present_samples == len\(samples\)/);
  assert.match(nativeHarness, /elif \"WebKit\" in name:/);
  assert.match(nativeHarness, /if role == \"renderer\":/);
  assert.match(nativeHarness, /renderer_present = true|renderer_present = True/);
  assert.match(nativeHarness, /descendants\(proc\.pid\)/);
});


test('T143 Linux reference runtime retains only the measured-beneficial JSC JIT profile', () => {
  const runLinux = readFileSync(join(desktopRoot, 'tests/performance/t143_run_linux.sh'), 'utf8');
  assert.match(host, /configure_linux_webkit_memory_profile/);
  assert.match(host, /var_os\("JSC_useJIT"\)\.is_none\(\)/);
  assert.match(host, /set_var\("JSC_useJIT", "false"\)/);
  assert.match(runLinux, /export JSC_useJIT=false/);
  assert.doesNotMatch(host, /JSC_libpasScavengeContinuously/);
  assert.doesNotMatch(runLinux, /JSC_libpasScavengeContinuously/);
});

test('T143 WebKitGTK harness retains raw samples for every frozen local interaction budget', () => {
  for (const needle of [
    'selection_p95_le_50_ms',
    'single_dual_p95_le_100_ms',
    'composer_p95_le_16_ms',
    'cached_right_dock_p95_le_50_ms',
    'resize_p95_le_100_ms',
    'large_search_p95_le_50_ms',
    'large_scroll_p95_le_50_ms',
    'large_focus_p95_le_50_ms',
  ]) assert.equal(webkitHarness.includes(needle), true, needle);
  assert.match(webkitHarness, /"raw_ms"/);
  assert.match(webkitHarness, /visibleSessionWorkEvents/);
});

test('T143 workflow separates native qualification bytes from benchmark renderer bytes without new package dependency', () => {
  assert.match(workflow, /runs-on: ubuntu-24\.04/);
  assert.match(workflow, /CANDIDATE_SHA/);
  assert.match(workflow, /git rev-parse HEAD/);
  assert.match(workflow, /webkit2gtk-driver epiphany-browser xvfb dbus-x11 xdotool/);
  assert.match(workflow, /VITE_WINDS_T143_NATIVE_READY=1 npm run frontend:build/);
  assert.match(workflow, /cargo build --release --locked --manifest-path desktop\/src-tauri\/Cargo\.toml/);
  assert.doesNotMatch(workflow, /--features t143-benchmark/);
  assert.match(workflow, /native-renderer-dist/);
  assert.match(workflow, /VITE_WINDS_T143_BENCHMARK=1 npm run frontend:build/);
  assert.match(workflow, /benchmark-renderer-dist/);
  assert.match(workflow, /t097_release_benchmark_campaign/);
  assert.match(workflow, /t143_assemble\.py/);
  assert.match(workflow, /actions\/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a/);
});


test('T143 repairs the closed T142A regression gate without reapplying historical task scope to successors', () => {
  assert.match(t142aWorkflow, /Verify closed T142A historical scope and merge identity/);
  assert.match(t142aWorkflow, /54880f256551659be5fb4b4456754c7c9f69d9d9/);
  assert.match(t142aWorkflow, /521787cdad46ef89325b3d97b43be397e4673b01/);
  assert.match(t142aWorkflow, /8d75cab455f11afddeaa6fbe6244dbb1021d1467/);
  assert.doesNotMatch(t142aWorkflow, /BASE_SHA|os\.environ\[\"BASE_SHA\"\]/);
  assert.match(t142aWorkflow, /git\", \"diff\", \"--name-only\", canonical_base, canonical_head/);
  assert.match(t142aWorkflow, /Current Spectrum deterministic gates/);
});
