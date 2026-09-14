import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
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


test('T132 confines renderer host invocation to the typed left-dock bridge', () => {
  const sourceRoot = join(root, 'src');
  const sourceFiles = [];
  const collect = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) collect(path);
      else if (/\.(ts|tsx)$/.test(entry.name)) sourceFiles.push(path);
    }
  };
  collect(sourceRoot);
  const invoking = sourceFiles.filter((path) => {
    const text = readFileSync(path, 'utf8');
    return text.includes('@tauri-apps/api') || text.includes('invoke(');
  });
  assert.deepEqual(invoking.map((path) => path.slice(sourceRoot.length + 1)), ['leftDock/bridge.ts']);
});

test('T132 host errors log detail locally but return bounded renderer messages', () => {
  const host = readFileSync(join(root, 'src-tauri/src/main.rs'), 'utf8');
  assert.match(host, /eprintln!\("Winds desktop host:/);
  assert.equal(host.includes('error.to_string()'), false);
  assert.match(host, /Refresh and retry\./);
});
