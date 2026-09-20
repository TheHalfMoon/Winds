# Tasks: Workspace Multiplexer & Agent Plane

## Canonical Inputs

```text
SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_SPEC=CLOSED_CANONICAL
SPEC_012_PLAN=CLOSED_CANONICAL
SPEC_012_TASKS=IN_QUALIFICATION
SPEC_012_IMPLEMENTATION_AUTHORIZED=NO

ENTRY_MERGE=ec214544d94d90b23fe4a0156d0040a2faf0622c
ENTRY_POST_MERGE_QUALITY=35513773204 SUCCESS

SPEC_MERGE=27dd3e88255abd1184c8ca1e54cd671f20e058fd
SPEC_TREE=ec0d9a525502bbb0609f3fe38a3f86871f3b92c9
SPEC_POST_MERGE_QUALITY=35517252780 SUCCESS

PLAN_MERGE=7070b6a1faf0aab308d03a4a92a514ab0802f872
PLAN_TREE=be687ae73e53487e01b4d6a1013357e0133fc77f
PLAN_POST_MERGE_QUALITY=35519723712 SUCCESS

HERDR_TASKS_CREATION_PIN=f10df75c8a5f1b5e5f90689c3a8ceb6758366e05
HERDR_TASKS_CREATION_TREE=3fa0fa2a9f48cf1cf1f0820cfab94abda63b8549
HERDR_TASKS_CREATION_MESSAGE=fix: reject terminal-less attach before starting a session (#4395)

AGENT_FAMILIES=24
SPEC_012_FUNCTIONAL_REQUIREMENTS=92
SPEC_012_SUCCESS_CRITERIA=30
SPEC_012_LEDGER_ROWS=142
```

Herdr movement through the Tasks creation pin is already reconciled by the canonical Plan: explicit worktree membership survives discovery drift, event-history loss is fail-explicit with authoritative resnapshot recovery, and required client capability is preflighted before avoidable side effects. Herdr remains research/test-design evidence only; no Herdr source is admitted by this Tasks file.

## Global Rules

1. Live canonical repository truth overrides this Tasks file if `main`, accepted dependencies, platform evidence, or Herdr research moves.
2. Each implementation task starts from then-current exact canonical `main` only after its predecessor is `CLOSED_CANONICAL` with successful post-merge verification.
3. Canonical acceptance of this Tasks file authorizes **T162 only**. T163..T185 remain blocked by their immediate predecessor.
4. No force-push, rebase, shared-history rewrite, rerun-to-green, stale-head review reuse, hidden failure suppression, or fabricated CI/test/review/platform/runtime evidence.
5. Candidate movement invalidates candidate-bound CI, security, performance, platform, and review evidence unless a reviewer explicitly covers the successor range.
6. `MultiplexerWorkspaceId != GitWorkspaceId != Project != Session != Workstream != RuntimeNamespaceId != terminal ID != provider-native session != candidate/evidence ID != OS PID/path/label/focus`.
7. `PaneId` is immutable for the pane lifetime and is never reused for a replacement pane. `TopologyGeneration` is ordering/staleness proof, not pane identity.
8. `MultiplexerWrite != Runtime Controller`. Neither authority implies the other and neither survives owner reconnect/generation implicitly.
9. Every consequential runtime action targets exactly one immutable PaneId + RuntimeNamespaceId under current controller authority. Implicit multi-pane/agent broadcast is prohibited.
10. Every topology mutation carries exact target IDs plus expected `TopologyGeneration`; stale mutation fails closed and never retargets by alias/ordinal/focus/geometry.
11. Terminal/agent/renderer content is untrusted data and can never become protocol, detection, Needs You, verification, approval, acceptance, trust, or Git authority merely by matching text.
12. Spec 012 extends the existing Spec 011 persistent owner and private transport only. No second daemon/server, TCP/HTTP/WebSocket/public RPC, SSH/cloud relay, or renderer-direct owner connection.
13. Protocol v2 does not silently downgrade. A live v1 owner yields explicit `BLOCKED_LEGACY_OWNER`; no auto-kill, PID reclaim, live handoff, or second owner.
14. Existing PTY/ConPTY ownership remains authoritative. No second terminal backend or pane-owned process handle/PID authority.
15. `DETACH_VIEW` and `STOP_RUNTIME_THEN_CLOSE` remain distinct. No implicit terminate-on-close or orphan cleanup.
16. `pane.clear` is presentation-only and sends no child input or verification/evidence/process transition.
17. Saved layout templates are authority-free and applying them creates fresh topology IDs and no process.
18. The first program uses a static compile-time data-only 24-family detector catalog. No dynamic plugin, downloaded manifest, hot reload, arbitrary code loading, or marketplace.
19. `agent.start` remains prohibited. T176 may admit only exact-pane prompt/send-keys/structured wait if all authority tests pass; failure to prove safety means those behaviors remain unadmitted.
20. Needs You derives only from accepted structured Winds/runtime state. Terminal prose, punctuation, titles, model claims, or regex output cannot create trusted attention.
21. Event/history loss invalidates cached projections and requires fresh subscription + authoritative snapshot; buffered events are never blindly replayed across an unproven snapshot boundary.
22. System Git remains the only Git authority. No libgit2/alternate Git implementation is introduced.
23. Repository trust is explicit and repository-scoped. Discovery cannot create trust.
24. Explicit worktree membership is user-owned metadata; discovery drift cannot silently erase it. Missing members become stale until explicit resolution.
25. Worktree create/remove never uses `--force`, implicit fetch/pull, branch overwrite, or recursive-delete fallback.
26. No Spec 012 task may merge, rebase, cherry-pick, push, create/approve PRs, or automatically land code.
27. Migration `0014_workspace_multiplexer.sql` is owned only by T164. Migrations 0011/0012/0013 remain immutable.
28. No new direct dependency is planned. Any later necessity requires exact task-stage version/license/MSRV/platform/security/transitive/removal proof before admission.
29. Linux, macOS, native Windows, and WSL are distinct evidence domains; no platform claim substitutes for another.
30. Same-effective-user execution is not a sandbox claim.
31. Interactive client/surface capability is preflighted before avoidable topology/runtime/persistence/filesystem side effects; TUI/CLI TTY capability is distinct from trusted Desktop terminal-surface capability.
32. Herdr is research/test-design evidence only. No direct/adapted copy or test port is authorized by this Tasks file.
33. Failed/rejected/superseded candidates and failed workflows remain historical evidence and are never relabelled.
34. Desktop-touching implementation tasks must pass exact-head `desktop-quality` in addition to repository `quality`; native-Windows/platform tasks must pass applicable exact platform workflows.
35. The program may close only after T184/T185 reconcile all 92 FR, 30 SC, and 142 Spec 012 ledger rows with exact evidence or truthful explicit disposition.

## Standard Acceptance Gate

Every implementation-bearing task must satisfy on its exact final candidate:

- task-focused deterministic tests PASS;
- `git diff --check` PASS;
- `cargo fmt --all -- --check` PASS for Rust-touched candidates;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` PASS for Rust-touched candidates;
- complete applicable repository tests under `quality` PASS;
- exact-head `desktop-quality` PASS for any `desktop/**` or Tauri-host change;
- applicable `windows-terminal`, `t160-native-platform`, or task-owned native/security/performance workflow PASS when that task touches or claims those domains;
- no unrelated source/dependency/migration/workflow mutation;
- exact source/dependency/license/security provenance when a task changes those truths;
- author correctness/safety/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact final candidate or explicitly scoped successor range with zero unresolved material findings;
- zero unresolved review threads;
- exact base/head/tree/scope/ruleset/mergeability and current Herdr freshness reconciliation immediately before landing;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered post-merge push workflow succeeds.

Documentation/governance-only candidates use the same exact identity/review/landing discipline and repository `quality` without inventing irrelevant runtime tests.

No task may weaken a predecessor's accepted security/resource/identity boundary merely to make its own tests pass.

## Dependency Order

```text
T162 -> T163 -> T164 -> T165 -> T166 -> T167 -> T168 -> T169
     -> T170 -> T171 -> T172 -> T173 -> T174 -> T175 -> T176
     -> T177 -> T178 -> T179 -> T180 -> T181 -> T182 -> T183
     -> T184 -> T185
```

Canonical acceptance and successful post-merge verification of this Tasks file authorize **T162 only**.

---

## Phase 1 — Domain, Identity, Authority, and Error Contract

### [ ] T162 — Freeze Spec 012 identity domains, topology-generation contract, write authority, capability preflight, and closed error vocabulary

**Purpose**: Define the complete multiplexer authority vocabulary before persistence, owner integration, protocol v2, UI, detection, or Git mutation exists.

**Authorized paths**:
- new `src/multiplexer/mod.rs` and `src/multiplexer/domain.rs` only;
- `src/lib.rs` only for module/test registration;
- focused `src/t162_multiplexer_domain_tests.rs`;
- focused provenance/design evidence under `docs/provenance/`;
- no migration, protocol version change, owner/service mutation, desktop/TUI behavior, Git mutation, detector, dependency, or workflow-semantic change;

**Required work**:
- opaque immutable `MultiplexerWorkspaceId`, `TabId`, `PaneId`, `TopologyGeneration`, `LayoutTemplateId`, and `AgentObservationId` types using the already-qualified Spec 011 OS entropy seam where random identity is required;
- explicit separation from existing Git Workspace ID, Project, Session, Workstream, RuntimeNamespaceId, terminal ID, provider-native session ID, candidate/evidence IDs, OS PID, path, label, ordinal, focus, and geometry;
- closed `MultiplexerWrite` authority vocabulary that is non-transitive with Runtime Controller authority;
- closed topology error vocabulary including stale generation, unknown/closed target, duplicate/replayed mutation, snapshot limit exceeded, capability unavailable, unsupported operation/platform, ownership loss, and outcome unknown where applicable;
- closed client capability vocabulary distinguishing TUI/CLI controlling-terminal requirements from trusted Desktop terminal-surface capability;
- type-level prohibition on generic ambiguous `workspace_id` in agent observation domain;

**Acceptance**:
- duplicate aliases/labels cannot create identity equivalence;
- replacement panes cannot reuse closed PaneId values;
- TopologyGeneration cannot be mistaken for PaneId/RuntimeNamespaceId/owner generation;
- MultiplexerWrite never implies Runtime Controller and Runtime Controller never implies MultiplexerWrite;
- no process, database migration, endpoint, UI, Git mutation, detector, or new dependency exists after T162;

**Primary FR coverage**: FR-001..FR-005, FR-013, FR-043, FR-055..FR-056, FR-063, FR-087.

**Primary SC coverage**: SC-001, SC-002.

**Closes to authorize**: T163.

---

## Phase 2 — Pure Topology State Machine and Navigation

### [ ] T163 — Implement deterministic workspace/tab/pane topology mutations and directional navigation over immutable IDs

**Purpose**: Prove topology semantics as a pure deterministic state machine before persistence or owner/service integration.

**Authorized paths**:
- `src/multiplexer/domain.rs` and new `src/multiplexer/navigation.rs`;
- focused `src/t163_multiplexer_topology_tests.rs`;
- no Store/migration, owner, protocol, terminal runtime, desktop, Git, detector, or dependency change;

**Required work**:
- workspace create/list/get/focus/rename/move/close with immutable identity;
- tab create/list/get/focus/rename/move/close with immutable identity;
- pane split/swap/move/resize/zoom/focus/close topology semantics;
- expected-TopologyGeneration compare-and-apply for every consequential topology mutation;
- stale-target failure with zero alias/ordinal/focus/nearest-geometry retargeting;
- integer split ratios and deterministic geometry derivation;
- directional/neighbor/edge navigation with deterministic stable tie-breaking;
- `DETACH_VIEW` topology close semantics only; no process stop in this task;
- layout-template structure types remain authority-free and process-free;

**Acceptance**:
- same initial topology + same operation sequence yields byte-for-byte equivalent normalized topology;
- stale close/split/move against a replaced pane is rejected;
- duplicate labels do not affect exact selection;
- split creates a fresh unbound PaneId and starts no process;
- no persistence, IPC, process mutation, or UI is introduced;

**Primary FR coverage**: FR-006..FR-015, FR-056.

**Primary SC coverage**: SC-001..SC-005, SC-021.

**Closes to authorize**: T164.

---

## Phase 3 — Versioned Persistence and Migration 0014

### [ ] T164 — Add bounded TopologySnapshotV1/LayoutTemplateV1 persistence and migration 0014 without live authority restoration

**Purpose**: Persist only presentation/topology and explicit membership/trust metadata while stripping live process/controller authority.

**Authorized paths**:
- new `migrations/0014_workspace_multiplexer.sql` only;
- `src/store.rs` narrowly;
- new `src/multiplexer/persistence.rs`;
- `src/multiplexer/domain.rs` only for serialization validators;
- focused `src/t164_multiplexer_persistence_tests.rs`;
- migration/provenance evidence;
- no owner protocol/client/UI/Git mutation/detector/dependency change;

**Required work**:
- `multiplexer_workspaces`, `multiplexer_layout_templates`, `multiplexer_worktree_memberships`, and repository-trust persistence exactly as justified by Plan, or a strictly smaller schema preserving all Plan invariants;
- versioned bounded TopologySnapshotV1 and LayoutTemplateV1;
- layout templates omit MultiplexerWorkspaceId/TabId/PaneId/RuntimeNamespaceId/controller/provider/evidence authority and generate fresh IDs on apply;
- explicit worktree membership source/state preserved independently from discovery;
- repository trust keyed by canonical RepositoryIdentity plus monotonic revision/equivalent exact snapshot;
- no secrets/full environment/PID/controller lease/transcript/canonical evidence persisted;
- corrupt/oversized/unknown-version records quarantine rather than silently reinitialize or truncate;
- migrations 0011/0012/0013 remain byte-identical;

**Acceptance**:
- fresh/migrated/restart/corrupt/oversized/template round-trip fixtures pass;
- saved layout cannot recreate live ownership or controller authority;
- removing layout/UI metadata does not delete Git/worktree content;
- 0014 is the only new migration;

**Primary FR coverage**: FR-010..FR-012, FR-073, FR-091.

**Primary SC coverage**: SC-004.

**Closes to authorize**: T165.

---

## Phase 4 — Owner Multiplexer Service

### [ ] T165 — Integrate serialized topology authority and MultiplexerWrite state into the existing Spec 011 owner without new transport

**Purpose**: Make the accepted persistent owner the sole live topology serializer while preserving read-only observers and separate runtime controller authority.

**Authorized paths**:
- new `src/multiplexer/service.rs`;
- `src/persistent_runtime/owner.rs` narrowly;
- `src/multiplexer/persistence.rs` and domain support;
- focused `src/t165_multiplexer_owner_service_tests.rs`;
- no protocol-v2 wire exposure yet, no desktop/TUI, no Git mutation, no detector, no new owner/daemon;

**Required work**:
- in-memory live topology authority under one owner generation;
- explicit request/release of MultiplexerWrite; every connection starts topology read-only;
- multiple writers may coexist but conflicting writes serialize by expected TopologyGeneration;
- no automatic observer-to-writer promotion;
- capability preflight occurs before known-invalid interactive side effects;
- persistence and accepted-generation publication follow a fail-closed atomicity rule;
- restart restores presentation topology only and reconciles runtime references against canonical Spec 011 truth;
- owner restart never restores controller leases or stale agent observations as fresh;

**Acceptance**:
- multi-client convergence fixtures;
- simultaneous stale topology writes produce one accepted generation and explicit stale results;
- capability-unavailable request creates no topology/runtime/persistence/filesystem side effect;
- no second endpoint/server/daemon is created;

**Primary FR coverage**: FR-004..FR-005, FR-011..FR-015, FR-055..FR-056.

**Primary SC coverage**: SC-002, SC-004.

**Closes to authorize**: T166.

---

## Phase 5 — Private Protocol v2

### [ ] T166 — Bump the existing private local-control protocol to v2 with bounded multiplexer messages, gap semantics, and live-v1 fail-closed behavior

**Purpose**: Expose topology authority through the already-qualified private transport without adding a second API server or generic RPC bus.

**Authorized paths**:
- `src/persistent_runtime/protocol.rs`;
- new `src/multiplexer/protocol.rs` if separation reduces coupling;
- `src/persistent_runtime/owner.rs` only for typed dispatch;
- focused `src/t166_multiplexer_protocol_v2_tests.rs`;
- no desktop/TUI, Git mutation, detector, dependency, network/public RPC;

**Required work**:
- protocol version 2 exact handshake on both owner and the existing shared Rust client; no automatic downgrade;
- all existing accepted Spec 011 runtime-control message semantics remain available under v2 so T166 does not strand the in-repo client while T167 topology projection is still blocked;
- `BLOCKED_LEGACY_OWNER` behavior for a live v1 owner with no auto-kill/reclaim/handoff/second owner;
- freeze the complete first-program v2 wire vocabulary and bounded payload schemas for topology/write-state, agent-observation, attention, and worktree families listed by the canonical Plan, even when a later-domain handler is not yet authorized;
- for every collection response frozen in v2, prove from legal count + field-byte ceilings that one encoded frame fits the 256 KiB protocol ceiling, or freeze a domain-specific bounded page/cursor shape in T166; silent truncation is prohibited;
- any paging admitted by T166 is typed to that exact collection, cursor-bound to owner generation plus the collection's stable snapshot/revision boundary, bounded in item/byte count, and is not a generic pagination/RPC framework;
- not-yet-authorized v2 domain operations return typed `UNSUPPORTED_OPERATION`/equivalent and have no side effect; later tasks activate frozen handlers/projections rather than silently changing v2 message kinds, payload schemas, or collection framing;
- any incompatible wire-schema addition after T166 requires an explicit protocol-v3 or separately accepted Plan/Tasks amendment, not an in-place v2 mutation;
- exact request sequence/correlation/owner-generation/target binding;
- topology request includes exact workspace/target IDs + expected TopologyGeneration;
- event loss/gap explicitly marks client projection stale and requires fresh subscription + authoritative snapshot;
- buffered events are not blindly replayed over a newer snapshot;
- `MULTIPLEXER_SNAPSHOT` exact encoded frame <=262144 bytes including 4-byte prefix; JSON <=262140 bytes; all variable fields bounded;
- `SNAPSHOT_LIMIT_EXCEEDED` rejects before generation/persistence/publication with no truncation;

**Acceptance**:
- v1/v2 mismatch in both directions, live-v1 owner blocking, and same-revision owner/client v2 regression fixtures;
- every existing Spec 011 runtime-control client operation still round-trips under v2 before T166 can close;
- malformed/oversized/truncated/unknown-kind, unsupported-not-yet-authorized-domain, stale-generation, replay/duplicate fixtures;
- encoded maximum-size boundary tests prove total-frame accounting;
- each frozen v2 collection proves either exact single-frame maximum or deterministic typed paging with stale-cursor rejection, no duplicates/omissions, and no silent truncation;
- lost-history fixtures prove resnapshot rather than silent continuation;
- terminal strings shaped like v2 messages remain untrusted data;

**Primary FR coverage**: FR-004..FR-005, FR-052..FR-060.

**Primary SC coverage**: SC-002, SC-013, SC-015, SC-026.

**Closes to authorize**: T167.

---

## Phase 6 — Shared Rust Client Projection

### [ ] T167 — Extend the single Rust local-control client with bounded topology snapshots/events and stale-cache recovery

**Purpose**: Give CLI/TUI/Desktop host one trusted Rust client seam without renderer-direct owner access.

**Authorized paths**:
- `src/persistent_runtime/client.rs`;
- new `src/multiplexer/projection.rs`;
- focused `src/t167_multiplexer_client_tests.rs`;
- no renderer/WebView direct IPC, no Git mutation, detector, UI layout, or new dependency;

**Required work**:
- list/snapshot/subscribe topology through protocol v2;
- fresh subscription + authoritative snapshot recovery after gap;
- exact next-generation application only; discontinuity triggers resnapshot;
- immutable target/result binding retained in client projections;
- v1 owner surfaces explicit upgrade/block state;
- read-only client does not acquire MultiplexerWrite implicitly;

**Acceptance**:
- multi-client snapshot/event convergence;
- gap/reconnect/restart/generation-discontinuity fixtures;
- no direct renderer-to-owner path;
- no authority reconstruction from cached presentation;

**Primary FR coverage**: FR-004..FR-005, FR-013, FR-052..FR-056, FR-061, FR-063.

**Primary SC coverage**: SC-002, SC-013, SC-015.

**Closes to authorize**: T168.

---

## Phase 7 — TUI Topology Projection

### [ ] T168 — Project exact workspace/tab/pane topology into the existing workbench with deterministic keyboard and pointer-equivalent navigation

**Purpose**: Reuse current TUI/workbench rendering while preserving the new immutable topology and authority model.

**Authorized paths**:
- `src/workbench.rs`, `src/workbench_ui.rs`, `src/workbench_screen.rs`, `src/workbench_input.rs`, `src/workbench_interaction.rs`, `src/workbench_context.rs` narrowly;
- `src/multiplexer/projection.rs` and navigation support;
- focused `src/t168_multiplexer_tui_tests.rs`;
- no desktop, detector, worktree mutation, new terminal backend, or dependency;

**Required work**:
- workspace/tab/pane projection uses immutable IDs even under duplicate labels;
- visible deterministic focus and keyboard paths for all primary topology actions;
- pointer focus/manipulation uses exact hit target and never changes authority semantics;
- search/navigation result binds immutable target before action;
- large topology remains deterministic;
- reduced-motion/high-contrast/scaled text preserve lifecycle/authority distinction where TUI can represent them;

**Acceptance**:
- keyboard-only create/split/focus/move/resize/zoom/close flows;
- pointer-equivalent exact-target fixtures;
- duplicate-name and focus-race fixtures;
- no new process launch or worktree mutation;

**Primary FR coverage**: FR-006..FR-009, FR-075..FR-081.

**Primary SC coverage**: SC-001, SC-003, SC-021..SC-023.

**Closes to authorize**: T169.

---

## Phase 8 — Desktop Topology Projection

### [ ] T169 — Add typed Tauri-hosted recursive workspace/tab/pane topology with accessibility and no generic invoke surface

**Purpose**: Bring the same canonical multiplexer model to Desktop through the trusted Rust host only.

**Authorized paths**:
- `src/desktop.rs` and narrowly required Rust bridge/projection code;
- `desktop/src-tauri/src/main.rs` typed commands only;
- new or adapted `desktop/src/multiplexer/**` plus `desktop/src/App.tsx`/styles as required;
- focused Rust tests and `desktop/tests/**`;
- no renderer-direct owner socket, no arbitrary invoke/method bus, no new dependency unless separately proven;

**Required work**:
- recursive tab/pane layout rendered from immutable IDs and generation;
- typed Tauri commands validate all privileged arguments in Rust;
- Desktop terminal-surface capability preflight is independent of controlling TTY;
- keyboard/pointer visible focus parity, duplicate-label safety, reduced motion/high contrast/scaled text;
- no UI label/color becomes trusted authority;

**Acceptance**:
- `quality` succeeds;
- `desktop-quality` succeeds on exact head;
- frontend format/typecheck/lint/tests/build and inert Tauri build pass;
- renderer fuzz cannot invoke unknown privileged method;

**Primary FR coverage**: FR-006..FR-009, FR-061, FR-075..FR-081.

**Primary SC coverage**: SC-001, SC-003, SC-021..SC-023, SC-026.

**Closes to authorize**: T170.

---

## Phase 9 — Exact Pane/Runtime Binding

### [ ] T170 — Bind panes to accepted RuntimeNamespaceId values and enforce controller-only process effects, close policy, and pane-clear presentation epochs

**Purpose**: Connect topology to live terminal ownership without allowing topology write authority to become process authority.

**Authorized paths**:
- `src/multiplexer/domain.rs`, service/projection support;
- `src/persistent_runtime/controller.rs`, `runtime.rs`, and client support narrowly;
- `src/workbench_terminal.rs` / desktop terminal bridge only as exact projections;
- focused `src/t170_pane_runtime_binding_tests.rs`;
- no provider launch, Git mutation, second PTY backend, or dependency;

**Required work**:
- PaneId↔RuntimeNamespaceId binding is explicit and stale-generation checked;
- input/resize/interrupt/stop requires exact Runtime Controller lease;
- `DETACH_VIEW` removes topology only and leaves runtime discoverable;
- `STOP_RUNTIME_THEN_CLOSE` requires MultiplexerWrite + exact Runtime Controller and truthful terminal disposition;
- no implicit terminate-on-close or orphan cleanup;
- `pane.clear` is presentation-only: no child bytes, no lifecycle mutation, no verification/evidence effect;
- cross-focus race cannot retarget input;

**Acceptance**:
- simultaneous focus/swap/takeover/input race fixtures prove zero cross-pane dispatch;
- failed/outcome-unknown stop cannot be shown as clean stop;
- clear affects exact pane only with deterministic retained-history semantics;
- bound pane replacement never inherits stale controller authority;

**Primary FR coverage**: FR-014..FR-017, FR-022, FR-030, FR-055..FR-058, FR-062..FR-064.

**Primary SC coverage**: SC-002, SC-005, SC-008, SC-014..SC-015.

**Closes to authorize**: T171.

---

## Phase 10 — Terminal UX Hardening

### [ ] T171 — Implement bounded scrollback/search/selection/copy/link/graphics/notification behavior inside accepted terminal ownership

**Purpose**: Complete terminal UX without creating remote-control or trusted-status channels.

**Authorized paths**:
- `src/terminal.rs`, `src/workbench_terminal.rs`, `src/workbench_output.rs` and focused helpers;
- `desktop/src/terminal/**`, trusted desktop bridge code only as required;
- focused `src/t171_terminal_ux_tests.rs` and desktop tests;
- no transcript database, remote clipboard/image/file bridge, new terminal backend, or dependency unless separately qualified;

**Required work**:
- bounded scrollback with explicit truncation;
- deterministic selection/word selection/search/copy;
- link activation validates scheme/target and requires accepted user/policy action;
- mouse/right-click/copy-on-select/scroll routing binds exact pane;
- CJK/IME/input-source support is claim-gated by native platform evidence;
- local graphics bounded per Plan; no remote fetch authority;
- clipboard local-only and user/policy mediated;
- notifications/toasts/sound bounded and suppressible;
- titles/themes/effects/borders/gaps/labels remain presentation-only;

**Acceptance**:
- high-output/selection/link/clipboard/graphics/notification adversarial fixtures;
- forged VERIFIED/ACCEPTED/Needs You/provider/model text never changes trusted state;
- desktop-touching candidate passes `desktop-quality`;
- no remote bridge or hidden host action;

**Primary FR coverage**: FR-018..FR-030, FR-075..FR-080.

**Primary SC coverage**: SC-006, SC-008, SC-019, SC-024, SC-026.

**Closes to authorize**: T172.

---

## Phase 11 — Static 24-Family Agent Detector Catalog

### [ ] T172 — Implement the compile-time data-only detector catalog and deterministic fixtures for all 24 required agent families

**Purpose**: Prove detection coverage without plugins, downloaded manifests, terminal-text inference, or execution authority.

**Authorized paths**:
- new `src/multiplexer/agent_catalog.rs`;
- focused `src/t172_agent_catalog_tests.rs`;
- optional immutable test fixtures under `tests/fixtures/`;
- no provider execution adapter, plugin loader, manifest refresh service, network fetch, UI, Git mutation, or dependency;

**Required work**:
- exact enum entries for Pi, Claude, Codex, Gemini, Cursor, Devin, Antigravity, Cline, Omp, Mastracode, OpenCode, GithubCopilot, Kimi, Kiro, Droid, Amp, Grok, Hermes, Kilo, Qodercli, Qwen, Letta, Maki, Muse;
- data-only accepted executable/structured metadata patterns and platform applicability;
- positive/negative/near-match/ambiguous/stale/unsupported fixtures for every family;
- terminal prose, user labels, shell titles, or provider-looking text cannot promote detection;
- detection-only support is explicit; catalog presence grants no launch/install/prompt authority;

**Acceptance**:
- all 24 families have positive + negative + ambiguous + stale evidence;
- catalog cannot execute code or hot reload;
- malformed fixture/rule data fails closed at build/test boundary;
- agent.start remains absent;

**Primary FR coverage**: FR-031..FR-037, FR-042, FR-047, FR-092.

**Primary SC coverage**: SC-007..SC-009, SC-011.

**Closes to authorize**: T173.

---

## Phase 12 — Structured Agent Observation Service

### [ ] T173 — Create bounded agent observations with exact source/freshness/pane/runtime identity and protocol projections

**Purpose**: Turn qualified detector inputs into truthful current observations without provider/model/verification conflation.

**Authorized paths**:
- new `src/multiplexer/agent_state.rs`;
- `src/multiplexer/service.rs`, protocol/projection support narrowly;
- activation of the already-frozen T166 agent-observation v2 handlers/projections only; no v2 message-kind or payload-schema change;
- focused `src/t173_agent_observation_tests.rs`;
- no agent launch/provider API, UI dock, Git mutation, or dependency;

**Required work**:
- `AgentObservationV1` uses required `multiplexer_workspace_id: MultiplexerWorkspaceId` and optional independent `git_workspace_id: GitWorkspaceId`;
- exact TabId/PaneId/RuntimeNamespaceId binding and owner generation/freshness;
- ordered source classes and explicit observed/unknown/ambiguous/stale/unavailable states;
- provider-native session identity remains separate and optional;
- bounded agent-observation snapshot/event projections with explicit gap/resnapshot recovery;
- T173 freezes the exact observation projection byte/item budget permitted by the T166 wire contract and proves its single-frame or already-frozen typed-page boundary; it may not invent new v2 framing;
- no terminal prose detection; no verification/acceptance promotion;

**Acceptance**:
- pane/runtime replacement invalidates stale observations;
- duplicate labels cannot change association;
- event gap invalidates cached observation projection;
- detection support never claims real provider execution;

**Primary FR coverage**: FR-032..FR-044, FR-047, FR-052..FR-054.

**Primary SC coverage**: SC-007..SC-011, SC-013.

**Closes to authorize**: T174.

---

## Phase 13 — Agent Dock and Presentation

### [ ] T174 — Add compact exact-identity agent list/get/read/explain/view/focus/rename presentation to TUI/Desktop

**Purpose**: Expose current observations for human control without granting launch or provider authority.

**Authorized paths**:
- `src/workbench_ui.rs`/projection support narrowly;
- `src/desktop.rs` trusted bridge support;
- new `desktop/src/agentDock/**` or the smallest existing dock extension;
- focused Rust/desktop tests;
- no provider launch, no agent.start, no terminal-text trust, no dependency;

**Required work**:
- compact scanable list with exact immutable bindings and source/freshness labels;
- filter/sort/group are presentation-only and cannot hide material Needs You state;
- rename is display alias only;
- focus resolves immutable pane target and does not grant controller/write authority;
- detection-only limitation visible where execution is unproven;

**Acceptance**:
- duplicate labels/concurrent observations preserve exact target;
- keyboard and pointer paths preserve visible focus;
- `desktop-quality` succeeds for desktop changes;
- no launch/install/provider action exists;

**Primary FR coverage**: FR-038..FR-047, FR-075..FR-081.

**Primary SC coverage**: SC-010..SC-011, SC-021..SC-023.

**Closes to authorize**: T175.

---

## Phase 14 — Structured Needs You

### [ ] T175 — Aggregate trusted human-attention state with deterministic deduplication, stale handling, and bounded events

**Purpose**: Create useful attention without inferring human needs from arbitrary model/terminal text.

**Authorized paths**:
- new `src/multiplexer/attention.rs`;
- service/projection support and activation of already-frozen T166 attention v2 handlers narrowly;
- `src/desktop.rs` / workbench attention projection only as required;
- focused `src/t175_needs_you_tests.rs` and desktop tests;
- no terminal-text classifier, LLM inference, provider launch, or dependency;

**Required work**:
- closed attention kinds such as controller required/conflict, outcome unknown, ownership lost, explicit structured agent attention, worktree trust/destructive confirmation/stale membership, protocol upgrade required;
- stable dedup key retains source domain/event identity/exact target/kind/generation;
- late/stale items remain bound to original target and are discarded or rendered explicitly stale;
- event loss invalidates cached attention projection and requires fresh snapshot;
- T175 freezes the exact attention projection byte/item budget permitted by the T166 wire contract and proves its single-frame or already-frozen typed-page boundary; it may not invent new v2 framing;
- completion/process exit/agent success/verification/acceptance/landing remain distinct;

**Acceptance**:
- identical structured inputs deduplicate deterministically;
- late A event cannot become current B truth after focus change;
- forged terminal Needs You text creates no attention;
- bounded queue and explicit loss semantics proven;

**Primary FR coverage**: FR-046, FR-048..FR-054, FR-082..FR-083.

**Primary SC coverage**: SC-008, SC-012..SC-013, SC-020.

**Closes to authorize**: T176.

---

## Phase 15 — Narrow Agent Input/Wait Evaluation

### [ ] T176 — Evaluate and, only if proven safe, admit exact-pane prompt/send-keys and structured wait for already-live detected agents; keep agent.start prohibited

**Purpose**: Provide bounded local automation without turning detection into execution authority or adding provider APIs.

**Authorized paths**:
- `src/multiplexer/agent_state.rs`/service/protocol/client narrowly;
- `src/persistent_runtime/controller.rs` only for existing exact runtime input path;
- TUI/Desktop typed bindings only if exact behavior is admitted;
- focused `src/t176_agent_local_action_tests.rs`;
- no provider adapter, model router, generic agent.start, plugin, remote, Git landing, or dependency;

**Required work**:
- prompt/send-keys requires AgentObservationId + PaneId + RuntimeNamespaceId + owner generation + current observation freshness + exact Runtime Controller;
- wait is observer-safe and waits only on structured Winds events;
- terminal phrases such as done/approve/question marks never satisfy wait;
- duplicate/replayed request IDs follow protocol semantics;
- any unresolved authority ambiguity closes this task with prompt/send-keys not admitted rather than weakening gates;
- `agent.start` remains explicitly prohibited regardless of outcome;

**Acceptance**:
- focus/swap/takeover/observation-stale races cannot retarget input;
- zero multi-pane broadcast;
- wait uses structured source only;
- UI states truthfully distinguish detection-only from admitted local-input support;

**Primary FR coverage**: FR-041..FR-044, FR-055..FR-058, FR-062..FR-064.

**Primary SC coverage**: SC-008, SC-011, SC-014..SC-015, SC-026.

**Closes to authorize**: T177.

---

## Phase 16 — Repository Trust and Worktree Discovery

### [ ] T177 — Implement read-only system-Git worktree discovery, repository trust, and explicit membership without create/remove

**Purpose**: Build exact Git identity/trust/membership truth before any consequential worktree mutation.

**Authorized paths**:
- new `src/multiplexer/worktree.rs`;
- `src/git.rs`, `src/workspace.rs`, `src/workspace_inventory.rs`, `src/store.rs` narrowly;
- focused `src/t177_worktree_discovery_tests.rs`;
- no create/remove/checkout/fetch/pull/landing, no libgit2, no dependency;

**Required work**:
- machine-readable system Git worktree inventory;
- exact RepositoryIdentity from canonical Git common directory;
- explicit trust record with revision/equivalent snapshot;
- explicit membership states survive discovery drift;
- missing explicit member becomes stale rather than silently erased;
- path reuse by another GitWorkspaceId cannot inherit trust/membership;
- untrusted repositories remain read-only;
- T177 freezes the exact worktree-inventory item/byte budget within the already-frozen T166 collection framing and proves long-path/many-worktree behavior without truncation;

**Acceptance**:
- discovery drift/deleted/recreated path/nested repository/symlink/reparse fixtures;
- explicit membership is not rewritten by discovery;
- no Git mutation command occurs;
- agent/terminal text cannot establish trust;

**Primary FR coverage**: FR-065..FR-070, FR-072..FR-074.

**Primary SC coverage**: SC-017..SC-018.

**Closes to authorize**: T178.

---

## Phase 17 — Safe Worktree Create/Open

### [ ] T178 — Add explicit trusted worktree create/open using exact system-Git identity, OIDs, and destination policy with no force/fetch/landing

**Purpose**: Provide bounded local worktree creation/adoption without branch overwrite or downstream Git workflow authority.

**Authorized paths**:
- `src/multiplexer/worktree.rs`;
- `src/git.rs`, `src/workspace.rs`, `src/workspace_inventory.rs`, `src/store.rs` narrowly;
- focused `src/t178_worktree_create_open_tests.rs`;
- no removal, merge/rebase/cherry-pick/push/PR/approval/landing, no dependency;

**Required work**:
- trusted RepositoryIdentity revalidated at action time;
- base ref resolved read-only to exact commit OID;
- destination root/path canonical, absolute, non-traversing, non-ambiguous, non-state-root;
- default create is detached at exact OID;
- new local branch only when user explicitly supplies a new branch name and conflicts fail closed;
- no `--force`, implicit fetch/pull, existing destination overwrite, or branch stealing;
- post-create/open identity inspection required before membership acceptance;

**Acceptance**:
- untrusted/unsafe/symlink/reparse/existing destination/ref drift/branch conflict fixtures reject before harmful mutation;
- created worktree registers exact GitWorkspaceId + CREATED_BY_WINDS membership only after post-proof;
- open/adopt performs no checkout/fetch/merge/push;
- prohibited Git-operation trace remains clean;

**Primary FR coverage**: FR-065..FR-074.

**Primary SC coverage**: SC-017..SC-018.

**Closes to authorize**: T179.

---

## Phase 18 — Safe Worktree Remove and Recovery

### [ ] T179 — Implement clean-only no-force worktree removal with runtime-binding guards and explicit recovery state

**Purpose**: Allow explicit cleanup without data loss, recursive-delete fallback, or hidden process/Git authority.

**Authorized paths**:
- `src/multiplexer/worktree.rs`;
- `src/git.rs`, workspace/store support narrowly;
- focused `src/t179_worktree_remove_tests.rs`;
- no create behavior expansion, no dependency, no landing operations;

**Required work**:
- exact GitWorkspaceId/canonical path/repository trust revalidated;
- dirty/untracked/unmerged/ambiguous/unsafe state blocks removal;
- active Winds runtime/pane binding blocks until explicitly detached/stopped through existing authority;
- system `git worktree remove` without `--force` only;
- no filesystem recursive-delete fallback;
- post-remove `git worktree list` proof required before membership removal;
- failure preserves stale/recovery-required membership;

**Acceptance**:
- dirty, locked, main worktree, symlink/reparse, path-reuse, active-runtime, partial failure fixtures;
- no user data deleted on failed qualification;
- no `--force` or prohibited Git operation appears;
- UI metadata deletion cannot delete worktree content;

**Primary FR coverage**: FR-067..FR-074.

**Primary SC coverage**: SC-017..SC-018.

**Closes to authorize**: T180.

---

## Phase 19 — Typed Command Palette and Action Registry

### [ ] T180 — Replace command-palette execution ambiguity with a closed typed Winds action registry and exact immutable target binding

**Purpose**: Make navigation/local actions searchable without introducing arbitrary shell, dynamic method, RPC, or broadcast authority.

**Authorized paths**:
- new `src/multiplexer/actions.rs` or smallest equivalent typed registry;
- `src/command.rs` narrowly if shared registry is required;
- `desktop/src/commandPalette/**` and trusted Tauri bridge;
- TUI action palette/search support narrowly;
- focused Rust/desktop tests;
- no arbitrary user shell dispatcher, plugin bus, public RPC, dependency;

**Required work**:
- compile-time action IDs and closed argument schemas;
- selection resolves immutable target snapshot before execution;
- search rank/focus/hover never grants authority;
- unknown action IDs and forged renderer payloads fail closed;
- only actions already authorized by prior tasks may appear as consequential operations;
- no implicit multi-target fanout;

**Acceptance**:
- fuzz unknown/malformed IDs/arguments;
- focus race cannot retarget selected action;
- renderer cannot invoke privileged generic method;
- `desktop-quality` succeeds;

**Primary FR coverage**: FR-055..FR-064.

**Primary SC coverage**: SC-014..SC-016, SC-026.

**Closes to authorize**: T181.

---

## Phase 20 — Adversarial Identity, Authority, Protocol, and Path Campaign

### [ ] T181 — Attack stale identity, cross-target input, forged content, detector ambiguity, protocol v2, and worktree path boundaries

**Purpose**: Prove that the integrated Spec 012 system fails closed under adversarial races and untrusted content.

**Authorized paths**:
- new `src/t181_multiplexer_adversarial_tests.rs` and focused integration fixtures;
- source fixes only inside already-authorized Spec 012 modules when required by failing tests;
- no new product scope, dependency, remote/plugin/updater/Git landing behavior;

**Required work**:
- stale topology/closed-replaced pane/replayed request;
- focus changes during delayed input/prompt/worktree action;
- controller takeover race;
- forged VERIFIED/ACCEPTED/Needs You/provider/model/control JSON/title escape sequences;
- detector basename/wrapper/alias/stale metadata ambiguity;
- event history loss and resnapshot;
- symlink/reparse/path reuse/repository identity substitution;
- malformed/oversized protocol v2 and snapshot-limit campaigns;
- same-user sandbox nonclaim preserved;

**Acceptance**:
- zero unintended cross-target mutation;
- zero trusted-status forgery;
- zero generic dispatch;
- zero unsafe Git cleanup/landing authority;
- all discovered defects fixed within existing authority only;

**Primary FR coverage**: cross-cutting FR-004..FR-092 security/authority portions.

**Primary SC coverage**: SC-002, SC-008, SC-013..SC-020, SC-026.

**Closes to authorize**: T182.

---

## Phase 21 — Stress, Performance, and Resource Qualification

### [ ] T182 — Prove bounded topology, snapshot, replay, observer, detection, notification, graphics, and worktree-discovery resources

**Purpose**: Validate Plan ceilings and usable large-topology behavior without weakening identity/security checks.

**Authorized paths**:
- new `src/t182_multiplexer_performance_tests.rs` and benchmark/evidence artifacts;
- focused workflow only if existing workflows cannot supply reproducible release-profile evidence;
- source fixes only inside accepted Spec 012 modules;
- no feature/dependency expansion;

**Required work**:
- up to 32 multiplexer workspaces, 32 tabs/workspace, 64 panes/tab, 256 aggregate panes, 512 current observations, 128 templates, 256 memberships;
- exact 256 KiB total snapshot-frame bound and field byte bounds;
- owner multiplexer idle CPU/RSS targets;
- snapshot/topology mutation/navigation/projection/agent refresh/worktree discovery p95 targets from Plan;
- high output + slow observers + notification/graphics pressure without duplicated unbounded replay;
- reconnect/churn and repeated split/close cycles;

**Acceptance**:
- raw/lossless-enough metrics with exact candidate/platform/build/fixture provenance;
- all Plan ceilings met or a separate Plan amendment lands before relaxation;
- no correctness/security gate disabled for performance;
- large topology remains deterministic under duplicate names;

**Primary FR coverage**: FR-018, FR-028, FR-081..FR-084.

**Primary SC coverage**: SC-006, SC-020..SC-021, SC-027.

**Closes to authorize**: T183.

---

## Phase 22 — Native Platform Qualification

### [ ] T183 — Directly qualify Linux, macOS, native Windows, and only separately exercised WSL claims for Spec 012

**Purpose**: Prevent platform substitution for input, paths, cursor, process authority, worktree safety, and UI behavior.

**Authorized paths**:
- focused platform tests/evidence and narrowly required fixes;
- existing `windows-terminal.yml`, `t160-native-platform.yml`, desktop/platform workflows or exact focused workflow if proven necessary;
- no cross-domain inference, no remote bridge, no feature expansion;

**Required work**:
- Linux and macOS protocol-v2/private owner/terminal/topology/worktree evidence separately;
- native Windows named-pipe/ConPTY/path/reparse/worktree/input/cursor evidence;
- CJK/IME/input-source evidence on every claimed domain where directly exercisable;
- Desktop macOS evidence through `desktop-quality` where desktop claims are made;
- WSL remains distinct; no Windows-host↔WSL multiplexer bridge;
- same-effective-user execution remains explicitly non-sandbox;

**Acceptance**:
- every released claim has direct exact-candidate evidence;
- native Windows results are never used as WSL evidence and vice versa;
- unsupported limitations remain explicit;
- all applicable platform workflows pass without retries-to-green;

**Primary FR coverage**: FR-023, FR-029, FR-078, FR-084..FR-092.

**Primary SC coverage**: SC-024..SC-027.

**Closes to authorize**: T184.

---

## Phase 23 — Parity and Requirement Reconciliation

### [ ] T184 — Reconcile all 92 FR, 30 SC, and 142 Spec 012 ledger rows to exact canonical evidence or truthful explicit disposition

**Purpose**: Prevent implemented-looking UI from becoming unproven parity, security, provider, or authority claims.

**Authorized paths**:
- new `specs/012-workspace-multiplexer-agent-plane/t184-reconciliation.md`;
- this `tasks.md` checked-state updates for canonically closed T162..T183;
- focused provenance/evidence indexes and proven README/docs updates only;
- no new product behavior;

**Required work**:
- FR-001..FR-092 each mapped to exact canonical task/merge/tests/platform/security/performance evidence or permitted explicit nonclaim;
- SC-001..SC-030 each mapped;
- all 142 ledger rows classified implemented / intentionally non-applicable / explicitly deferred within-domain / authority-blocked with no silent omission;
- 24 agent family detection support matrix exact; provider execution nonclaims preserved;
- migration 0014/dependency inventory exact;
- Herdr donor-code ledger remains empty unless an explicit prior task changed that truth;
- remote/plugin/marketplace/updater/public-RPC/automatic-landing exclusions verified;
- all historical failed/rejected/superseded candidates remain inspectable;

**Acceptance**:
- machine-checkable or deterministic coverage audit finds zero missing FR/SC/ledger row;
- fresh author/Ponytail/independent review finds zero material omission;
- repository quality and applicable platform/security/performance evidence are current;

**Primary FR coverage**: FR-001..FR-092.

**Primary SC coverage**: SC-001..SC-030.

**Closes to authorize**: T185.

---

## Phase 24 — Final Spec 012 Program Closeout

### [ ] T185 — Close the first Workspace Multiplexer & Agent Plane program only after exact evidence and canonical post-merge verification

**Purpose**: Establish final truthful Spec 012 state without opening Spec 013 or any prohibited scope.

**Authorized paths**:
- new `specs/012-workspace-multiplexer-agent-plane/t185-final-closeout.md`;
- this `tasks.md` final checked-state reconciliation;
- proven documentation/status updates only;
- no new product behavior;

**Required work**:
- canonical merge/post-merge proof for T162..T184;
- T184 coverage report complete with zero silent omissions;
- all current source/migration/dependency/protocol/platform/security/resource claims reconciled;
- zero unresolved material review findings/threads;
- fresh Herdr movement classified without scope leakage;
- guarded expected-head normal merge and post-merge push verification;
- no automatic authorization of future remote/plugin/updater/marketplace/public SDK/Git landing/provider-start scope;

**Acceptance**:
- final exact repository truth supports `T162..T185=CLOSED_CANONICAL`;
- `SPEC_012_FIRST_MULTIPLEXER_AGENT_PLANE_PROGRAM=CLOSED_CANONICAL` only after post-merge quality succeeds;
- all prohibited/nonclaimed scope remains explicit;

**Primary FR coverage**: FR-001..FR-092.

**Primary SC coverage**: SC-028..SC-030.

**Closes to authorize**: none inside Spec 012.

---

# Requirement-to-Task Coverage Matrix

This matrix assigns primary implementation/evidence ownership. T184/T185 perform final exact reconciliation.

| Requirement | Primary task(s) |
| --- | --- |
| FR-001 | T162 |
| FR-002 | T162 |
| FR-003 | T162 |
| FR-004 | T162/T163/T165/T166 |
| FR-005 | T162/T163/T166 |
| FR-006 | T163 |
| FR-007 | T163 |
| FR-008 | T163 |
| FR-009 | T163 |
| FR-010 | T163/T164 |
| FR-011 | T163/T164/T165 |
| FR-012 | T164 |
| FR-013 | T162/T167 |
| FR-014 | T163/T170 |
| FR-015 | T163/T170 |
| FR-016 | T170 |
| FR-017 | T170 |
| FR-018 | T171/T182 |
| FR-019 | T171 |
| FR-020 | T171 |
| FR-021 | T171 |
| FR-022 | T170/T171 |
| FR-023 | T171/T183 |
| FR-024 | T171 |
| FR-025 | T171 |
| FR-026 | T171 |
| FR-027 | T171 |
| FR-028 | T171/T182 |
| FR-029 | T171/T183 |
| FR-030 | T170/T171/T181 |
| FR-031 | T172 |
| FR-032 | T173 |
| FR-033 | T172/T173 |
| FR-034 | T172/T173 |
| FR-035 | T172 |
| FR-036 | T172 |
| FR-037 | T172 |
| FR-038 | T173/T174 |
| FR-039 | T173/T174 |
| FR-040 | T173 |
| FR-041 | T176 |
| FR-042 | T172/T176 |
| FR-043 | T162/T173 |
| FR-044 | T173 |
| FR-045 | T174 |
| FR-046 | T174/T175 |
| FR-047 | T172/T174 |
| FR-048 | T175 |
| FR-049 | T175 |
| FR-050 | T175 |
| FR-051 | T175 |
| FR-052 | T166/T167/T175 |
| FR-053 | T167/T175 |
| FR-054 | T173/T175 |
| FR-055 | T162/T165/T170/T176/T180 |
| FR-056 | T162/T165/T180 |
| FR-057 | T166/T170/T176/T180 |
| FR-058 | T166/T176/T180 |
| FR-059 | T180 |
| FR-060 | T166/T180 |
| FR-061 | T167/T169/T180 |
| FR-062 | T170/T176/T180 |
| FR-063 | T162/T167/T176/T180 |
| FR-064 | T170/T176/T180/T181 |
| FR-065 | T177 |
| FR-066 | T177/T178/T179 |
| FR-067 | T177/T178/T179 |
| FR-068 | T177/T179/T181 |
| FR-069 | T177/T178/T179 |
| FR-070 | T177/T179 |
| FR-071 | T178/T179/T181 |
| FR-072 | T177/T178 |
| FR-073 | T164/T179 |
| FR-074 | T177/T181 |
| FR-075 | T168/T169/T174 |
| FR-076 | T168/T169/T174 |
| FR-077 | T168/T169/T174/T180 |
| FR-078 | T168/T169/T183 |
| FR-079 | T168/T169/T174/T180 |
| FR-080 | T168/T169/T171/T174 |
| FR-081 | T163/T168/T169/T174/T182 |
| FR-082 | T175/T182 |
| FR-083 | T166/T171/T175/T182 |
| FR-084 | T182/T183 |
| FR-085 | T183 |
| FR-086 | T183 |
| FR-087 | T162/T181/T183 |
| FR-088 | T181/T183/T185 |
| FR-089 | T172/T180/T181/T185 |
| FR-090 | T181/T185 |
| FR-091 | T164/T181 |
| FR-092 | T172/T183/T184 |

# Success-Criterion-to-Task Coverage Matrix

| Criterion | Primary task(s) |
| --- | --- |
| SC-001 | T162/T163 |
| SC-002 | T162/T163/T181 |
| SC-003 | T163/T168/T169 |
| SC-004 | T164/T165 |
| SC-005 | T170 |
| SC-006 | T171/T182 |
| SC-007 | T172/T173 |
| SC-008 | T171/T173/T181 |
| SC-009 | T172/T181 |
| SC-010 | T173/T174 |
| SC-011 | T172/T173/T176 |
| SC-012 | T175 |
| SC-013 | T167/T175/T181 |
| SC-014 | T170/T176/T180/T181 |
| SC-015 | T166/T176/T180/T181 |
| SC-016 | T180/T181 |
| SC-017 | T177/T178/T179/T181 |
| SC-018 | T178/T179/T181 |
| SC-019 | T171/T181 |
| SC-020 | T171/T175/T182 |
| SC-021 | T163/T168/T169/T182 |
| SC-022 | T168/T169/T174 |
| SC-023 | T168/T169/T174 |
| SC-024 | T171/T183 |
| SC-025 | T183 |
| SC-026 | T169/T180/T181 |
| SC-027 | T182/T183 |
| SC-028 | T184/T185 |
| SC-029 | T181/T184/T185 |
| SC-030 | T184/T185 |

## Tasks Acceptance Gate

This Tasks candidate may land only if its exact final candidate proves:

- changed scope is exactly `specs/012-workspace-multiplexer-agent-plane/tasks.md` unless a separately justified governance-only correction is required;
- Spec 012 Entry, Specification, and Plan are canonically closed with successful post-merge quality on their exact merges;
- dependency order is explicit and canonical Tasks acceptance authorizes T162 only;
- T162 is domain/contract-only and cannot create persistence, protocol-v2 wire behavior, process authority, UI, detection execution, worktree mutation, or new dependency;
- T163..T185 remain blocked by their immediate predecessor;
- all 92 FR appear in the requirement coverage matrix;
- all 30 SC appear in the success-criterion coverage matrix;
- all 142 Spec 012 ledger rows remain owned by T184/T185 final reconciliation with no silent omission;
- migration 0014 is owned only by T164 and migrations 0011/0012/0013 remain frozen;
- protocol v2 is owned by T166 and cannot be implemented early;
- Desktop implementation is split from TUI and requires `desktop-quality`;
- pane/runtime process authority remains owned by Spec 011 and controller-only mutations are not authorized before T170;
- all 24 agent families are detection-only until T172/T173 prove the catalog/observation path;
- `agent.start` is prohibited across all tasks;
- T176 can only evaluate exact-pane prompt/send-keys/structured wait and may truthfully close without admitting them if authority proof fails;
- worktree trust/discovery precedes create/open, which precedes remove;
- no worktree task acquires merge/rebase/cherry-pick/push/PR/approval/landing authority;
- command palette is typed and cannot become a generic shell/RPC/plugin dispatcher;
- resource/platform/security/adversarial qualification occurs before final parity closeout;
- no new direct dependency is pre-admitted;
- Herdr remains research/test-design evidence only with no source admission;
- immediate pre-landing Herdr movement is reconciled without silently expanding Spec 012;
- repository `quality` succeeds on the exact final head;
- author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review finds no over-broad task, generic framework, hidden dependency, premature downstream scope, or task that combines migration/protocol/UI/detection/Git mutation unsafely;
- fresh independent substantive review confirms no omitted FR/SC, impossible ordering, identity ambiguity, authority leak, unsafe worktree path, unbounded resource path, or platform-evidence substitution;
- zero unresolved material findings/review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation immediately before landing;
- guarded expected-head normal merge succeeds;
- merge tree/ordered parents/GitHub signature are verified;
- every actually-triggered post-merge push workflow succeeds.

Only after canonical Tasks landing may repository truth state:

```text
SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_SPEC=CLOSED_CANONICAL
SPEC_012_PLAN=CLOSED_CANONICAL
SPEC_012_TASKS=CLOSED_CANONICAL

T162=AUTHORIZED
T163..T185=BLOCKED_BY_PREDECESSOR

SPEC_012_IMPLEMENTATION_AUTHORIZED=T162_ONLY

MIGRATION_0014_AUTHORIZED=NO
PROTOCOL_V2_IMPLEMENTATION_AUTHORIZED=NO
MULTIPLEXER_OWNER_SERVICE_AUTHORIZED=NO
AGENT_START_AUTHORIZED=NO
WORKTREE_MUTATION_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
MARKETPLACE_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
```

T162 is a domain/contract task only. No production multiplexer service, protocol-v2 wire behavior, migration, terminal/process mutation, detector execution, worktree mutation, or Desktop/TUI behavior becomes authorized merely because this Tasks document lands.
