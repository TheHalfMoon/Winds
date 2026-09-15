import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const host = readFileSync(join(root, 'src-tauri/src/main.rs'), 'utf8');
const commandNames = [
  'left_dock_snapshot',
  'left_dock_update_project',
  'left_dock_create_session',
  'left_dock_rename_session',
  'left_dock_update_session',
  'workspace_load_layout',
  'workspace_save_layout',
  'right_dock_bind',
  'right_dock_files',
  'right_dock_preview_file',
  'right_dock_changes',
  'right_dock_evidence',
  'right_dock_context',
  'right_dock_artifacts',
  'terminal_status',
  'terminal_start',
  'terminal_input',
  'terminal_resize',
  'terminal_interrupt',
  'terminal_terminate',
  'terminal_close'
];

test('T137 exposes only bounded left-dock, layout, right-dock inspection, and terminal Tauri commands', () => {
  assert.equal((host.match(/#\[tauri::command\]/g) ?? []).length, commandNames.length);
  for (const name of commandNames) assert.equal(host.includes(name), true, name);
  for (const forbidden of ['@tauri-apps/plugin-', 'Command::new(', 'std::fs', 'std::process']) {
    assert.equal(host.includes(forbidden), false, forbidden);
  }
});

test('T135 retains the native Windows Tauri icon resource', () => {
  assert.equal(existsSync(join(root, 'src-tauri/icons/icon.ico')), true);
});


test('T135 terminal startup is async and moves blocking preparation outside the registry lock', () => {
  assert.match(host, /async fn terminal_start\(/);
  assert.match(host, /tauri::async_runtime::spawn_blocking/);
  assert.match(host, /reserve_start/);
  assert.match(host, /DesktopTerminalRegistry::prepare_start/);
  assert.match(host, /commit_prepared_start/);
});
