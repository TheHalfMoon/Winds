import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const desktopRoot = fileURLToPath(new URL('..', import.meta.url));
const repoRoot = join(desktopRoot, '..');
const workflow = readFileSync(join(repoRoot, '.github/workflows/t142-native-platform.yml'), 'utf8');
const windowsWorkflow = readFileSync(join(repoRoot, '.github/workflows/windows-terminal.yml'), 'utf8');
const evidenceScript = readFileSync(join(repoRoot, 'scripts/ci/t142-platform-evidence.mjs'), 'utf8');
const provenance = readFileSync(join(repoRoot, 'docs/provenance/010-t142-native-platform.md'), 'utf8');
const accessibility = readFileSync(join(desktopRoot, 'tests/t140-accessibility.test.mjs'), 'utf8');
const mainSource = readFileSync(join(desktopRoot, 'src/main.tsx'), 'utf8');

function requireText(text, values) {
  for (const value of values) assert.equal(text.includes(value), true, value);
}

test('T142 directly builds the exact desktop candidate on Linux macOS and Windows', () => {
  requireText(workflow, [
    'os: [ubuntu-24.04, macos-15, windows-2025]',
    'fail-fast: false',
    'ref: ${{ env.CANDIDATE_SHA }}',
    'git rev-parse HEAD',
    'npm run desktop:build',
  ]);
});
test('T142 exercises native input focus resize and the platform terminal backend', () => {
  requireText(workflow, [
    't096_native_workbench_path_directly_qualifies_current_host_domain',
    'git::terminal::tests',
    'terminal::windows_tests',
    'wsl_launch::tests',
  ]);
  assert.match(workflow, /if: runner\.os != 'Windows'[\s\S]*git::terminal::tests/);
  assert.match(workflow, /if: runner\.os == 'Windows'[\s\S]*terminal::windows_tests/);
});

test('T142 records the actual WebView family and release binary identity per host', () => {
  requireText(workflow, [
    'pkg-config --modversion webkit2gtk-4.1',
    'WebKit.framework/Resources/Info.plist',
    'Microsoft\\EdgeWebView\\Application',
    't142-native-platform-${{ runner.os }}-${{ env.CANDIDATE_SHA }}',
  ]);
  requireText(evidenceScript, [
    "git('rev-parse', 'HEAD')",
    "git('rev-parse', 'HEAD^{tree}')",
    "createHash('sha256')",
    'WINDS_WEBVIEW_VERSION',
    'binary_sha256',
  ]);
});
test('T142 keeps native Windows and real WSL2 as distinct directly exercised domains', () => {
  requireText(windowsWorkflow, [
    'name: real-wsl2-integration (windows-2025 / Ubuntu)',
    'wsl.exe --set-default-version 2',
    'T062_REAL_WINDOWS_WSL2_INTEGRATION',
    't096_real_wsl2_workbench_path_preserves_host_guest_domain_and_path_truth',
    'repository_head -cne $expected',
    'mapped_workspace.git_head_oid -cne $expected',
  ]);
  for (const path of [
    'scripts/ci/t142-platform-evidence.mjs',
    'desktop/tests/t142-native-platform.test.mjs',
    'docs/provenance/010-t142-native-platform.md',
    '.github/workflows/t142-native-platform.yml',
  ]) {
    assert.equal(windowsWorkflow.split(path).length >= 3, true, path);
  }
});

test('T142 preserves renderer appearance IME and high-DPI contracts on every host', () => {
  requireText(workflow, ['npm test', 'npm run frontend:build']);
  requireText(accessibility, [
    'preserve IME safety',
    'prefers-color-scheme: light',
  ]);
  assert.match(mainSource, /scale\s*===\s*['"]125['"]\s*\|\|\s*scale\s*===\s*['"]200['"]/);
});
test('T142 keeps unsupported native GUI automation as explicit truthful nonclaims', () => {
  requireText(provenance, [
    'OS_LEVEL_TAURI_WINDOW_FOCUS_AUTOMATION=NOT_CLAIMED',
    'OS_LEVEL_IME_INJECTION_AUTOMATION=NOT_CLAIMED',
    'LIVE_NATIVE_SYSTEM_THEME_TRANSITION_AUTOMATION=NOT_CLAIMED',
    'PLATFORM_BUILD_SUBSTITUTES_FOR_T144_HUMAN_VISUAL_ACCEPTANCE=NO',
  ]);
  requireText(evidenceScript, [
    'No OS-level IME injection automation is claimed by T142.',
    'No live native system-theme transition automation is claimed by T142.',
    'T142 does not substitute platform builds for the human visual acceptance required by T144.',
  ]);
});

test('T142 qualification retains exact platform evidence as a candidate-bound artifact', () => {
  requireText(workflow, [
    'T142_EVIDENCE_PATH: t142-platform-${{ runner.os }}.json',
    'node scripts/ci/t142-platform-evidence.mjs',
    'if-no-files-found: error',
    'retention-days: 30',
  ]);
  assert.doesNotMatch(workflow, /continue-on-error:\s*true/);
});
