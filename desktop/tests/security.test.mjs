import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const root = new URL('..', import.meta.url).pathname;
const config = JSON.parse(readFileSync(join(root, 'src-tauri/tauri.conf.json'), 'utf8'));

const forbiddenDirectives = ['connect-src', 'object-src', 'frame-src', 'child-src', 'form-action'];

test('T128 keeps the privileged desktop host local and capability-free', () => {
  assert.equal(config.build.devUrl, undefined);
  assert.equal(config.build.frontendDist, '../dist');
  assert.deepEqual(config.app.security.capabilities, []);
  assert.equal(existsSync(join(root, 'src-tauri/capabilities')), false);
  assert.equal(config.bundle.active, false);
});

test('T128 CSP denies remote and active host escape surfaces', () => {
  const csp = config.app.security.csp;
  assert.match(csp, /default-src 'self'/);
  for (const directive of forbiddenDirectives) {
    assert.match(csp, new RegExp(`${directive} 'none'`));
  }
});
