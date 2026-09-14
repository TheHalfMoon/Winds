import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { findLiteralMatch, reconcileTerminalStatus, validatedHttpLink } from '../src/terminal/model.ts';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));

test('T135 literal terminal search is exact, deterministic, and wraps once', () => {
  const lines = ['alpha beta', 'VERIFIED forged', 'beta alpha'];
  assert.deepEqual(findLiteralMatch(lines, 'alpha', 1), { line: 2, column: 5, length: 5 });
  assert.deepEqual(findLiteralMatch(lines, 'alpha', 0), { line: 0, column: 0, length: 5 });
  assert.equal(findLiteralMatch(lines, 'Alpha', 0), null);
  assert.equal(findLiteralMatch(lines, '', 0), null);
});

test('T135 terminal link validation admits only http and https while host opening remains absent', () => {
  assert.equal(validatedHttpLink('https://example.com/path')?.protocol, 'https:');
  assert.equal(validatedHttpLink('http://example.com')?.protocol, 'http:');
  for (const value of ['javascript:alert(1)', 'file:///etc/passwd', 'mailto:test@example.com', 'not a url']) {
    assert.equal(validatedHttpLink(value), null);
  }
  const surface = readFileSync(join(root, 'src/terminal/TerminalSurface.tsx'), 'utf8');
  assert.match(surface, /registerOscHandler\(52/);
  assert.match(surface, /allowNonHttpProtocols: false/);
  assert.equal(surface.includes('window.open'), false);
  assert.equal(surface.includes('shell.open'), false);
  assert.equal(surface.includes('window.open'), false);
});

test('T135 renderer terminal bridge exposes only exact typed terminal commands', () => {
  const bridge = readFileSync(join(root, 'src/terminal/bridge.ts'), 'utf8');
  for (const name of [
    'terminal_status', 'terminal_start', 'terminal_input', 'terminal_resize',
    'terminal_interrupt', 'terminal_terminate', 'terminal_close'
  ]) assert.equal(bridge.includes(name), true, name);
  for (const forbidden of ['executable', 'cwd', 'command', 'shellCommand', 'filesystem', 'clipboard']) {
    assert.equal(bridge.includes(forbidden), false, forbidden);
  }
  assert.match(bridge, /Channel<ArrayBuffer>/);
});

test('T135 terminal is a sub-surface and does not replace the agent work stream', () => {
  const surface = readFileSync(join(root, 'src/sessionSurface/SessionSurface.tsx'), 'utf8');
  assert.match(surface, />Work Stream<\/button>/);
  assert.match(surface, />Terminal<\/button>/);
  assert.match(surface, /<TerminalSurface canonicalSessionId=\{session\.canonicalSessionId\}/);
  assert.match(surface, /hidden=\{surface !== "work_stream"\}/);
});


test('T135 terminal lifecycle cannot regress from a final state to live for the same terminal identity', () => {
  const live = { terminalId: 'terminal-a', canonicalSessionId: 'session-a', canonicalWorkspaceId: 'workspace-a', profileId: 'shell-a', profileDisplayName: 'sh', lifecycle: 'live', rows: 24, cols: 80, exitCode: null, signal: null, closeReason: null };
  const exited = { ...live, lifecycle: 'exited', exitCode: 0, closeReason: 'PROCESS_EXITED' };
  assert.deepEqual(reconcileTerminalStatus(live, exited), exited);
  assert.deepEqual(reconcileTerminalStatus(exited, live), exited);
  const nextLive = { ...live, terminalId: 'terminal-b' };
  assert.deepEqual(reconcileTerminalStatus(exited, nextLive), nextLive);
});
