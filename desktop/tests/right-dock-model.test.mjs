import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import * as model from '../src/rightDock/model.ts';

const root = fileURLToPath(new URL('..', import.meta.url));
const surface = readFileSync(join(root, 'src/rightDock/RightDock.tsx'), 'utf8');
const bridge = readFileSync(join(root, 'src/rightDock/bridge.ts'), 'utf8');
const app = readFileSync(join(root, 'src/App.tsx'), 'utf8');
const sessionSurface = readFileSync(join(root, 'src/sessionSurface/SessionSurface.tsx'), 'utf8');

function binding(overrides = {}) {
  return {
    workspaceId: 'workspace-a',
    sessionId: 'session-a',
    worktreeRoot: '/repo/a',
    gitCommonDir: '/repo/a/.git',
    workflowRunId: null,
    stageRunId: null,
    candidateOid: null,
    candidateTree: null,
    headOid: 'a'.repeat(40),
    treeOid: 'b'.repeat(40),
    worktreeStateSha256: 'c'.repeat(64),
    bindingDigest: 'd'.repeat(64),
    ...overrides,
  };
}

test('T136 binding comparison rejects cross-session, tree, worktree-state, and digest movement', () => {
  const current = binding();
  assert.equal(model.sameRightDockBinding(current, { ...current }), true);
  for (const moved of [
    binding({ sessionId: 'session-b' }),
    binding({ treeOid: 'e'.repeat(40) }),
    binding({ candidateOid: '9'.repeat(40), candidateTree: '8'.repeat(40) }),
    binding({ worktreeStateSha256: 'f'.repeat(64) }),
    binding({ bindingDigest: '0'.repeat(64) }),
  ]) {
    assert.equal(model.sameRightDockBinding(current, moved), false);
  }
});

test('T136 Changes labels preserve raw Git status semantics without verification language', () => {
  assert.equal(model.statusLabel('??'), 'untracked');
  assert.equal(model.statusLabel(' M'), 'worktree M');
  assert.equal(model.statusLabel('M '), 'index M');
  assert.equal(model.statusLabel('MM'), 'index M · worktree M');
});



test('T136 path presentation escapes control and bidi-format characters without changing path identity', () => {
  assert.equal(model.displayPath('src/normal.ts'), 'src/normal.ts');
  assert.equal(model.displayPath('line\nname'), 'line\\u{A}name');
  assert.equal(model.displayPath(`spoof${String.fromCodePoint(0x202e)}txt`), 'spoof\\u{202E}txt');
});
test('T136 renderer exposes only fixed right-dock commands and no arbitrary filesystem/Git dispatcher', () => {
  for (const command of ['right_dock_bind', 'right_dock_files', 'right_dock_preview_file', 'right_dock_changes', 'right_dock_evidence', 'right_dock_context', 'right_dock_artifacts']) {
    assert.equal(bridge.includes(`invoke("${command}"`), true, command);
  }
  for (const forbidden of ['plugin-fs', 'plugin-shell', 'readTextFile', 'openPath', 'Command.create', 'git -C']) {
    assert.equal(bridge.includes(forbidden), false, forbidden);
    assert.equal(surface.includes(forbidden), false, forbidden);
  }
});

test('T136 delayed responses are generation- and immutable-binding checked before render', () => {
  assert.match(surface, /requestGeneration\.current/);
  assert.match(surface, /generation !== requestGeneration\.current/);
  assert.match(surface, /!sameRightDockBinding\(bound, response\.binding\)/);
  assert.match(surface, /Discarded stale Files result/);
  assert.match(surface, /Discarded stale Changes result/);
  assert.match(surface, /Discarded stale file preview/);
  assert.match(surface, /Discarded stale Evidence result/);
  assert.match(surface, /Discarded stale Context result/);
  assert.match(surface, /Discarded stale Artifacts result/);
  assert.match(surface, /TRUTH_REVALIDATION_MS = 2_000/);
  assert.match(surface, /Bound candidate\/tree moved · trusted dock treatment removed before refresh/);
  assert.match(surface, /Truth revalidation failed closed/);
  assert.match(surface, /setTruthRefresh\(\(value\) => value \+ 1\)/);
});

test('T136 right dock follows exact focused Session and file-change intent', () => {
  assert.match(app, /onFocusSession=\{focusDock\}/);
  assert.match(app, /onDockIntent=\{openDockIntent\}/);
  assert.match(app, /<RightDock target=\{dockTarget\} requestedSurface=\{dockSurface\}/);
  assert.match(sessionSurface, /onDockIntent\?\.\(event\.dockIntent, session\.canonicalWorkspaceId, session\.canonicalSessionId\)/);
  assert.equal(sessionSurface.includes('verified=true'), false);
});


test('T137 Evidence Context and Artifacts preserve source and authority boundaries', () => {
  for (const tab of ['evidence', 'context', 'artifacts']) {
    assert.equal(surface.includes(`surface === "${tab}"`), true, tab);
  }
  assert.match(surface, /Only persisted ELIGIBLE Winds evidence for the exact current candidate receives trusted treatment/);
  assert.match(surface, /Context is a source-labelled read-only projection/);
  assert.match(surface, /Artifact presence never grants verification authority/);
  assert.match(surface, /Safe in-dock reveal · no host open/);
  assert.equal(surface.includes('dangerouslySetInnerHTML'), false);
});
