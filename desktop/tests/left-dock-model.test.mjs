import assert from 'node:assert/strict';
import test from 'node:test';
import * as model from '../src/leftDock/model.ts';

function session(id, name = id, overrides = {}) {
  return {
    canonicalSessionId: id,
    canonicalWorkstreamId: 'workstream-a',
    canonicalWorkspaceId: 'workspace-a',
    displayName: name,
    displayAlias: null,
    canonicalDisplayName: name,
    pinned: false,
    presentationOrder: 0,
    archived: false,
    presentationRevision: null,
    runtime: { state: 'unknown', requested: [], observed: [] },
    attention: 'none',
    searchInput: `${name}\n${id}\nworkspace-a`,
    ...overrides
  };
}

function project(id, sessions = [], overrides = {}) {
  const summary = {
    projectViewId: id,
    displayName: id,
    canonicalWorkspaceId: id,
    canonicalRepoRoot: `/repo/${id}`,
    canonicalGitCommonDir: `/repo/${id}/.git`,
    pinned: false,
    presentationOrder: 0,
    collapsed: false,
    presentationRevision: null,
    sessionCount: sessions.length,
    attentionCount: 0,
    attention: 'none',
    searchInput: `${id}\n/repo/${id}`,
    ...overrides
  };
  return { project: summary, sessions, availableWorkstreams: [{ workstreamId: `${id}-ws`, displayName: 'Primary' }] };
}

test('T132 runtime presentation preserves proof state instead of trusting labels', () => {
  const forged = session('session-runtime', 'I am Claude', {
    runtime: { state: 'observed', requested: ['claude'], observed: ['codex'] }
  });
  assert.deepEqual(model.runtimeView(forged), {
    family: 'codex', label: 'Codex observed', proof: 'Winds-observed runtime identity'
  });
  const requested = session('requested', 'Codex says observed', {
    runtime: { state: 'requested', requested: ['claude'], observed: [] }
  });
  assert.equal(model.runtimeView(requested).label, 'Claude requested');
  assert.match(model.runtimeView(requested).proof, /not observed/);
});

test('T132 search renders a collapsed Project expanded when a matching Session is visible', () => {
  const collapsed = project('project-only', [session('session-a', 'Needle Session')], { collapsed: true });
  const filtered = model.filterProjects({ projects: [collapsed] }, 'needle');
  assert.equal(filtered.length, 1);
  assert.equal(filtered[0].sessions.length, 1);
  assert.equal(model.projectRenderedExpanded(filtered[0], 'needle'), true);
  assert.equal(model.projectRenderedExpanded(collapsed, ''), false);
  assert.equal(model.projectRenderedExpanded(collapsed, 'project-only'), false);
});

test('T132 duplicate aliases resolve as ambiguous while exact canonical identity wins', () => {
  const snapshot = { projects: [project('workspace-a', [session('session-a', 'Duplicate'), session('session-b', 'Duplicate')])] };
  assert.deepEqual(model.resolveSessionSearch(snapshot, 'Duplicate'), {
    kind: 'ambiguous', sessionIds: ['session-a', 'session-b']
  });
  assert.deepEqual(model.resolveSessionSearch(snapshot, 'session-b'), {
    kind: 'unique', sessionId: 'session-b'
  });
});

test('T132 request planners preserve exact identity and current revisions', () => {
  const item = session('session-a', 'Alias', {
    displayAlias: 'Alias', canonicalDisplayName: 'Canonical', pinned: true,
    presentationOrder: 40, presentationRevision: 7
  });
  assert.deepEqual(model.sessionRenamePlan(item, 'Renamed'), {
    sessionId: 'session-a',
    displayName: 'Renamed',
    presentation: {
      sessionId: 'session-a', displayAlias: 'Renamed', pinned: true,
      sortOrder: 40, archived: false, expectedRevision: 7
    }
  });
  const owner = project('workspace-a', [item]);
  assert.deepEqual(model.createSessionPlan(owner, 'workspace-a-ws', 'New Session'), {
    workspaceId: 'workspace-a', workstreamId: 'workspace-a-ws', displayName: 'New Session'
  });
});

test('T132 adjacent reorder respects Project pin partitions and tied order', () => {
  const projects = [
    project('a', [], { pinned: true, presentationOrder: 0, presentationRevision: 1 }),
    project('b', [], { pinned: true, presentationOrder: 0, presentationRevision: 2 }),
    project('c', [], { pinned: false, presentationOrder: 0, presentationRevision: 3 })
  ];
  const plan = model.projectReorderPlan(projects, 'b', -1);
  assert.equal(plan.length, 2);
  assert.deepEqual(plan.map((item) => item.workspaceId), ['b', 'a']);
  assert.deepEqual(plan.map((item) => item.expectedRevision), [2, 1]);
  assert.equal(model.projectReorderPlan(projects, 'a', -1), null);
});

test('T132 adjacent Session reorder respects archived and pinned partitions', () => {
  const sessions = [
    session('a', 'A', { pinned: true, presentationOrder: 10, presentationRevision: 1 }),
    session('b', 'B', { pinned: true, presentationOrder: 30, presentationRevision: 2 }),
    session('c', 'C', { pinned: false, presentationOrder: 20, presentationRevision: 3 }),
    session('d', 'D', { pinned: true, archived: true, presentationOrder: 0, presentationRevision: 4 })
  ];
  const plan = model.sessionReorderPlan(sessions, 'b', -1);
  assert.deepEqual(plan.map((item) => [item.sessionId, item.sortOrder, item.expectedRevision]), [
    ['b', 10, 2], ['a', 30, 1]
  ]);
  assert.equal(model.sessionReorderPlan(sessions, 'a', -1), null);
  assert.equal(model.sessionReorderPlan(sessions, 'd', -1), null);
});

test('T132 keyboard row navigation wraps deterministically', () => {
  assert.equal(model.nextSessionFocusIndex(3, 0, -1), 2);
  assert.equal(model.nextSessionFocusIndex(3, 2, 1), 0);
  assert.equal(model.nextSessionFocusIndex(3, -1, 1), 0);
  assert.equal(model.nextSessionFocusIndex(0, 0, 1), null);
});

test('T132 1000-Session fixture keeps search, filter, and selection deterministic', () => {
  const projects = [];
  for (let p = 0; p < 100; p += 1) {
    const sessions = [];
    for (let s = 0; s < 10; s += 1) {
      const id = `session-${String(p).padStart(3, '0')}-${String(s).padStart(2, '0')}`;
      sessions.push(session(id, `Session ${p}-${s}`, { canonicalWorkspaceId: `workspace-${p}` }));
    }
    projects.push(project(`workspace-${p}`, sessions));
  }
  const snapshot = { projects };
  for (let repeat = 0; repeat < 5; repeat += 1) {
    assert.deepEqual(model.resolveSessionSearch(snapshot, 'session-099-09'), {
      kind: 'unique', sessionId: 'session-099-09'
    });
    assert.equal(model.reconcileSelectedSession(snapshot, 'session-099-09'), 'session-099-09');
    const filtered = model.filterProjects(snapshot, 'session-099-09');
    assert.equal(filtered.length, 1);
    assert.equal(filtered[0].sessions.length, 1);
    assert.equal(filtered[0].sessions[0].canonicalSessionId, 'session-099-09');
  }
});
