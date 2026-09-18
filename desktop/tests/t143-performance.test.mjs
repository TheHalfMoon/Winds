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
const tauriConfig = readFileSync(join(desktopRoot, 'src-tauri/tauri.conf.json'), 'utf8');
const main = readFileSync(join(desktopRoot, 'src/main.tsx'), 'utf8');
const fixture = readFileSync(join(desktopRoot, 'src/leftDock/fixture.ts'), 'utf8');
const bridge = readFileSync(join(desktopRoot, 'src/leftDock/bridge.ts'), 'utf8');
const leftDock = readFileSync(join(desktopRoot, 'src/leftDock/LeftDock.tsx'), 'utf8');
const styles = readFileSync(join(desktopRoot, 'src/styles.css'), 'utf8');
const rightDockBridge = readFileSync(join(desktopRoot, 'src/rightDock/bridge.ts'), 'utf8');
const nativeHarness = readFileSync(join(desktopRoot, 'tests/performance/t143_native.py'), 'utf8');
const platformBaselineHarness = readFileSync(join(desktopRoot, 'tests/performance/t143_platform_baseline.py'), 'utf8');
const platformBaselineHtml = readFileSync(join(desktopRoot, 'tests/performance/t143_platform_baseline/index.html'), 'utf8');
const reactBaselineHtml = readFileSync(join(desktopRoot, 'tests/performance/t143_react_baseline/index.html'), 'utf8');
const reactBaselineMain = readFileSync(join(desktopRoot, 'tests/performance/t143_react_baseline/main.tsx'), 'utf8');
const webkitHarness = readFileSync(join(desktopRoot, 'tests/performance/t143_webkit.py'), 'utf8');
const selectionProbe = readFileSync(join(desktopRoot, 'tests/performance/t143_selection_probe.py'), 'utf8');
const assembleHarness = readFileSync(join(desktopRoot, 'tests/performance/t143_assemble.py'), 'utf8');

test('T143 native readiness uses a qualification-only window-title capability and no production command', () => {
  assert.doesNotMatch(hostManifest, /t143-benchmark/);
  assert.equal((host.match(/#\[tauri::command\]/g) ?? []).length, 22);
  assert.doesNotMatch(host, /PageLoadEvent|WINDS_T143_READY_MS|t143-ready/);
  assert.match(main, /VITE_WINDS_T143_NATIVE_READY === "1"/);
  assert.match(main, /markT143NativeReadyWindow\(\)/);
  assert.doesNotMatch(main, /@tauri-apps\/api/);
  assert.match(bridge, /markT143NativeReadyWindow/);
  assert.match(bridge, /import\("@tauri-apps\/api\/window"\)/);
  assert.match(bridge, /getCurrentWindow\(\)\.setTitle\("Winds \[T143 Ready\]"\)/);
  assert.doesNotMatch(main, /searchParams\.set\("t143-ready"|location\.replace/);
  assert.match(bridge, /VITE_WINDS_T143_NATIVE_READY === "1"/);
  assert.match(rightDockBridge, /VITE_WINDS_T143_NATIVE_READY === "1"/);
  assert.match(rightDockBridge, /return fixtureBridge/);
  assert.match(tauriConfig, /"capabilities": \[\]/);
  assert.match(workflow, /TAURI_CONFIG=.*t143-native-ready/);
  assert.match(workflow, /core:window:allow-set-title/);
  assert.doesNotMatch(workflow, /core:window:allow-(?!set-title)/);
});

test('T143 large performance fixture is benchmark-only and exceeds the frozen scale floor', () => {
  assert.match(fixture, /T143_PROJECT_COUNT = 100/);
  assert.match(fixture, /T143_SESSIONS_PER_PROJECT = 10/);
  assert.match(fixture, /VITE_WINDS_T143_BENCHMARK !== "1"/);
  assert.match(bridge, /VITE_WINDS_T143_BENCHMARK === "1"/);
  assert.match(fixture, /t143-large/);
  assert.match(leftDock, /data-project-browse/);
  assert.doesNotMatch(leftDock, /\shidden=\{searching\}/);
  assert.ok(leftDock.includes('data-search-hidden={searching ? "true" : "false"}'));
  assert.ok(leftDock.includes('aria-hidden={searching}'));
  assert.ok(leftDock.includes('inert={searching ? true : undefined}'));
  assert.match(leftDock, /data-project-search-results/);
  assert.match(leftDock, /focusedControlKeyRef/);
  assert.doesNotMatch(leftDock, /\[focusedControlKey, setFocusedControlKey\]/);
  assert.ok(styles.includes('[data-project-browse][data-search-hidden="true"]'));
  assert.ok(styles.includes('content-visibility: hidden'));
  assert.ok(styles.includes('contain-intrinsic-block-size: 0px'));
  assert.match(styles, /\[data-project-browse\] \.project-group[\s\S]*content-visibility: auto/);
  assert.match(styles, /contain-intrinsic-block-size: auto 445px/);
});

test('T143 native harness directly enforces cold-launch and idle CPU RSS ceilings', () => {
  assert.match(nativeHarness, /--launches.*default=20/);
  assert.match(nativeHarness, /--idle-seconds.*default=60\.0/);
  assert.match(nativeHarness, /cold_launch_p95_le_1500_ms/);
  assert.match(nativeHarness, /idle_cpu_le_2_percent_one_core/);
  assert.match(nativeHarness, /renderer_host_idle_rss_le_320_mib/);
  assert.match(nativeHarness, /320 \* 1024 \* 1024/);
  assert.doesNotMatch(nativeHarness, /renderer_host_idle_rss_le_300_mib/);
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
  assert.match(nativeHarness, /tempfile\.TemporaryFile\(\)/);
  assert.match(nativeHarness, /os\.pread\(/);
  assert.doesNotMatch(nativeHarness, /stdout=subprocess\.PIPE/);
  assert.match(nativeHarness, /Path\(f"\/proc\/\{pid\}\/task"\)\.iterdir\(\)/);
});


test('T143 Linux reference runtime carries measured allocator defaults and the pinned Xvfb fallback', () => {
  const runLinux = readFileSync(join(desktopRoot, 'tests/performance/t143_run_linux.sh'), 'utf8');
  assert.match(host, /configure_linux_webkit_memory_profile/);
  assert.match(host, /var_os\("JSC_useJIT"\)\.is_none\(\)/);
  assert.match(host, /set_var\("JSC_useJIT", "false"\)/);
  assert.match(runLinux, /JSC_useJIT=false/);
  assert.match(host, /var_os\("Malloc"\)\.is_none\(\)/);
  assert.match(host, /set_var\("Malloc", "1"\)/);
  assert.match(runLinux, /Malloc=1/);
  assert.match(host, /set_var\("MALLOC_ARENA_MAX", "1"\)/);
  assert.match(runLinux, /MALLOC_ARENA_MAX=1/);
  assert.match(runLinux, /WINDS_T143_WEBKIT_BROWSER_NAME=MiniBrowser/);
  assert.match(runLinux, /WINDS_T143_WEBKIT_ARGUMENT=--automation/);
  assert.match(runLinux, /webkit2gtk-4\.1\/MiniBrowser/);
  assert.doesNotMatch(runLinux, /epiphany|--automation-mode/);
  assert.doesNotMatch(host, /GLIBC_TUNABLES|glibc\.malloc\.tcache_count/);
  assert.doesNotMatch(runLinux, /GLIBC_TUNABLES|glibc\.malloc\.tcache_count/);
  assert.doesNotMatch(runLinux, /export (?:JSC_useJIT|Malloc|MALLOC_ARENA_MAX)=/);
  assert.doesNotMatch(host, /JSC_forceRAMSize/);
  assert.doesNotMatch(runLinux, /JSC_forceRAMSize/);
  assert.doesNotMatch(host, /JSC_aggressiveHeapThresholdInMB/);
  assert.doesNotMatch(runLinux, /JSC_aggressiveHeapThresholdInMB/);
  assert.match(runLinux, /export WEBKIT_DISABLE_DMABUF_RENDERER=1/);
  assert.doesNotMatch(nativeHarness, /WEBKIT_DISABLE_DMABUF_RENDERER/);
  assert.doesNotMatch(host, /MALLOC_TRIM_THRESHOLD_/);
  assert.doesNotMatch(runLinux, /MALLOC_TRIM_THRESHOLD_/);
  assert.doesNotMatch(host, /JSC_libpasScavengeContinuously/);
  assert.doesNotMatch(runLinux, /JSC_libpasScavengeContinuously/);
});

test('T143 platform baseline measures the same-host minimal-renderer baseline without changing the product gate', () => {
  const runLinux = readFileSync(join(desktopRoot, 'tests/performance/t143_run_linux.sh'), 'utf8');
  assert.match(platformBaselineHtml, /Winds T143 platform baseline/);
  assert.match(platformBaselineHarness, /Diagnostic same-host minimal-renderer baseline/);
  assert.match(platformBaselineHarness, /native\.idle_campaign\(proc, args\.idle_seconds, settle=2\.0\)/);
  assert.match(platformBaselineHarness, /renderer_present_every_idle_sample/);
  assert.match(platformBaselineHarness, /renderer_host_process_count_ge_2_every_idle_sample/);
  assert.match(platformBaselineHarness, /diagnostic_renderer_host_idle_rss_le_300_mib/);
  assert.doesNotMatch(platformBaselineHarness, /all_checks_pass/);
  assert.match(runLinux, /t143_platform_baseline\.py/);
  assert.match(runLinux, /platform-baseline\.json/);
  assert.match(runLinux, /--idle-seconds 60/);
  assert.equal(workflow.includes('frontendDist\":\"../tests/performance/t143_platform_baseline'), true);
  assert.match(workflow, /Winds \[T143 Platform Baseline\]/);
  assert.match(workflow, /platform-baseline-binary\.sha256/);
  assert.match(workflow, /test \"\$baseline_sha\" != \"\$product_sha\"/);
  assert.doesNotMatch(workflow, /--native t143-evidence\/platform-baseline\.json/);
});

test('T143 React baseline isolates selected frontend runtime cost without substituting for the product gate', () => {
  const runLinux = readFileSync(join(desktopRoot, 'tests/performance/t143_run_linux.sh'), 'utf8');
  assert.match(reactBaselineHtml, /Winds T143 React Baseline/);
  assert.match(reactBaselineMain, /StrictMode/);
  assert.match(reactBaselineMain, /createRoot/);
  assert.match(reactBaselineMain, /getCurrentWindow/);
  assert.match(reactBaselineMain, /Winds \[T143 React Baseline\]/);
  assert.match(workflow, /npx vite build tests\/performance\/t143_react_baseline --config vite\.config\.ts/);
  assert.equal(workflow.includes('frontendDist\":\"../tests/performance/t143_react_baseline/dist'), true);
  assert.match(workflow, /react-baseline-binary\.sha256/);
  assert.match(workflow, /winds-desktop-host-react-baseline/);
  assert.match(runLinux, /react-baseline\.json/);
  assert.match(runLinux, /winds-t143-react-baseline-v1/);
  assert.match(runLinux, /Diagnostic same-host minimal React renderer baseline/);
  assert.match(runLinux, /--idle-seconds 60/);
  assert.doesNotMatch(workflow, /--native t143-evidence\/react-baseline\.json/);
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
  assert.match(webkitHarness, /WEBDRIVER_SCRIPT_TIMEOUT_MS = 600_000/);
  assert.match(webkitHarness, /webkit2gtk-4\.1\/MiniBrowser/);
  assert.match(webkitHarness, /WINDS_T143_WEBKIT_BROWSER_NAME\", \"MiniBrowser/);
  assert.match(webkitHarness, /WINDS_T143_WEBKIT_ARGUMENT\", \"--automation/);
  assert.doesNotMatch(webkitHarness, /epiphany|--automation-mode/);
  assert.match(selectionProbe, /NUL = chr\(0\)/);
  assert.match(selectionProbe, /session_key\("fixture-winds", "fixture-session-a"\)/);
  assert.match(selectionProbe, /session_key\("fixture-winds", "fixture-session-b"\)/);
  assert.match(selectionProbe, /Diagnostic only; never contributes to T143 qualification checks or sample floors/);
  assert.match(webkitHarness, /WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS = 660\.0/);
  assert.match(webkitHarness, /"script": WEBDRIVER_SCRIPT_TIMEOUT_MS/);
  assert.match(webkitHarness, /timeout=WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS/);
  assert.match(webkitHarness, /urlopen\(request, timeout=timeout\)/);
  assert.match(webkitHarness, /renderer-preflight\.json/);
  assert.match(webkitHarness, /renderer-failure\.json/);
  assert.match(webkitHarness, /winds-t143-renderer-preflight-v1/);
  assert.match(webkitHarness, /winds-t143-renderer-failure-v1/);
  assert.match(webkitHarness, /browser_phase/);
  assert.match(webkitHarness, /normal:selection/);
  assert.match(webkitHarness, /selectExactSession/);
  assert.match(webkitHarness, /exactSessionCommitted/);
  assert.match(webkitHarness, /chat-session-identity/);
  assert.match(webkitHarness, /session-slot\[data-focused=\"true\"\] \.session-identity/);
  assert.match(webkitHarness, /focusedIdentity/);
  assert.match(webkitHarness, /sampleIndex/);
  assert.match(webkitHarness, /errorMessage/);
  assert.match(webkitHarness, /errorStack/);
  assert.match(webkitHarness, /normal:composer/);
  assert.match(webkitHarness, /normal:layout/);
  assert.match(webkitHarness, /normal:right-dock/);
  assert.match(webkitHarness, /normal:resize/);
  assert.match(webkitHarness, /large:search/);
  assert.match(webkitHarness, /Object\.getOwnPropertyDescriptor\(HTMLInputElement\.prototype, 'value'\)/);
  assert.match(webkitHarness, /inputValueSetter\.call\(search, value\)/);
  assert.match(webkitHarness, /setSearchValue\(query\)/);
  assert.match(webkitHarness, /setSearchValue\(''\)/);
  assert.doesNotMatch(webkitHarness, /search\.value = query/);
  assert.doesNotMatch(webkitHarness, /search\.value = ''/);
  assert.match(webkitHarness, /visibleSessionRows/);
  assert.ok(webkitHarness.includes("closest('[hidden], [inert]')"));
  assert.match(webkitHarness, /large:scroll/);
  assert.match(webkitHarness, /large:focus/);
});

test('T143 workflow separates native qualification bytes from benchmark renderer bytes without new package dependency', () => {
  assert.match(workflow, /runs-on: ubuntu-24\.04/);
  assert.match(workflow, /CANDIDATE_SHA/);
  assert.match(workflow, /T143_RSS_AMENDMENT_SHA: 55c29ebb5833a1f2856e6f3a820cc5484940b705/);
  assert.match(workflow, /git merge-base --is-ancestor \"\$T143_RSS_AMENDMENT_SHA\" \"\$CANDIDATE_SHA\"/);
  assert.match(workflow, /git rev-parse HEAD/);
  assert.match(workflow, /libwebkit2gtk-4\.1-dev webkit2gtk-driver xvfb dbus-x11 xdotool/);
  assert.doesNotMatch(workflow, /epiphany-browser/);
  assert.match(workflow, /test -x \/usr\/lib\/x86_64-linux-gnu\/webkit2gtk-4\.1\/MiniBrowser/);
  assert.match(workflow, /VITE_WINDS_T143_NATIVE_READY=1 npm run frontend:build/);
  assert.match(workflow, /TAURI_CONFIG=.*core:window:allow-set-title/);
  assert.match(workflow, /cargo build --release --locked --manifest-path desktop\/src-tauri\/Cargo\.toml/);
  assert.doesNotMatch(workflow, /--features t143-benchmark/);
  assert.match(workflow, /native-renderer-dist/);
  assert.match(workflow, /VITE_WINDS_T143_BENCHMARK=1 npm run frontend:build/);
  assert.match(workflow, /benchmark-renderer-dist/);
  assert.match(workflow, /t097_release_benchmark_campaign/);
  assert.match(workflow, /t143_assemble\.py/);
  assert.match(assembleHarness, /rss_budget_amendment/);
  assert.match(assembleHarness, /renderer_host_idle_rss_budget_mib/);
  assert.match(assembleHarness, /320/);
  assert.match(assembleHarness, /minibrowser_path/);
  assert.match(assembleHarness, /minibrowser_sha256/);
  assert.match(assembleHarness, /minibrowser_package/);
  assert.doesNotMatch(assembleHarness, /epiphany/);
  assert.match(workflow, /actions\/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a/);
  assert.match(workflow, /desktop\/src-tauri\/Cargo\.toml desktop\/src-tauri\/Cargo\.lock/);
  assert.doesNotMatch(workflow, /unexpected T143 Linux dependency set/);
});


test('T143 repairs the closed T142A regression gate without reapplying historical task scope to successors', () => {
  assert.match(t142aWorkflow, /Verify closed T142A historical scope and merge identity/);
  assert.match(t142aWorkflow, /54880f256551659be5fb4b4456754c7c9f69d9d9/);
  assert.match(t142aWorkflow, /521787cdad46ef89325b3d97b43be397e4673b01/);
  assert.match(t142aWorkflow, /8d75cab455f11afddeaa6fbe6244dbb1021d1467/);
  assert.doesNotMatch(t142aWorkflow, /BASE_SHA|os\.environ\[\"BASE_SHA\"\]/);
  assert.match(t142aWorkflow, /git\", \"diff\", \"--name-only\", canonical_base, canonical_head/);
  assert.match(t142aWorkflow, /merge-base\", \"--is-ancestor\", canonical_merge, \"HEAD\"/);
  assert.match(t142aWorkflow, /current candidate does not descend from the canonical T142A merge/);
  assert.match(t142aWorkflow, /Current Spectrum deterministic gates/);
});
