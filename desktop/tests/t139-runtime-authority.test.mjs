import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import * as dual from '../src/dualSession/model.ts';

const desktopRoot = fileURLToPath(new URL('..', import.meta.url));
const repositoryRoot = join(desktopRoot, '..');

function session(id, state, requested, observed) {
  return {
    canonicalSessionId: id,
    canonicalWorkstreamId: 'workstream-t139',
    canonicalWorkspaceId: 'workspace-t139',
    displayName: id,
    displayAlias: null,
    canonicalDisplayName: id,
    pinned: false,
    presentationOrder: 0,
    archived: false,
    presentationRevision: null,
    runtime: { state, requested, observed },
    attention: 'none',
    searchInput: id,
  };
}

const project = {
  project: {
    projectViewId: 'workspace-t139', displayName: 'T139', canonicalWorkspaceId: 'workspace-t139',
    canonicalRepoRoot: '/repo/t139', canonicalGitCommonDir: '/repo/t139/.git', pinned: false,
    presentationOrder: 0, collapsed: false, presentationRevision: null, sessionCount: 2,
    attentionCount: 0, attention: 'none', searchInput: 't139',
  },
  sessions: [
    session('codex-observed', 'observed', ['codex'], ['codex']),
    session('claude-requested', 'requested', ['claude'], []),
  ],
  availableWorkstreams: [],
};

test('T139 canonical product Sessions keep direct runtime input unavailable', () => {
  const codex = dual.sessionForSlot(project, 'codex-observed', 0);
  const claude = dual.sessionForSlot(project, 'claude-requested', 1);
  assert.equal(codex.runtime.label, 'Codex observed');
  assert.equal(claude.runtime.label, 'Claude requested');
  assert.equal(codex.composerMode, 'unavailable');
  assert.equal(claude.composerMode, 'unavailable');
});

test('T139 UI presents the unavailable path instead of a fake launch control', () => {
  const surface = readFileSync(join(desktopRoot, 'src/sessionSurface/SessionSurface.tsx'), 'utf8');
  const model = readFileSync(join(desktopRoot, 'src/sessionSurface/model.ts'), 'utf8');
  assert.match(surface, /Direct launch unavailable/);
  assert.match(surface, /Direct runtime launch unavailable/);
  assert.match(model, /canonical desktop launch authority is not established/);
  for (const forbidden of ['codex_launch', 'claude_launch', 'runtime_start', 'runtime_input']) {
    assert.equal(surface.includes(forbidden), false, forbidden);
  }
});

test('T139 evidence records exact unauthorized Codex and Claude decisions', () => {
  const evidence = readFileSync(
    join(repositoryRoot, 'docs/provenance/010-t139-desktop-runtime-authority.md'),
    'utf8',
  );
  assert.match(evidence, /DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED/);
  assert.match(evidence, /DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED/);
  assert.match(evidence, /REAL_CLAUDE_EXECUTION=NO/);
  assert.match(evidence, /REAL_CODEX_WORKER_EXECUTION=NO/);
});

test('T139 Tauri host exposes no direct Codex or Claude launch command', () => {
  const host = readFileSync(join(desktopRoot, 'src-tauri/src/main.rs'), 'utf8');
  for (const forbidden of ['codex_launch', 'claude_launch', 'runtime_start', 'runtime_input']) {
    assert.equal(host.includes(forbidden), false, forbidden);
  }
});
