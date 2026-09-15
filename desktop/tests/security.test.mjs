import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
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


test('T136 confines renderer host invocation to three typed bridge modules', () => {
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
  assert.deepEqual(invoking.map((path) => path.slice(sourceRoot.length + 1).replaceAll('\\', '/')).sort(), ['leftDock/bridge.ts', 'rightDock/bridge.ts', 'terminal/bridge.ts']);
});

test('T132 host errors log detail locally but return bounded renderer messages', () => {
  const host = readFileSync(join(root, 'src-tauri/src/main.rs'), 'utf8');
  assert.match(host, /eprintln!\("Winds desktop host:/);
  assert.equal(host.includes('error.to_string()'), false);
  assert.match(host, /Refresh and retry\./);
});

test('T132 sends reorder plans as one bounded host batch', () => {
  const dock = readFileSync(join(root, 'src/leftDock/LeftDock.tsx'), 'utf8');
  const bridge = readFileSync(join(root, 'src/leftDock/bridge.ts'), 'utf8');
  assert.match(dock, /bridge\.updateProject\(plan\)/);
  assert.match(dock, /bridge\.updateSession\(plan\)/);
  assert.equal(dock.includes('for (const request of plan)'), false);
  assert.match(bridge, /left_dock_update_project[\s\S]*request: \{ updates \}/);
  assert.match(bridge, /left_dock_update_session[\s\S]*request: \{ updates \}/);
});


test('T132 restores inline-form trigger focus only after explicit successful exits', () => {
  const dock = readFileSync(join(root, 'src/leftDock/LeftDock.tsx'), 'utf8');
  assert.match(dock, /function closeRenameEditor\(\)[\s\S]*onFocusKey\(renameTriggerKey\)[\s\S]*setEditing\(false\)/);
  assert.match(dock, /if \(await onRename\(displayName\)\) closeRenameEditor\(\)/);
  assert.match(dock, /closeCreateSession\(project\.project\.canonicalWorkspaceId\)/);
  assert.match(dock, /onClick=\{\(\) => closeCreateSession\(current\.canonicalWorkspaceId\)\}/);
});

test('T132 manual refresh failures are surfaced instead of becoming unhandled rejections', () => {
  const dock = readFileSync(join(root, 'src/leftDock/LeftDock.tsx'), 'utf8');
  assert.match(dock, /refresh\(\)\.catch\(\(error\) => setStatus\(`Refresh failed/);
});
