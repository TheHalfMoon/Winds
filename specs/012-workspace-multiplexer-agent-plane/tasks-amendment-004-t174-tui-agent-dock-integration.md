# Spec 012 Tasks Amendment 004 — T174 TUI Agent Dock Integration

Status: CLOSED_CANONICAL

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
SPEC_012_TASKS_AMENDMENT_004=CLOSED_CANONICAL
T173=COMPLETE_CANONICAL
T174=AUTHORIZED
T175..T185=BLOCKED_BY_PREDECESSOR

T174_WORKBENCH_RS_AUTHORITY=CANONICAL_NARROW_RENDER_INPUT_INTEGRATION_ONLY
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

## Canonical closeout

```text
AMENDMENT_PR=277
BASE_SHA=a3268f707cd09259212925e8c02815cb3f1bc7a4
QUALIFIED_HEAD_SHA=453b2f5ad103177111930e12a687120f36839894
MERGE_SHA=4cbe1fea59a6d9164d512c733f95b225a4fe1502
MERGE_TREE=23209d53e43f8e20a497c94faaccd8352d34030e
PRE_MERGE_QUALITY_RUN=36913845958 SUCCESS
POST_MERGE_QUALITY_RUN=36929308847 SUCCESS

JEV=TypeSafe jev-1.13.0 exact range a3268f707cd09259212925e8c02815cb3f1bc7a4..453b2f5ad103177111930e12a687120f36839894
JEV_CLI=0.3.2
JEV_PROVIDER=typesafe
JEV_grants_unbounded_scope=no p=0.07
JEV_claims_unproven_evidence=no p=0.10
JEV_fabricates_platform_or_tooling_claim=no p=0.15
JEV_permanent_self_authorization=no p=0.03
JEV_modifies_code_or_tooling=no p=0.03
JEV_authorizes_new_runtime_authority=no p=0.04
JEV_governance_fit=narrow_correction confidence=1.0
JEV_EXECUTION_NOTE=GitHub Actions lacked TYPESAFE_API_KEY; genuine local credential-backed execution on the exact range is the qualifying Jev evidence. The failed CI credential probe is not treated as PASS.

ALIBABA_OCR=v1.12.9 exact-range accounting
ALIBABA_OCR_REVIEWABLE_COUNT=0
ALIBABA_OCR_EXCLUDED_COUNT=1
ALIBABA_OCR_EXCLUSION=unsupported_ext
ALIBABA_OCR_RULE=resolved
MANUAL_MARKDOWN_REVIEW=CLEAN
INDEPENDENT_EXACT_HEAD_REVIEW=CLEAN
UNRESOLVED_MATERIAL_FINDINGS=0
UNRESOLVED_REVIEW_THREADS=0
ZERO_COST_CONSTRAINT=SATISFIED
MERGE_METHOD=NORMAL_MERGE_COMMIT_WITH_EXPECTED_HEAD_GUARD
```

Amendment 004 is therefore closed canonical. It authorizes only the minimum `src/workbench.rs` TUI Agent Dock render/input integration described above; all broader runtime, process, provider, multiplexer-write, Git, verification, and T175+ authority remains explicitly denied.
