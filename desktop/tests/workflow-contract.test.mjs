import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const desktopRoot = fileURLToPath(new URL('..', import.meta.url));
const repositoryRoot = join(desktopRoot, '..');
const workflow = readFileSync(join(repositoryRoot, '.github/workflows/desktop-quality.yml'), 'utf8');
const pkg = JSON.parse(readFileSync(join(desktopRoot, 'package.json'), 'utf8'));

test('desktop-quality pins the first macOS image and attests its architecture', () => {
  assert.match(workflow, /runs-on: macos-15/);
  assert.match(workflow, /sw_vers/);
  assert.match(workflow, /uname -m/);
  assert.doesNotMatch(workflow, /runs-on: macos-latest/);
});

test('desktop-quality installs exact npm and uses the frozen frontend gates', () => {
  assert.match(workflow, /npm install --global npm@10\.9\.8/);
  assert.match(workflow, /npm ci --ignore-scripts/);
  assert.match(workflow, /npm run typecheck && npm run lint/);
  assert.match(workflow, /npm test && npm run frontend:build/);
});

test('Tauri release build is Cargo-lock constrained', () => {
  assert.equal(pkg.scripts['desktop:build'], 'tauri build --no-bundle --ci -- --locked');
  assert.match(workflow, /npm run desktop:build/);
});

const terminalWorkflow = readFileSync(
  join(repositoryRoot, '.github/workflows/t135-terminal-renderer.yml'),
  'utf8',
);

test('T135 terminal renderer qualification directly covers Linux macOS and Windows exact candidates', () => {
  for (const os of ['ubuntu-24.04', 'macos-15', 'windows-2025']) {
    assert.match(terminalWorkflow, new RegExp(`- ${os.replaceAll('.', '\\.')}`));
  }
  assert.match(terminalWorkflow, /CANDIDATE_SHA/);
  assert.match(terminalWorkflow, /git rev-parse HEAD/);
  assert.match(terminalWorkflow, /libwebkit2gtk-4\.1-dev/);
});

test('T135 terminal renderer qualification runs the exact focused bridge and locked desktop gates', () => {
  assert.match(terminalWorkflow, /cargo test --locked t135_desktop_terminal_tests -- --test-threads=1/);
  assert.match(terminalWorkflow, /npm ci --ignore-scripts/);
  assert.match(terminalWorkflow, /npm run frontend:build/);
  assert.match(terminalWorkflow, /cargo clippy --manifest-path desktop\/src-tauri\/Cargo\.toml --locked/);
  assert.match(terminalWorkflow, /npm run desktop:build/);
});

const runtimeAuthorityWorkflow = readFileSync(
  join(repositoryRoot, '.github/workflows/t139-runtime-authority.yml'),
  'utf8',
);

test('T139 runtime authority qualification binds exact candidate and proves unavailable decisions', () => {
  assert.match(runtimeAuthorityWorkflow, /CANDIDATE_SHA/);
  assert.match(runtimeAuthorityWorkflow, /git rev-parse HEAD/);
  assert.match(runtimeAuthorityWorkflow, /persist-credentials: false/);
  assert.match(runtimeAuthorityWorkflow, /npm ci --ignore-scripts/);
  assert.match(runtimeAuthorityWorkflow, /npm run format:check && npm run typecheck && npm run lint && npm test && npm run frontend:build/);
  assert.match(runtimeAuthorityWorkflow, /DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED/);
  assert.match(runtimeAuthorityWorkflow, /DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED/);
});
