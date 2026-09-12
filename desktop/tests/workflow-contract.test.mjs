import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const desktopRoot = new URL('..', import.meta.url).pathname;
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
