# Spec 012 Formal Entry Gate — Workspace Multiplexer & Agent Plane

**Status:** Documentation-only governance candidate. This file authorizes no Plan, Tasks, implementation, dependency, migration, donor-code copy, remote transport, plugin runtime, updater, or Git landing by itself.

## 1. Purpose

This gate decides whether Winds may create a formal Spec 012 specification for the Founder-directed next product slice after canonical Spec 011 closeout.

The candidate product problem is:

> Build a Winds-native workspace multiplexer and agent plane over the already accepted desktop and persistent-runtime foundations, with exact workspace/tab/pane identity, broad agent detection, explicit lifecycle truth, bounded automation, worktree-aware workflows, and compact human control—without silently importing remote-machine, plugin-marketplace, updater, or automatic-Git authority.

This gate may authorize **only** creation of a separate:

```text
specs/012-workspace-multiplexer-agent-plane/spec.md
```

It does not authorize the specification contents in advance and does not authorize Plan, Tasks, source implementation, dependencies, migrations, remote execution, plugin installation, generic integrations, updater behavior, or direct/adapted Herdr source reuse.

## 2. Canonical predecessor

Spec 011 closed canonically at:

```text
T161_HEAD=cc4f99fdb6dc96beeb54180406cf66bccc1b186e
T161_HEAD_TREE=76e059cb15638f2013580a04e2a00282a4e857e2
T161_MERGE=bdc30eb34dea2d89213aebabb1bccdf2d5eb5edd
T161_MERGE_TREE=76e059cb15638f2013580a04e2a00282a4e857e2
POST_MERGE_QUALITY_RUN=35510161683
POST_MERGE_QUALITY=SUCCESS
```

Canonical repository truth is:

```text
T146..T161=CLOSED_CANONICAL
SPEC_011_ENTRY=CLOSED_CANONICAL
SPEC_011_SPEC=CLOSED_CANONICAL
SPEC_011_PLAN=CLOSED_CANONICAL
SPEC_011_TASKS=CLOSED_CANONICAL
SPEC_011_FIRST_PERSISTENT_RUNTIME_PROGRAM=CLOSED_CANONICAL

SPEC_011_WSL_PERSISTENT_OWNER=NOT_CLAIMED
WINDOWS_HOST_TO_WSL_OWNER_BRIDGE=NOT_INTRODUCED
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO
```

Spec 012 must treat the accepted Spec 011 local owner/control plane as an upstream authority boundary. It may consume that boundary; it may not silently broaden it.

## 3. Current Herdr research pin

A fresh upstream reconciliation for this gate binds:

```text
HERDR_REPOSITORY=herdrdev/herdr
HERDR_DEFAULT_BRANCH=master
HERDR_PIN=29f9f4056f344af60f411004fc89c7eb5f357c48
HERDR_TREE=8e11bddc337094b8aece2a7abea4ee6ee6340f2a
HERDR_PACKAGE_VERSION=0.9.1
HERDR_ROOT_LICENSE=Apache-2.0
HERDR_PIN_GITHUB_COMMIT_VERIFICATION=VERIFIED_VALID
HERDR_PIN_GITHUB_VERIFICATION_REASON=valid
HERDR_PIN_GITHUB_VERIFIED_AT=2026-09-20T12:23:58Z
HERDR_PIN_GITHUB_COMMITTER=GitHub <noreply@github.com>
HERDR_PIN_GITHUB_COMMIT_URL=https://github.com/herdrdev/herdr/commit/29f9f4056f344af60f411004fc89c7eb5f357c48
HERDR_PIN_GITHUB_VERIFIED_TREE=8e11bddc337094b8aece2a7abea4ee6ee6340f2a
HERDR_PIN_GITHUB_VERIFIED_PARENT=a3a1c94ed54e8a65d929528336d69c7e537ed2ef
```

GitHub's exact commit-verification response for this pin reports `verified=true`, `reason=valid`, and `verified_at=2026-09-20T12:23:58Z`. The signed payload binds the exact tree `8e11bddc...`, parent `a3a1c94e...`, author identity, and GitHub committer identity shown above. Winds records GitHub's verification result and payload binding; it does not invent a separate cryptographic signer identity beyond the metadata GitHub exposes.

The immediately previous Winds research pin was:

```text
da6bcd5969779bfe0396bcf89a8025d4375d611e
```

The current pin is 14 commits ahead with zero reverse divergence.

An intermediate pin `a3a1c94ed54e8a65d929528336d69c7e537ed2ef` was observed while this gate was being authored. Upstream advanced once more before qualification; that intermediate observation is superseded and is not used as final entry authority.

Current source-derived inventory:

```text
AGENT_FAMILIES=24
FROZEN_SERIALIZED_INTEGRATION_TARGETS=17
EXPERIMENTAL_CLI_ONLY_INSTALLABLE_TARGETS=1
EXPERIMENTAL_TARGET=letta
PUBLIC_SERIALIZED_API_METHODS=105
ADDITIONAL_RUNTIME_UI_TRANSPORT_CONFIG_PLUGIN_PACKAGING_ROWS=72
HERDR_BASELINE_CAPABILITY_ROWS=219
```

The omission-prevention ledger is:

```text
docs/research/021-herdr-exhaustive-capability-ledger.md
```

and is pinned to the same exact upstream commit/tree.

Founder permission to use Herdr source remains recorded as `FOUNDER_PERMISSION_ASSERTION`. It does not waive Apache-2.0 notice/copyright obligations or independent review of vendored, generated, patched, or third-party material.

## 4. Upstream movement since the previous audit

The 14-commit delta is material and therefore cannot be silently inherited.

Material Spec 012-facing movement includes:

- Grok activity detection under custom or disabled OSC signaling;
- Kiro status detection from live controls and OSC signals;
- Codex status detection under custom/remapped interrupt-key hints;
- agent-detection tests refactored toward engine contracts;
- the new public `pane.clear` method plus keybinding/config path;
- aggregate navigation exposing every agent and terminal in the go-to picker;
- native-Windows cursor redraw stabilization and input-origin qualification;
- request-id preservation in socket error responses.

Material movement also includes SSH compression and remote clipboard-image qualification. Those changes are recorded for later Spec 013 and do **not** expand Spec 012 authority.

The refreshed ledger appends `M105 pane.clear` rather than renumbering historical method IDs.

## 5. Candidate Spec 012 scope envelope

A future formal Specification may define requirements only inside this envelope.

### 5.1 Workspace / tab / pane topology

Candidate requirements may cover:

- workspace creation/list/get/focus/rename/move/close;
- tab create/list/get/focus/rename/move/close;
- recursive pane topology;
- split/swap/move/resize/zoom/focus/close;
- pane neighbors/edges/directional focus;
- layout export/apply/split ratios;
- saved layouts;
- pointer/drag/keyboard operations;
- pane clear behavior and scrollback semantics;
- deterministic identity and stale-target rejection.

### 5.2 Terminal UX inside the accepted local authority plane

Candidate requirements may cover:

- terminal profiles/default shell policy;
- scrollback read/edit/copy/search/selection;
- link resolve/activate;
- mouse routing/copy-on-select/scroll tuning;
- CJK/IME-facing input behavior;
- pane borders/gaps/scrollbars/labels;
- terminal title/theme/effects where they remain presentation;
- platform-specific input qualification.

No second terminal backend is implied. Accepted PTY/ConPTY ownership remains upstream.

### 5.3 Agent detection and agent plane

Candidate requirements may cover:

- all 24 source-observed agent detection families;
- manifest-driven detection and safe refresh;
- agent state/lifecycle presentation;
- agent list/get/read/explain/focus/rename/view;
- exact pane/session association;
- explicit detection confidence/source truth;
- `Needs You` aggregation and attention rollups;
- compact Founder agent dock;
- install/launch presentation for **explicitly qualified** agents without generic integration-install authority.

Detection does not prove real provider execution. Provider/model/runtime/session identities remain separate.

### 5.4 Structured local automation

Candidate requirements may cover bounded local workspace/pane/agent actions such as:

- exact-target pane navigation and layout mutation;
- exact-target text/key/input dispatch through accepted controller authority;
- agent prompt/wait/start requirements only where later Tasks separately prove and authorize the execution seam;
- command palette/custom command presentation where the invoked operation is already typed/allowlisted;
- zero hidden broadcast authority;
- deterministic request/result correlation.

The formal Specification must not turn this into a generic RPC, arbitrary shell dispatcher, plugin method bus, or remote command plane.

### 5.5 Worktree-aware workflows

Candidate requirements may cover:

- worktree discovery/create/open/remove UX;
- explicit repository trust;
- binding workspace/pane/agent state to exact repository/worktree identity;
- removal safety and stale-target protection.

This does not authorize merge, rebase, cherry-pick, push, PR creation, automatic landing, or bypass of existing Git governance.

## 6. Explicitly excluded successor domains

The following remain outside Spec 012 unless a later canonical amendment explicitly changes the boundary.

### Spec 013 — Remote Machines & Thin Clients

Not authorized here:

- SSH remote-control transport;
- saved remote machines as an implementation feature;
- thin clients;
- remote owner setup/update;
- remote clipboard/image/file bridging;
- cross-machine session control;
- remote reconnect supervision.

### Spec 014 — Integration, Plugin & Marketplace Platform

Not authorized here:

- generic integration install/uninstall/status lifecycle;
- plugin registry/runtime;
- GitHub plugin install;
- plugin actions/events/panes/link handlers;
- marketplace/catalog;
- generic third-party execution.

The 17 frozen integration targets plus experimental Letta remain parity scope assigned to Spec 014, not implementation authority for Spec 012.

### Spec 015 — Update, Distribution & Final Parity

Not authorized here:

- updater channels;
- live binary handoff;
- release promotion;
- installers/package management;
- generic update/version orchestration.

## 7. Inherited Winds invariants

A Spec 012 specification candidate must preserve at minimum:

```text
RUNTIME_NAMESPACE_ID != RUNTIME_ALIAS
OWNER_GENERATION_ID != OS_PID
RUNTIME_NAMESPACE_ID != CANONICAL_WINDS_SESSION
RUNTIME_NAMESPACE_ID != PROVIDER_NATIVE_SESSION_ID
RUNTIME_NAMESPACE_ID != GIT_CANDIDATE_OR_EVIDENCE_ID

AGENT_REPORTED != WINDS_OBSERVED != HUMAN_DECIDED
REPLAYED_OUTPUT != CANONICAL_EVIDENCE
PROCESS_EXIT != VERIFIED
DIFF_RENDERED != VERIFIED
DONE != VERIFIED
```

Additional inherited boundaries:

- no PID-as-authority or blind process reconstruction;
- no raw terminal/agent text as trusted control or evidence;
- no implicit controller promotion;
- no multi-runtime input/control broadcast;
- no generic renderer/WebView privileged dispatcher;
- no public network control plane;
- no same-effective-user sandbox claim;
- no secret/full-environment persistence merely for continuity;
- no automatic Git landing;
- no direct Herdr code admission without exact later Tasks provenance.

Inherited runtime/provider nonclaims remain explicit unless a later exact governance chain proves otherwise:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED
DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED
```

## 8. Entry questions the formal Specification must answer

A future Spec 012 specification candidate must answer, without implementation:

1. What is the immutable workspace/tab/pane identity model, and how does it relate to canonical Project/Session/runtime/worktree identities?
2. What topology mutations are supported, and what stale-target/conflict semantics apply?
3. What pane/agent actions are observer-safe versus controller-only?
4. How are pane input, agent prompt, and automation prevented from becoming broadcast or generic command execution?
5. What are the exact agent detection evidence classes and false-positive/false-negative states?
6. How is `Needs You` derived without turning agent-reported text into trusted lifecycle truth?
7. Which of the 24 detected agent families are presentation/detection only, and which later execution seams, if any, may become separately qualified?
8. How do terminal profiles, scrollback, clear/copy/selection/link/IME behavior preserve accepted PTY/ConPTY ownership and bounded memory?
9. How do worktree operations preserve repository trust and avoid acquiring Git landing authority?
10. Which Herdr parity rows are in Spec 012, and which remain assigned to Specs 013–015?
11. What direct platform claims require native macOS/Linux/Windows evidence?
12. What migration/dependency additions, if any, would require later Plan/Tasks qualification?
13. What YAGNI boundary prevents a workspace multiplexer from becoming a generic remote/plugin/orchestration platform?
14. What acceptance criteria prove exact-target behavior under multi-pane/multi-agent scale and stale identities?

## 9. Donor and reuse firewall

This gate admits no donor code.

```text
HERDR_DIRECT_COPY_AUTHORIZED=NO
HERDR_ADAPTED_COPY_AUTHORIZED=NO
HERDR_TEST_PORT_AUTHORIZED=NO
NEW_DEPENDENCY_AUTHORIZED=NO
NEW_MIGRATION_AUTHORIZED=NO
```

A future Task may admit a direct/adapted/test-port slice only if that exact Task records:

- exact upstream repository/commit/tree/path;
- source license/copyright/NOTICE obligations;
- vendor/third-party provenance;
- selected reuse mode;
- Winds destination;
- modifications;
- authority/threat-model delta;
- deterministic tests;
- native-platform requirements where applicable;
- update/removal path.

Founder permission is necessary context but not sufficient admission evidence.

## 10. Current authority before landing

```text
SPEC_011_FIRST_PERSISTENT_RUNTIME_PROGRAM=CLOSED_CANONICAL

SPEC_012_ENTRY_GATE=IN_QUALIFICATION
SPEC_012_FORMAL_SPEC_AUTHORIZED=NO
SPEC_012_PLAN_AUTHORIZED=NO
SPEC_012_TASKS_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO

REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_BINARY_HANDOFF_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

## 11. Entry-gate acceptance requirements

This documentation-only candidate may close only when one exact final head proves:

- canonical base is the post-merge-qualified T161 closeout or a governance-only forward descendant;
- changed scope is limited to the refreshed Herdr research/ledger and this Entry gate;
- upstream Herdr head is rechecked immediately before final qualification;
- any movement from `29f9f405...` is classified and, if material, reconciled before landing;
- 24 agent families, 17 frozen integrations + 1 experimental Letta target, 105 public methods, and 219 total ledger rows remain exact or are forward-updated with evidence;
- no Plan, Tasks, implementation, dependency, migration, remote, plugin, update, automatic-Git, or donor-copy authority is introduced;
- repository `quality` succeeds on the exact final head;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review reports zero material findings;
- zero unresolved material review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation immediately before landing;
- guarded normal merge with exact expected-head binding;
- merge tree/ordered parents/GitHub signature reconciled;
- every actually-triggered post-merge push workflow succeeds.

## 12. Post-landing authority state

Only after this exact Entry gate lands canonically and completes post-merge verification may repository truth state:

```text
SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_FORMAL_SPEC_AUTHORIZED=YES

SPEC_012_PLAN_AUTHORIZED=NO
SPEC_012_TASKS_AUTHORIZED=NO
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO

HERDR_DIRECT_COPY_AUTHORIZED=NO
HERDR_ADAPTED_COPY_AUTHORIZED=NO
HERDR_TEST_PORT_AUTHORIZED=NO

REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
LIVE_BINARY_HANDOFF_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

The next authorized repository unit after canonical Entry closeout is the **Spec 012 formal Specification candidate only**.
