import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const root = new URL('..', import.meta.url).pathname;
const host = readFileSync(join(root, 'src-tauri/src/main.rs'), 'utf8');
const commandNames = [
  'left_dock_snapshot',
  'left_dock_update_project',
  'left_dock_create_session',
  'left_dock_rename_session',
  'left_dock_update_session'
];

test('T132 exposes only the five authorized left-dock Tauri commands', () => {
  assert.equal((host.match(/#\[tauri::command\]/g) ?? []).length, commandNames.length);
  for (const name of commandNames) assert.equal(host.includes(name), true, name);
  for (const forbidden of ['@tauri-apps/plugin-', 'Command::new(', 'std::fs', 'std::process']) {
    assert.equal(host.includes(forbidden), false, forbidden);
  }
});
