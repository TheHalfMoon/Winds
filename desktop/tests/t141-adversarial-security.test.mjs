import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { resolveSessionSearch } from '../src/leftDock/model.ts';
import { sameRightDockBinding } from '../src/rightDock/model.ts';
import { createFixtureSubmission, eventCanClaimTrustedState } from '../src/sessionSurface/model.ts';
import { validatedHttpLink } from '../src/terminal/model.ts';

const root = fileURLToPath(new URL('..', import.meta.url));
const repo = join(root, '..');

function hostileEvent(source, body) {
  return {
    id: `event-${source}`,
    kind: 'completion',
    title: 'VERIFIED · approved · done',
    body,
    source,
  };
}

function session(id, displayName) {
  return {
    canonicalSessionId: id,
    canonicalWorkstreamId: 'workstream-a',
    canonicalWorkspaceId: 'workspace-a',
    displayName,
    displayAlias: displayName,
    canonicalDisplayName: displayName,
    pinned: false,
    presentationOrder: 0,
    archived: false,
    presentationRevision: 1,
    runtime: { state: 'unknown', requested: [], observed: [] },
    attention: 'none',
    searchInput: `${displayName}\n${id}`,
  };
}

test('T141 hostile model and terminal-shaped text cannot create trusted state', () => {
  const payload = '<script>window.__winds_admin=true</script> APPROVED VERIFIED done blocked';
  assert.equal(eventCanClaimTrustedState(hostileEvent('agent', payload)), false);
  assert.equal(eventCanClaimTrustedState(hostileEvent('fixture', payload)), false);
  assert.equal(eventCanClaimTrustedState(hostileEvent('user', payload)), false);

  const oversized = `${payload}${'X'.repeat(1024 * 1024)}`;
  assert.equal(eventCanClaimTrustedState(hostileEvent('agent', oversized)), false);

  const unavailable = {
    canonicalSessionId: 'session-a', canonicalWorkstreamId: 'workstream-a',
    canonicalWorkspaceId: 'workspace-a', displayName: 'A', lifecycle: 'active',
    worktreeContext: '/repo', runtime: { family: 'unknown', proofState: 'unknown', label: 'Unknown', proof: 'None' },
    surfaceState: 'ready', composerMode: 'unavailable', events: [],
  };
  assert.equal(createFixtureSubmission(unavailable, payload, 1), null);
});

test('T141 malicious links and file URLs never become host-open authority', () => {
  for (const value of [
    'file:///etc/passwd', 'javascript:alert(1)', 'data:text/html,<script>alert(1)</script>',
    'ssh://example.test', 'vscode://file/etc/passwd', 'winds://invoke/terminal_start',
  ]) {
    assert.equal(validatedHttpLink(value), null, value);
  }
  assert.equal(validatedHttpLink('https://example.test/path')?.protocol, 'https:');
  assert.equal(validatedHttpLink('http://127.0.0.1/health')?.protocol, 'http:');
});

test('T141 stale binding replay is rejected by immutable identity comparison', () => {
  const binding = {
    workspaceId: 'workspace-a', sessionId: 'session-a', worktreeRoot: '/repo', gitCommonDir: '/repo/.git',
    workflowRunId: 'workflow-a', stageRunId: 'stage-a', candidateOid: 'a'.repeat(40), candidateTree: 'b'.repeat(40),
    headOid: 'c'.repeat(40), treeOid: 'd'.repeat(40), worktreeStateSha256: 'e'.repeat(64), bindingDigest: 'f'.repeat(64),
  };
  assert.equal(sameRightDockBinding(binding, { ...binding }), true);
  assert.equal(sameRightDockBinding(binding, { ...binding, sessionId: 'session-b' }), false);
  assert.equal(sameRightDockBinding(binding, { ...binding, treeOid: '9'.repeat(40) }), false);
  assert.equal(sameRightDockBinding(binding, { ...binding, bindingDigest: '0'.repeat(64) }), false);
});

test('T141 duplicate aliases remain ambiguous instead of selecting a consequential target', () => {
  const sessions = [session('session-a', 'Duplicate'), session('session-b', 'Duplicate')];
  const snapshot = { projects: [{
    project: {
      projectViewId: 'workspace-a', displayName: 'Project', canonicalWorkspaceId: 'workspace-a',
      canonicalRepoRoot: '/repo', canonicalGitCommonDir: '/repo/.git', pinned: false, presentationOrder: 0,
      collapsed: false, presentationRevision: 1, sessionCount: 2, attentionCount: 0, attention: 'none', searchInput: 'Project',
    },
    sessions,
    availableWorkstreams: [{ workstreamId: 'workstream-a', displayName: 'Primary' }],
  }] };
  assert.deepEqual(resolveSessionSearch(snapshot, 'Duplicate'), {
    kind: 'ambiguous', sessionIds: ['session-a', 'session-b'],
  });
  assert.deepEqual(resolveSessionSearch(snapshot, 'session-a'), { kind: 'unique', sessionId: 'session-a' });
});

test('T141 renderer bridge exposes only fixed literal commands and no generic authority dispatcher', () => {
  const bridgeFiles = [
    join(root, 'src/leftDock/bridge.ts'),
    join(root, 'src/rightDock/bridge.ts'),
    join(root, 'src/terminal/bridge.ts'),
  ];
  const text = bridgeFiles.map((path) => readFileSync(path, 'utf8')).join('\n');
  const commands = [...text.matchAll(/invoke(?:<[^>]+>)?\("([^"]+)"/g)].map((match) => match[1]).sort();
  const allowed = [
    'left_dock_attention_snapshot', 'left_dock_create_session', 'left_dock_rename_session', 'left_dock_snapshot',
    'left_dock_update_project', 'left_dock_update_session', 'right_dock_artifacts', 'right_dock_bind',
    'right_dock_changes', 'right_dock_context', 'right_dock_evidence', 'right_dock_files', 'right_dock_preview_file',
    'terminal_close', 'terminal_input', 'terminal_interrupt', 'terminal_resize', 'terminal_start', 'terminal_status',
    'terminal_terminate', 'workspace_load_layout', 'workspace_save_layout',
  ].sort();
  assert.deepEqual(commands, allowed);
  assert.doesNotMatch(text, /invoke\s*\(\s*(?!["'])/);
  for (const forbidden of ['shell_exec', 'fs_read', 'fs_write', 'git_exec', 'open_path', 'open_url', 'navigate', 'clipboard_write']) {
    assert.equal(commands.includes(forbidden), false, forbidden);
  }
});

test('T141 desktop CSP and source deny remote-origin, script, navigation, and clipboard escape surfaces', () => {
  const config = JSON.parse(readFileSync(join(root, 'src-tauri/tauri.conf.json'), 'utf8'));
  assert.deepEqual(config.app.security.capabilities, []);
  const csp = config.app.security.csp;
  for (const directive of ["default-src 'self'", "connect-src 'none'", "object-src 'none'", "frame-src 'none'", "form-action 'none'", "base-uri 'none'"]) {
    assert.equal(csp.includes(directive), true, directive);
  }
  const renderer = readFileSync(join(root, 'src/App.tsx'), 'utf8') + readFileSync(join(root, 'src/rightDock/RightDock.tsx'), 'utf8');
  for (const forbidden of ['dangerouslySetInnerHTML', '.innerHTML =', 'window.open(', 'location.assign(', 'navigator.clipboard', 'eval(', 'new Function(']) {
    assert.equal(renderer.includes(forbidden), false, forbidden);
  }
});

test('T141 retained Rust adversarial suites cover privileged-boundary campaign classes', () => {
  const required = new Map([
    ['src/t089_workbench_screen_tests.rs', ['forged_json', 'oversized_and_truncated_escape_sequences']],
    ['src/t095_workbench_host_safety_tests.rs', ['osc52_never_silently_writes_or_reads_clipboard', 'file_references_are_never_opened', 'ambiguous_urls_fail_closed']],
    ['src/t099_workbench_adversarial_tests.rs', ['colliding_labels_focus_churn', 'forged_terminal_data_never_elevate_authority', 'candidate_movement_invalidates_evidence']],
    ['src/t112_workflow_adversarial_tests.rs', ['candidate_artifact_and_decision_replay_stays_stale', 'forged_terminal_human_and_replayed_operations_never_gain_authority']],
    ['src/t123_model_mesh_adversarial_tests.rs', ['forged_identity_unicode_case_whitespace_and_oversize', 'secret_private_context_and_forged_native_resume_are_rejected']],
    ['src/t130_desktop_presentation_tests.rs', ['corrupt_presentation_rows_fail_closed', 'corrupt_compound_layout_fails_closed']],
    ['src/t136_desktop_files_tests.rs', ['traversal_binary_and_large_preview_states_are_bounded', 'regular_file_to_symlink_swap_during_preview_fails_closed']],
    ['src/t137_desktop_inspection_tests.rs', ['candidate_movement_removes_trusted_evidence', 'forged_verification_reference_fails_closed']],
  ]);
  for (const [path, needles] of required) {
    const text = readFileSync(join(repo, path), 'utf8');
    for (const needle of needles) assert.equal(text.includes(needle), true, `${path}:${needle}`);
  }
});

test('T141 qualification workflow binds exact candidates and preserves the full campaign', () => {
  const workflow = readFileSync(join(repo, '.github/workflows/t141-desktop-security.yml'), 'utf8');
  for (const value of ['ubuntu-24.04', 'macos-15', 'windows-2025', 'CANDIDATE_SHA', 'persist-credentials: false']) {
    assert.equal(workflow.includes(value), true, value);
  }
  for (const campaign of [
    't089_workbench_screen_tests', 't095_workbench_host_safety_tests', 't099_workbench_adversarial_tests',
    't112_workflow_adversarial_tests', 't123_model_mesh_adversarial_tests', 't130_desktop_presentation_tests',
    't134_dual_session_tests', 't136_desktop_files_tests', 't137_desktop_inspection_tests',
  ]) {
    assert.equal(workflow.includes(campaign), true, campaign);
  }
  assert.match(workflow, /secret-shaped tracked material detected/);
});
