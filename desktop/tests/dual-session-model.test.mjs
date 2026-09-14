import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import * as model from '../src/dualSession/model.ts';
import { createFixtureSubmission } from '../src/sessionSurface/model.ts';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));

function session(id, overrides = {}) {
  return {
    canonicalSessionId: id,
    canonicalWorkstreamId: 'workstream-a',
    canonicalWorkspaceId: 'workspace-a',
    displayName: id,
    displayAlias: null,
    canonicalDisplayName: id,
    pinned: false,
    presentationOrder: 0,
    archived: false,
    presentationRevision: null,
    runtime: { state: 'observed', requested: ['codex'], observed: ['codex'] },
    attention: 'none',
    searchInput: `${id}\nworkspace-a`,
    ...overrides,
  };
}

function project() {
  const sessions = [session('session-a'), session('session-b', { runtime: { state: 'requested', requested: ['claude'], observed: [] } }), session('session-c')];
  return {
    project: {
      projectViewId: 'workspace-a', displayName: 'Winds', canonicalWorkspaceId: 'workspace-a',
      canonicalRepoRoot: '/repo/Winds', canonicalGitCommonDir: '/repo/Winds/.git', pinned: false,
      presentationOrder: 0, collapsed: false, presentationRevision: 1, sessionCount: 3,
      attentionCount: 0, attention: 'none', searchInput: 'winds\nworkspace-a',
    },
    sessions,
    availableWorkstreams: [{ workstreamId: 'workstream-a', displayName: 'Primary' }],
  };
}

test('T134 defaults to exactly two distinct visible Session identities when available', () => {
  const layout = model.defaultLayoutForProject(project());
  assert.equal(layout.layoutMode, 'DUAL');
  assert.equal(layout.leftSessionId, 'session-a');
  assert.equal(layout.rightSessionId, 'session-b');
  assert.notEqual(layout.leftSessionId, layout.rightSessionId);
});

test('T134 swap and replace mutate only the exact requested Slot identity', () => {
  const owner = project();
  const initial = model.defaultLayoutForProject(owner);
  const swapped = model.swapSlots(initial);
  assert.deepEqual([swapped.leftSessionId, swapped.rightSessionId], ['session-b', 'session-a']);
  const replaced = model.replaceSlot(initial, owner, 'left', 'session-c');
  assert.equal(replaced.leftSessionId, 'session-c');
  assert.equal(replaced.rightSessionId, 'session-b');
  assert.equal(model.replaceSlot(initial, owner, 'left', 'session-b'), null);
  assert.equal(model.replaceSlot(initial, owner, 'right', 'missing'), null);
});

test('T134 left-dock selection targets only the focused Slot and never overwrites its peer', () => {
  const owner = project();
  const initial = model.defaultLayoutForProject(owner);
  const selectedLeft = model.selectSessionForSlot(initial, owner, 'left', 'session-c');
  assert.equal(selectedLeft.leftSessionId, 'session-c');
  assert.equal(selectedLeft.rightSessionId, 'session-b');
  const peerCollision = model.selectSessionForSlot(initial, owner, 'left', 'session-b');
  assert.equal(peerCollision, null);
  assert.equal(model.slotForSession(initial, 'session-a'), 'left');
  assert.equal(model.slotForSession(initial, 'session-b'), 'right');
  assert.equal(model.slotForSession(initial, 'session-c'), null);
});

test('T134 close-one and dual restore preserve the surviving peer identity', () => {
  const owner = project();
  const initial = model.defaultLayoutForProject(owner);
  const closeLeft = model.closeSlot(initial, 'left');
  assert.equal(closeLeft.layoutMode, 'SINGLE');
  assert.equal(closeLeft.leftSessionId, 'session-b');
  assert.equal(closeLeft.rightSessionId, null);
  const restored = model.enableDual(closeLeft, owner);
  assert.equal(restored.leftSessionId, 'session-b');
  assert.equal(restored.rightSessionId, 'session-a');
  const emptySingle = model.closeSlot({ ...closeLeft, leftSessionId: 'session-b' }, 'left');
  assert.equal(emptySingle.leftSessionId, null);
  const reopened = model.enableDual(emptySingle, owner);
  assert.equal(reopened.leftSessionId, 'session-a');
  assert.equal(reopened.rightSessionId, 'session-b');
});

test('T134 restart reconciliation keeps valid saved identities and fails stale identities to deterministic presentation defaults', () => {
  const owner = project();
  const saved = { workspaceId: 'workspace-a', layoutMode: 'DUAL', leftSessionId: 'session-c', rightSessionId: 'session-a', splitBasisPoints: 6200, revision: 7 };
  assert.deepEqual(model.reconcileLayout(saved, owner), saved);
  const stale = { ...saved, leftSessionId: 'deleted', rightSessionId: 'missing', splitBasisPoints: 9999 };
  const reconciled = model.reconcileLayout(stale, owner);
  assert.equal(reconciled.leftSessionId, 'session-a');
  assert.equal(reconciled.rightSessionId, 'session-b');
  assert.equal(reconciled.splitBasisPoints, 7500);
});

test('T134 persistence request carries exact identities and CAS revision only', () => {
  const layout = { ...model.defaultLayoutForProject(project()), revision: 4, splitBasisPoints: 4700 };
  assert.deepEqual(model.layoutSaveRequest(layout), {
    workspaceId: 'workspace-a', layoutMode: 'DUAL', leftSessionId: 'session-a',
    rightSessionId: 'session-b', splitBasisPoints: 4700, expectedRevision: 4,
  });
});

test('T134 adversarial target race cannot turn one Slot composer into peer or broadcast dispatch', () => {
  const owner = project();
  let layout = model.defaultLayoutForProject(owner);
  const rightIdentity = layout.rightSessionId;
  for (let index = 0; index < 100; index += 1) {
    const replacement = index % 2 === 0 ? 'session-c' : 'session-a';
    layout = model.replaceSlot(layout, owner, 'left', replacement) ?? layout;
    assert.equal(layout.rightSessionId, rightIdentity);
    const leftSurface = model.sessionForSlot(owner, layout.leftSessionId, 0);
    const submission = createFixtureSubmission(leftSurface, `prompt-${index}`, index + 1);
    assert.equal(submission.targetSessionId, layout.leftSessionId);
    assert.equal(submission.delivery, 'fixture_only');
    assert.notEqual(submission.targetSessionId, layout.rightSessionId);
  }
});

test('T134 renderer has explicit narrow fallback and no new direct host or runtime dispatch seam', () => {
  const workspace = readFileSync(join(root, 'src/dualSession/DualSessionWorkspace.tsx'), 'utf8');
  const app = readFileSync(join(root, 'src/App.tsx'), 'utf8');
  const dock = readFileSync(join(root, 'src/leftDock/LeftDock.tsx'), 'utf8');
  const styles = readFileSync(join(root, 'src/dualSession/dualSession.css'), 'utf8');
  assert.match(workspace, /Narrow fallback/);
  assert.match(styles, /session-slot\[data-focused="false"\]/);
  for (const forbidden of ['invoke(', '@tauri-apps/api', 'Command::new(', 'terminal_write', 'runtime_input', '.forEach((session)', 'Promise.all(']) {
    assert.equal(workspace.includes(forbidden), false, forbidden);
  }
  assert.match(workspace, /never broadcasts/);
  assert.match(app, /onSelectSession/);
  assert.match(app, /selection=\{selection\}/);
  assert.match(dock, /onSelectSession\?\.\(current\.canonicalWorkspaceId, session\.canonicalSessionId\)/);
});


test('T134 review regressions preserve flexible layout, queued selection, atomic Project/layout commit, surviving focus, and archived navigation refusal', () => {
  const workspace = readFileSync(join(root, 'src/dualSession/DualSessionWorkspace.tsx'), 'utf8');
  const dock = readFileSync(join(root, 'src/leftDock/LeftDock.tsx'), 'utf8');
  const styles = readFileSync(join(root, 'src/dualSession/dualSession.css'), 'utf8');
  assert.match(styles, /\.dual-session-layout\s*\{\s*grid-row:\s*4;/);
  assert.match(workspace, /pendingSelectionRef/);
  assert.match(workspace, /setSelectionReplayNonce/);
  assert.equal(workspace.includes('if (!selection || busyRef.current) return;'), false);
  assert.match(workspace, /await bridge\.saveLayout\(layoutSaveRequest\(next\)\);[\s\S]*?setProject\(owner\);[\s\S]*?setLayout\(reconcileLayout\(saved, owner\)\)/);
  assert.match(workspace, /setFocusedSlot\("left"\); setMaximizedSlot\(null\); operate\(closeSlot/);
  assert.match(dock, /if \(session\.archived\) \{ setStatus\(`Archived Session is not an active workspace target/);
});
