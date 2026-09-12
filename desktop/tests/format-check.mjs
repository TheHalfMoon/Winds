import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { extname, join } from 'node:path';

const root = new URL('..', import.meta.url).pathname;
const extensions = new Set(['.css', '.html', '.json', '.mjs', '.rs', '.toml', '.ts', '.tsx']);
const ignored = new Set(['dist', 'gen', 'node_modules', 'target']);

function files(directory) {
  return readdirSync(directory).flatMap((name) => {
    if (ignored.has(name)) return [];
    const path = join(directory, name);
    if (statSync(path).isDirectory()) return files(path);
    return extensions.has(extname(path)) ? [path] : [];
  });
}

for (const path of files(root)) {
  const text = readFileSync(path, 'utf8');
  assert.equal(text.includes('\r'), false, `${path}: CRLF is not canonical`);
  assert.equal(text.endsWith('\n'), true, `${path}: missing final newline`);
  text.split('\n').forEach((line, index) => {
    assert.equal(/[ \t]+$/.test(line), false, `${path}:${index + 1}: trailing whitespace`);
  });
}
