import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const root = new URL('..', import.meta.url).pathname;
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));

const expected = {
  react: '19.3.0',
  'react-dom': '19.3.0'
};

for (const [name, version] of Object.entries(expected)) {
  test(`pins ${name}@${version}`, () => assert.equal(pkg.dependencies[name], version));
}

for (const name of ['@tauri-apps/api', '@xterm/addon-fit', '@xterm/xterm', 'lucide-react']) {
  test(`defers unused runtime dependency ${name}`, () => {
    assert.equal(pkg.dependencies[name], undefined);
    assert.equal(pkg.devDependencies[name], undefined);
  });
}
const expectedDev = {
  '@tauri-apps/cli': '2.11.4',
  '@types/node': '22.20.2',
  '@types/react': '19.3.0',
  '@types/react-dom': '19.3.0',
  '@vitejs/plugin-react': '6.1.1',
  typescript: '7.0.2',
  vite: '8.3.0'
};

for (const [name, version] of Object.entries(expectedDev)) {
  test(`pins dev tool ${name}@${version}`, () => assert.equal(pkg.devDependencies[name], version));
}

test('pins the selected Node and npm toolchain', () => {
  assert.equal(pkg.engines.node, '22.22.3');
  assert.equal(pkg.engines.npm, '10.9.8');
  assert.equal(pkg.packageManager, 'npm@10.9.8');
});
