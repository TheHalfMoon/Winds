import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const root = new URL('..', import.meta.url).pathname;
const host = readFileSync(join(root, 'src-tauri/src/main.rs'), 'utf8');
const app = readFileSync(join(root, 'src/App.tsx'), 'utf8');
const main = readFileSync(join(root, 'src/main.tsx'), 'utf8');
const combined = `${host}\n${app}\n${main}`;

const forbidden = [
  '#[tauri::command]',
  '.invoke_handler(',
  '@tauri-apps/plugin-',
  '@tauri-apps/api/core',
  'invoke(',
  'Command::new(',
  'std::fs',
  'std::process'
];

test('T128 source exposes no renderer-to-host product authority', () => {
  for (const token of forbidden) assert.equal(combined.includes(token), false, token);
});
