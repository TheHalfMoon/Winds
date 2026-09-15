import assert from 'node:assert/strict';
import test from 'node:test';
import * as model from '../src/commandPalette/model.ts';

function session(id, name = id) {
  return {
    canonicalSessionId: id, canonicalWorkstreamId: 'ws', canonicalWorkspaceId: 'workspace-a',
    displayName: name, displayAlias: null, canonicalDisplayName: name, pinned: false,
    presentationOrder: 0, archived: false, presentationRevision: null,
    runtime: { state: 'unknown', requested: [], observed: [] }, attention: 'none', searchInput: `${name}\n${id}`,
  };
}

const snapshot = {
  projects: [{
    project: { projectViewId: 'workspace-a', displayName: 'Alpha', canonicalWorkspaceId: 'workspace-a', canonicalRepoRoot: '/repo/a', canonicalGitCommonDir: '/repo/a/.git', pinned: false, presentationOrder: 0, collapsed: false, presentationRevision: null, sessionCount: 2, attentionCount: 0, attention: 'none', searchInput: 'alpha' },
    sessions: [session('session-a', 'Duplicate'), session('session-b', 'Duplicate')], availableWorkstreams: [],
  }],
};

test('T138 palette exposes safe dock navigation including Needs You', () => {
  const items = model.commandPaletteItems(snapshot, '', null);
  assert.equal(items.some((item) => item.id === 'surface:needs_you'), true);
  assert.equal(items.some((item) => item.id === 'action:refresh'), true);
});

test('T138 Project navigation requires explicit Session disambiguation', () => {
  const project = model.commandPaletteItems(snapshot, 'Alpha', null).find((item) => item.kind === 'project');
  assert.equal(project.workspaceId, 'workspace-a');
  const scoped = model.commandPaletteItems(snapshot, '', project.workspaceId);
  assert.deepEqual(scoped.map((item) => item.sessionId), ['session-a', 'session-b']);
  assert.equal(scoped.every((item) => item.kind === 'session'), true);
});

test('T138 palette search retains exact canonical Session identity', () => {
  const items = model.commandPaletteItems(snapshot, 'session-b', null);
  const selected = items.find((item) => item.kind === 'session');
  assert.equal(selected.sessionId, 'session-b');
  assert.equal(selected.workspaceId, 'workspace-a');
});
