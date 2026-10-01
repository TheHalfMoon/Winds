# Spec 012 Tasks Amendment 004 — T174 TUI Agent Dock Integration

Status: IN_QUALIFICATION

Authority basis: canonical Spec 012 T174; canonical T168 TUI topology projection; canonical T169 Desktop topology projection; canonical T173 owner-authoritative agent observation projection; and Spec 012 Tasks Amendment 003 precedent for correcting an omitted production TUI integration path without expanding runtime authority.

## Material contradiction

Canonical T174 requires compact exact-identity agent list/get/read/explain/view/focus/rename presentation in both TUI and Desktop, including keyboard and pointer focus paths. Its accepted path list permits `src/workbench_ui.rs` / projection support narrowly, `src/desktop.rs` trusted bridge support, the Desktop Agent Dock extension, and focused tests.

The existing TUI production renderer is owned by `src/workbench.rs`. `src/workbench_ui.rs` owns navigation/event projection, but a projection-only implementation cannot make the Agent Dock visible in the actual TUI or prove a real keyboard/pointer path reaches that presentation. Avoiding `src/workbench.rs` would therefore weaken T174 into model-only coverage and would not satisfy the authorized TUI requirement.

This is a narrow authorized-path omission. It does not authorize a redesign, a second TUI architecture, process/provider behavior, runtime controller authority, MultiplexerWrite authority, topology mutation, Git authority, verification authority, or any T175+ behavior.

## Narrow correction

T174 additionally permits `src/workbench.rs` only for the minimum production integration needed to render and route the already-authorized T174 Agent Dock presentation.

The correction permits only:

- render a compact Agent Dock derived from trusted owner-authoritative observations already projected through T173/T174 types;
- expose visible exact immutable bindings, source/freshness, and the literal `DETECTION_ONLY_UNPROVEN` limitation;
- route keyboard and pointer selection/view/focus to an exact `AgentObservationId` and its immutable `MultiplexerWorkspaceId` / `TabId` / `PaneId` binding;
- use the existing read-only topology binding/validation path for focus/navigation and refuse stale, absent, ambiguous, or substituted targets;
- keep filtering, sorting, grouping, and display aliases presentation-only;
- keep material Needs You state visible independently of Agent Dock presentation filtering/sorting/grouping;
- register or connect no new authority-bearing module beyond the minimum existing TUI integration seam.

The correction does **not** authorize:

- `agent.start`, provider launch/install/action, provider process control, shell dispatch, terminal input, or arbitrary command execution;
- Runtime Controller acquisition, MultiplexerWrite acquisition, or `ApplyTopologyOperationV2` mutation initiated by Agent Dock focus;
- changing `AgentObservationId`, `PaneId`, `TabId`, `MultiplexerWorkspaceId`, `RuntimeNamespaceId`, `OwnerGenerationId`, provider-native session identity, or detection truth during rename/view/focus;
- alias/label/ordinal/focus/geometry-based target resolution;
- terminal/model prose as trusted observation, Needs You, verification, acceptance, or authority evidence;
- changes to T173 detector/catalog/projection semantics except narrowly consumed presentation support already authorized by T174;
- any change to the T168 canonical topology presentation, topology hit-region geometry, topology pointer-focus intent shape, or topology pointer-capture semantics;
- any new dependency, network service, daemon, public RPC, migration, protocol schema, Git behavior, verification behavior, or T175+ implementation.

If implementation proves that another production path or authority is required, T174 must stop and obtain a separate accepted amendment rather than widening this correction implicitly.

## Required qualification

This amendment is canonical only after exact-head qualification demonstrates:

1. the contradiction exists in the live canonical code: `src/workbench.rs` owns the production TUI render path while `src/workbench_ui.rs` alone cannot render the required Agent Dock;
2. the amendment changes exactly this Markdown file and no source, dependency, workflow, protocol, migration, or runtime behavior;
3. repository quality succeeds on the exact candidate head;
4. author correctness/safety/governance/evidence-integrity review finds no authority expansion beyond the narrow T174 render/input integration;
5. Ponytail/YAGNI review confirms the smallest sufficient correction;
6. genuine Jev review executes against the exact candidate/range; no unavailable Jev result is inferred or fabricated;
7. Alibaba Open Code Review executes exact-head/range delegation or rule accounting; if Markdown is truthfully classified unsupported, that result is recorded and manual Markdown review remains separately required;
8. fresh independent exact-head review has zero unresolved material findings/threads;
9. immediate pre-landing base/head/tree/scope/mergeability reconciliation succeeds;
10. landing uses a guarded expected-head normal merge; and
11. every actually-triggered post-merge workflow succeeds.

Candidate movement invalidates stale candidate-bound evidence.

## Authority state

```text
SPEC_012_TASKS_AMENDMENT_004=IN_QUALIFICATION
T173=COMPLETE_CANONICAL
T174=AUTHORIZED_BUT_TUI_WORKBENCH_RS_CHANGE_BLOCKED_PENDING_AMENDMENT_004
T175..T185=BLOCKED_BY_PREDECESSOR

T174_WORKBENCH_RS_AUTHORITY=PROPOSED_NARROW_RENDER_INPUT_INTEGRATION_ONLY
AGENT_START_AUTHORIZED=NO
PROVIDER_LAUNCH_INSTALL_ACTION_AUTHORIZED=NO
RUNTIME_CONTROLLER_AUTHORIZED=NO
MULTIPLEXER_WRITE_AUTHORIZED=NO
TOPOLOGY_MUTATION_AUTHORIZED=NO
GIT_AUTHORITY_AUTHORIZED=NO
VERIFICATION_AUTHORITY_AUTHORIZED=NO
NEW_DEPENDENCY_AUTHORIZED=NO
T175_PLUS_BEHAVIOR_AUTHORIZED=NO
```
