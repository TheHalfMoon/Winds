# Tasks: Winds Desktop Agentic Workspace

## Canonical Inputs

```text
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=CLOSED_CANONICAL
SPEC_010_TASKS=IN_QUALIFICATION
SPEC_010_IMPLEMENTATION_AUTHORIZED=NO
PLAN_MERGE=facbefc3ce3308b826c6faef4080a6c085ad2041
PLAN_TREE=2f013c13b369aa1cae73d2b24352e67026ae9347
POST_PLAN_MERGE_QUALITY=34716142293 SUCCESS ATTEMPT_1
```

Inherited live-runtime nonclaims remain material:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

## Global Rules

1. Live canonical repository truth overrides this Tasks file if the repository moves or an accepted predecessor changes the available seams.
2. Each implementation task starts from then-current exact canonical `main` after its predecessor is `CLOSED_CANONICAL`.
3. No force-push, rebase, shared-history rewrite, rerun-to-green, hidden failure suppression, or stale-head review reuse.
4. Candidate movement invalidates candidate-bound CI, performance, visual, platform, security, and review evidence.
5. The desktop has one local Winds Rust authority owner. WebView renderer domains and terminal/agent child processes are non-authoritative process domains.
6. Renderer state never becomes workspace/session/runtime/workflow/candidate/evidence/acceptance/Git authority.
7. Project labels, Session aliases, pin/order/archive/layout state, runtime marks, and visual grouping are presentation only.
8. Dual-session proximity never creates implicit multi-target action authority.
9. Every asynchronous right-dock result is request-bound to an immutable canonical binding snapshot; stale cross-session attribution is prohibited.
10. No daemon, external durable owner, public IPC/RPC/HTTP/WebSocket control server, remote control plane, browser runtime, generic plugin system, automatic provider routing, or automatic Git landing enters Spec 010 without an accepted amendment.
11. No Tauri shell/filesystem plugin or generic renderer-to-Rust command dispatcher is permitted in the first implementation program.
12. Frozen migration `migrations/0011_model_mesh_continuity.sql` remains immutable.
13. New presentation persistence, if selected by the owning task, must use migration `0012_desktop_presentation.sql` and remain removable without deleting canonical Winds truth.
14. Third-party code/assets are not copied before exact provenance/license/modification records exist.
15. Codex, Claude, Herdr, and Impeccable are references/identity labels only; Winds does not copy competitor trade dress or proprietary assets.
16. Runtime icons never prove runtime identity. Requested, observed, conflicting, stale, unknown, and unavailable states remain distinct.
17. Hidden chain-of-thought is neither requested nor displayed.
18. The first-program frontend uses authored CSS/custom properties and React primitives; no new state/component/CSS/animation framework may enter unless a later task proves necessity.
19. Human aesthetic acceptance in T144 cannot be self-certified by the authoring agent.
20. Closing T145 authorizes no successor implementation automatically.

## Standard Acceptance Gate

Every implementation-bearing task must satisfy, on its exact final head:

- task-focused deterministic tests PASS;
- `git diff --check` PASS;
- `cargo fmt --all -- --check` PASS for Rust-touched tasks;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` PASS for Rust-touched tasks unless the exact workspace transition task defines a narrower canonical command before the new desktop crate is admitted;
- complete applicable Rust tests PASS under repository `quality`;
- frontend format/typecheck/lint/test/build gates defined by T128 PASS for frontend-touched tasks;
- applicable desktop/platform/security/performance workflows PASS when triggered or required by task scope;
- author correctness/safety/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact final candidate with zero unresolved material findings;
- zero unresolved review threads;
- exact base/head/tree/scope/ruleset/mergeability reconciliation immediately before landing;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered post-merge push workflow succeeds.

Documentation/governance-only tasks use the same identity/review/landing discipline and repository `quality`, without inventing irrelevant source tests.

## Dependency Order

```text
T127 -> T128 -> T129 -> T130 -> T131 -> T132 -> T133 -> T134 -> T135
     -> T136 -> T137 -> T138 -> T139 -> T140 -> T141 -> T142 -> T143
     -> T144 -> T145
```

Canonical acceptance and successful post-merge verification of this Tasks file authorize **T127 only**.

---

## Phase 1 — Shared Rust Product Seam

### [ ] T127 — Convert the current package into a reusable Rust library plus thin CLI entry

**Purpose**: allow CLI/TUI/Desktop to share the exact same Rust authority implementation without subprocess duplication or a second domain model.

**Authorized paths**:
- `Cargo.toml` only for a root library target if needed;
- `src/lib.rs`;
- `src/main.rs`;
- existing module visibility/import wiring only where required by the library move;
- focused tests proving CLI behavior and test registration remain unchanged.

**Required behavior**:
- current modules move under the library crate without semantic changes;
- `src/main.rs` becomes a thin binary entry that calls a public library CLI entry;
- existing `pub(crate)` boundaries remain internal to the library wherever practical;
- the library exposes no desktop API yet beyond the minimum entry seam needed by later Tasks;
- every existing CLI/TUI command spelling, output/error contract, exit behavior, and test registration remains unchanged;
- no dependency, migration, desktop source, frontend source, runtime behavior, or new authority path.

**Acceptance**:
- full existing repository quality remains green;
- binary-facing T057 and relevant CLI regressions pass;
- no duplicate module implementation exists between CLI and future desktop paths;
- no current Spec 003/006/007/008/009 behavior changes.

**Closes to authorize**: T128.

---

## Phase 2 — Exact Desktop Dependency Qualification and Inert Shell

### [ ] T128 — Qualify the selected desktop dependency graph and land an inert Tauri/React shell

**Purpose**: establish deterministic desktop build tooling and the smallest privileged shell before product behavior.

**Selected direct versions to qualify**:

```text
tauri=2.11.5
tauri-build=2.6.3
@tauri-apps/api=2.11.1
@tauri-apps/cli=2.11.4
react=19.3.0
react-dom=19.3.0
vite=8.3.0
@vitejs/plugin-react=6.1.1
@xterm/xterm=6.0.0
@xterm/addon-fit=0.11.0
lucide-react=1.45.0
```

**Authorized paths**:
- root Cargo workspace/package metadata only as required for the desktop member;
- `desktop/package.json` + exact lockfile;
- `desktop/tsconfig*.json`, `desktop/vite.config.*`, minimal frontend bootstrap;
- `desktop/src-tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`, capabilities, minimal Rust bootstrap;
- `docs/provenance/` exact dependency qualification artifacts;
- CI workflow additions strictly required to typecheck/build the inert desktop shell on claimed build hosts.

**Required behavior**:
- local-only privileged origin;
- zero remote-origin capabilities;
- no shell/filesystem plugin;
- no generic command dispatcher;
- no product Store/Git/PTTY mutation;
- inert window renders a plain Winds-owned readiness surface only;
- exact dependency versions/licenses/source/checksums/engines/MSRVs/platform graph recorded;
- package manager/toolchain decision is singular and deterministic.

**Acceptance**:
- Rust CLI remains independently buildable;
- desktop frontend typecheck/lint/test/build commands are frozen and documented;
- Tauri dev/release build succeeds on the first directly claimed host(s);
- security review verifies remote content cannot invoke privileged commands;
- dependency review finds no unjustified runtime/framework.

**Closes to authorize**: T129.

---

## Phase 3 — Winds Visual System and Static Product Shell

### [ ] T129 — Establish Winds `Quiet Current` design system and static desktop anatomy

**Purpose**: make visual quality a governed product artifact before data binding spreads styling decisions across the codebase.

**Authorized paths**:
- `PRODUCT.md`, `DESIGN.md`, and Spec 010 surface brief(s);
- `desktop/src/` design tokens, authored CSS, static shell components, runtime mark components, story/fixture data;
- frontend tests and screenshot fixtures;
- no Rust product mutation.

**Required visual system**:
- Quiet Current dark + light foundations;
- high-contrast and reduced-motion tokens;
- compact-professional density;
- left dock / center / right dock anatomy;
- no card-grid shell, decorative glass, AI-purple gradient, fake terminal neon, oversized chat bubbles, or competitor clone;
- general UI icons use the qualified icon grammar;
- Codex/Claude first-program marks are Winds-authored glyph/monogram treatments plus accessible text, not copied proprietary logo files;
- complete hover/focus/active/disabled/loading/error/empty states for primary primitives.

**Required artifacts**:
- color/spacing/type/radius/elevation/motion/focus/state tokens;
- primary 1280x800, 1440x900, and 1920x1080 fixture layouts;
- dark/light/high-contrast fixture screenshots;
- keyboard-focus and scaled-text fixture.

**Acceptance**:
- one bounded visual QA pass plus at most one confirmation pass following the pinned Impeccable Operate/craft-floor discipline;
- WCAG 2.2 AA contrast for product text/control states;
- no competitor asset/trade-dress copy;
- author + independent design review identify no structural UX blocker.

**Closes to authorize**: T130.

---

## Phase 4 — Presentation Persistence

### [ ] T130 — Add removable desktop presentation persistence

**Purpose**: persist organization/layout without converting UI metadata into canonical work/evidence truth.

**Authorized paths**:
- new `migrations/0012_desktop_presentation.sql` only;
- Store APIs/types for desktop presentation records;
- `src/t130_desktop_presentation_tests.rs` and registration;
- no modification to migration 0011.

**Required state**:
- Project display/pin/order/collapse metadata anchored to exactly one canonical Workspace;
- Session alias/pin/order/archive metadata anchored to canonical Winds Session;
- last single/dual layout, slot bindings, split ratio, right-dock surface/binding, dock collapse/width, appearance/density preferences;
- explicit schema version where a compound layout record is used;
- no raw secret, terminal content, agent prose, candidate evidence, or process-ownership fact.

**Required invariants**:
- deleting presentation records leaves canonical Workspace/Workstream/Session/runtime/workflow/candidate/evidence truth intact;
- restored layout does not imply restored live ownership;
- foreign-key/canonical identity mismatch fails closed;
- presentation update races are monotonic/deterministic.

**Acceptance**:
- fresh DB + migrated DB + restart tests;
- corrupt/partial presentation state fails safely and does not corrupt canonical tables;
- migration inventory exact and 0011 digest unchanged.

**Closes to authorize**: T131.

---

## Phase 5 — Desktop Rust Projection Facade

### [ ] T131 — Implement bounded Project/Session/runtime/attention desktop projections

**Purpose**: expose a single Rust-owned read/write facade for UI organization without generic Store/Git access.

**Authorized paths**:
- new `src/desktop.rs` and focused support module(s);
- Store read/write methods strictly required by desktop presentation behavior;
- `src/t131_desktop_projection_tests.rs` and registration;
- no Tauri command implementation beyond fixture adapters.

**Required projections**:
- Project summaries anchored to canonical Workspace/repository identity;
- Session summaries anchored to canonical Winds Session;
- existing Session rename path reused or wrapped rather than duplicated;
- runtime state separates requested/observed/mismatch/unknown/unavailable/stale;
- deterministic attention rollup from accepted sources;
- duplicate aliases retain disambiguating canonical context;
- no runtime icon or label upgrades proof level.

**Required commands**:
- list/open/create presentation Project context where canonical prerequisites exist;
- list/create/rename Session through accepted identity paths;
- update pin/order/collapse/archive presentation state;
- load/save layout;
- exact deterministic errors.

**Acceptance**:
- forged labels/self-report cannot create observed runtime state;
- rename leaves canonical IDs unchanged;
- presentation ordering has zero effect on routing/verification/acceptance;
- >=100 Projects / >=1000 Sessions fixture produces stable deterministic ordering/search inputs.

**Closes to authorize**: T132.

---

## Phase 6 — Left Dock Projects and Sessions

### [ ] T132 — Bind the left dock to canonical desktop projections

**Purpose**: deliver the primary Project -> Sessions organization experience requested by the Founder.

**Authorized paths**:
- `desktop/src/` left-dock components/state/tests;
- exact Tauri commands/types wrapping T131 only;
- no terminal or right-dock implementation.

**Required behavior**:
- projects expandable/collapsible/pinnable/reorderable where T130 state exists;
- new Session action originates inside exact Project context;
- in-place/equally direct Session rename;
- runtime mark + accessible label + truthful proof state;
- lifecycle/attention status not color-only;
- collapsed Project material attention rollup;
- deterministic search/filter with explicit ambiguity for consequential action;
- keyboard and pointer navigation;
- stable focus through async row updates/reorder.

**Acceptance**:
- SC-001, SC-005 through SC-008, and left-dock portions of SC-012/013 directly exercised;
- 1000-session fixture retains stable selection/search and meets later performance-instrumentation hooks;
- forged agent/terminal strings cannot inject trusted dock chrome.

**Closes to authorize**: T133.

---

## Phase 7 — Single Agent Session Surface

### [ ] T133 — Implement one Codex-direct, Winds-native Session work surface

**Purpose**: make one session feel like an agent workspace rather than a terminal or chat clone.

**Authorized paths**:
- `desktop/src/` Session Slot header, typed work stream, composer, fixture bridge/tests;
- desktop Rust typed event projection needed to render already-authorized facts;
- no dual-session layout and no real new Codex/Claude execution authority.

**Required behavior**:
- exact target Session header with alias/runtime/lifecycle/worktree context;
- sticky multiline composer whose target is always explicit;
- before T139 closes with separately accepted live-runtime input authority, the composer MUST be fixture-only/non-dispatching or render a truthful unavailable state; it MUST NOT send input to a live Codex, Claude, terminal, agent process, or any other execution surface;
- T133 UI events and fixtures MUST NOT be interpreted as runtime-launch or runtime-input evidence;
- typed event classes for user prompt, user-visible agent response/summary, tool/action, command/result, file change, test/check, approval/attention, error, completion;
- source/provenance retained visually and structurally;
- collapsible tool detail retains target/status/source/failure visibility;
- no hidden chain-of-thought surface;
- file/diff events route to future dock binding without forging verification.

**Acceptance**:
- fixture streams for Codex/Claude/requested-only/shell/unknown/mismatch;
- deterministic proof that pre-T139 composer submit cannot reach any live runtime/terminal/agent execution seam and exposes truthful unavailable/fixture state instead;
- agent-reported `PASS`, `approved`, `done`, runtime names, or evidence-shaped JSON cannot change trusted state;
- composer keyboard, IME, multiline, selection/copy, loading/error/empty states pass.

**Closes to authorize**: T134.

---

## Phase 8 — First-Class Dual Session Mode

### [ ] T134 — Implement exactly two independent visible Session Slots

**Purpose**: allow real parallel work in one page without accidental multi-target authority.

**Authorized paths**:
- `desktop/src/` center workspace/split state/components/tests;
- presentation layout binding updates through T130/T131;
- no terminal bridge yet.

**Required behavior**:
- single + dual modes;
- exactly two slots in first program;
- independent focus/composer/scroll/selection;
- resizable divider;
- swap/maximize/restore/replace-one/close-one semantics;
- peer Session identity/lifecycle untouched by operations on the other slot;
- shared-worktree condition visibly identified;
- no input/action broadcast from proximity, selection rectangle, or shared Project;
- too-narrow window uses explicit fallback rather than unreadable compression.

**Acceptance**:
- SC-002, SC-003, SC-026 directly exercised;
- adversarial target-race tests prove zero unintended cross-session dispatch;
- layout restart restores identities as presentation only and never live ownership.

**Closes to authorize**: T135.

---

## Phase 9 — Terminal Renderer Bridge

### [ ] T135 — Attach xterm.js presentation to existing Rust PTY/ConPTY ownership

**Purpose**: provide best-in-class terminal interaction where terminal is the correct Session sub-surface without a second terminal authority.

**Authorized paths**:
- bounded desktop terminal Rust facade/Tauri commands/channels;
- `desktop/src/` terminal component using only qualified xterm packages;
- terminal-specific tests/workflows;
- no custom PTY implementation.

**Required behavior**:
- Rust owns launch/input/resize/interrupt/terminate/close/lifecycle;
- renderer receives bounded output stream and sends exact-target input/resize only;
- no raw renderer shell command API;
- terminal output is visibly untrusted data;
- OSC 52 disabled/fail-closed by default;
- links/file references require explicit user action and validation;
- literal search and fit support;
- hidden terminal output does not force whole-app frame work per byte;
- ownership loss/exited/interrupted remain truthful.

**Acceptance**:
- accepted Unix PTY/native Windows ConPTY semantics remain unchanged;
- Unicode/control-sequence/resize/output-stress fixtures pass;
- forged terminal content cannot create trusted React controls/badges/actions;
- applicable Windows/macOS/Linux terminal workflows directly qualify touched platform claims.

**Closes to authorize**: T136.

---

## Phase 10 — Files and Changes Right Dock

### [ ] T136 — Implement Files and Changes with immutable binding snapshots

**Purpose**: keep both Sessions visible while safely inspecting the exact worktree/candidate context.

**Authorized paths**:
- Rust desktop file/Git read-only projections;
- right-dock `Files` and `Changes` frontend surfaces/tests;
- immutable request-token/binding code;
- no arbitrary renderer filesystem or Git passthrough.

**Required behavior**:
- Files rooted to exact canonical workspace/worktree;
- bounded text preview, explicit binary/large-file state;
- Changes uses repository-native Git status/diff projection;
- diff rendering never implies verification;
- dock follows selected Session by default;
- explicit pin binding where UI enables it;
- every async request/result carries immutable Project/Session/worktree/candidate/tree binding snapshot/digest;
- late mismatched result is discarded or rendered only as explicitly stale with original binding visible.

**Acceptance**:
- SC-009 + Files/Changes portions of SC-031;
- path traversal/symlink/root-mismatch hostile fixtures fail closed;
- terminal/agent forged path cannot inject trusted tree item or host-open action.

**Closes to authorize**: T137.

---

## Phase 11 — Evidence, Context, and Artifacts Right Dock

### [ ] T137 — Add verification-native Evidence/Context/Artifacts surfaces

**Purpose**: make Winds truth available beside active agent work without creating UI-local authority.

**Authorized paths**:
- read-only desktop projections over existing candidate/evidence/workflow/context/artifact seams;
- right-dock frontend surfaces/tests;
- no new verifier or artifact authority.

**Required behavior**:
- Evidence reuses repository-native verification semantics;
- Context exposes accepted session/workflow/context summaries with source labels;
- Artifacts expose exact identity/provenance and safe open/reveal behavior;
- stale candidate/tree movement removes trusted treatment immediately;
- immutable binding rule applies to every request/result;
- no agent/model prose becomes evidence because displayed here.

**Acceptance**:
- SC-010 and Evidence/Context/Artifacts portions of SC-031;
- candidate movement/stale response/adversarial forged evidence fixtures pass;
- no verification mutation occurs from read-only browsing.

**Closes to authorize**: T138.

---

## Phase 12 — Needs You and Universal Command Surface

### [ ] T138 — Implement deterministic human-attention rollups and command/navigation palette

**Purpose**: supervise parallel agent work without transcript polling.

**Authorized paths**:
- Rust attention projections from existing accepted facts;
- frontend `Needs You` dock, project/session rollups, command surface, tests;
- no new delegation/approval authority.

**Required behavior**:
- explicit reason + exact Project/Session/workflow/candidate/authority context;
- trusted item source must be accepted fact, not agent text;
- deterministic priority policy;
- one-click focus/navigation;
- approve/deny only if underlying canonical approval seam already authorizes it;
- `Cmd/Ctrl+K` covers Project/Session navigation, right-dock navigation, safe actions, and search;
- ranking never grants authority;
- ambiguous consequential targets require explicit disambiguation.

**Acceptance**:
- SC-007, SC-012/013 command paths, attention adversarial fixtures;
- printed `blocked/done/approved` strings cannot create trusted attention/decision state;
- collapsed Project rollups remain visible without color-only encoding.

**Closes to authorize**: T139.

---

## Phase 13 — Real Runtime Launch Authority Gate

### [ ] T139 — Qualify direct desktop Codex/Claude launch authority or close truthfully unavailable

**Purpose**: satisfy the Founder goal of launching Codex/Claude from a Session without promoting old Spec 006 nonclaims or smuggling execution through UI convenience.

**Authorized by default**:
- necessity/provenance/evidence artifact;
- read-only inspection of current runtime discovery/session/structured protocol seams;
- exact proposal for a separate governance amendment if current authority is insufficient;
- no direct live Codex/Claude execution code unless a separately accepted amendment explicitly authorizes it.

**Decision**:
- if then-current canonical authority already proves a bounded live launch seam, bind Desktop to it and prove exact live evidence;
- otherwise record `DESKTOP_DIRECT_CODEX_LAUNCH=UNAUTHORIZED` and/or `DESKTOP_DIRECT_CLAUDE_LAUNCH=UNAUTHORIZED`, keep requested/unavailable UI truthful, and create no backdoor launcher;
- if the Founder directive and current constitution permit a bounded amendment candidate, author and qualify that amendment separately before returning to this task.

**Non-negotiable**:
- preserve historical `REAL_CLAUDE_EXECUTION=NO` / `REAL_CODEX_WORKER_EXECUTION=NO` until new exact evidence truly supersedes them;
- terminal self-report/process title alone is insufficient structured-runtime proof;
- no credential scraping/storage;
- no automatic provider/model fallback.

**Acceptance**:
- exact authority status for Codex and Claude is documented and UI behavior matches it;
- any live implementation has direct runtime-specific exact-candidate evidence;
- unavailable path remains first-class and polished rather than fake-connected.

**Closes to authorize**: T140.

---

## Phase 14 — Accessibility, Themes, Density, Responsive Desktop

### [ ] T140 — Complete dark/light/high-contrast/reduced-motion/compact and accessibility acceptance

**Purpose**: make polish and accessibility product requirements, not final cosmetic patches.

**Authorized paths**:
- desktop tokens/components/layout/a11y tests;
- no canonical Rust authority change except accessibility metadata/projections required by existing facts.

**Acceptance**:
- WCAG 2.2 AA contrast;
- complete keyboard-only primary workflows;
- pointer parity;
- visible focus and semantic current/selected state;
- screen-reader meaningful Project/Session/runtime/attention structure;
- 200% text scale;
- IME/composer behavior on directly exercised platforms;
- reduced motion/high contrast/dark/light/compact fixtures preserve truth distinctions;
- narrow layout fallback remains usable.

**Closes to authorize**: T141.

---

## Phase 15 — Desktop Security and Adversarial Campaign

### [ ] T141 — Prove renderer/content/bridge safety under adversarial inputs

**Purpose**: directly challenge the privileged desktop boundary before broad platform claims.

**Required campaign**:
- hostile terminal/model text forging trusted controls/status/runtime marks;
- hostile HTML/Markdown/script-shaped content;
- path traversal/symlink/root swap;
- malicious external links and file URLs;
- clipboard-write attempts;
- remote-origin/navigation attempts;
- generic invoke/command-name guessing;
- oversized payload/output;
- stale binding/result replay;
- duplicate aliases and focus races;
- dual-session cross-target race;
- corrupt presentation metadata;
- secret-shaped strings in logs/fixtures/screenshots.

**Acceptance**:
- SC-011, SC-019, SC-021 through SC-023 and SC-031 adversarial paths;
- no remote origin obtains local command authority;
- no arbitrary filesystem/shell/Git bridge appears;
- security findings repair forward-only and restart exact-head qualification.

**Closes to authorize**: T142.

---

## Phase 16 — Native Platform Qualification

### [ ] T142 — Directly qualify macOS, native Windows, Linux, and WSL-domain desktop behavior claimed by the program

**Purpose**: prevent one-platform UI success from becoming a universal desktop claim.

**Required evidence**:
- release-like Tauri frontend/build on each claimed desktop host;
- native window/input/focus/IME behavior relevant to claims;
- terminal/PTTY or ConPTY integration where touched;
- native Windows and WSL2 remain distinct execution/path domains;
- dark/light/system appearance and high-DPI behavior where claimed;
- no cross-platform substitution.

**Acceptance**:
- every released platform claim has exact-candidate direct evidence;
- unsupported limitations are visible/product-safe;
- post-build artifacts are identity-bound where retained.

**Closes to authorize**: T143.

---

## Phase 17 — Performance and Stress Qualification

### [ ] T143 — Prove frozen launch/interaction/render/idle/scale budgets

**Purpose**: ensure the premium interface stays fast with real parallel work.

**Frozen Plan budgets**:

```text
cold launch <= 1500 ms p95
Project/Session selection <= 50 ms p95
single <-> dual layout <= 100 ms p95 excluding first fetch
composer keystroke-to-paint <= 16 ms p95
cached right-dock tab switch <= 50 ms p95
idle CPU <= 2% one logical core
renderer+host idle RSS <= 300 MiB excluding child agents/terminals
```

**Required campaigns**:
- >=20 cold launches;
- >=1000 selection/focus/composer operations;
- >=100 Projects / >=1000 Sessions list/search/scroll fixture;
- two visible sessions under representative output;
- multiple hidden output-producing sessions proving no full-frame work per byte;
- resize storm / huge output / right-dock switch stress;
- exact environment/candidate/build/WebView provenance.

**Acceptance**:
- all frozen budgets met without weakening correctness/security/a11y;
- WebGL/virtualization/new dependency is not added unless a measured failure separately justifies it;
- raw/lossless-enough results retained.

**Closes to authorize**: T144.

---

## Phase 18 — Human Visual Acceptance and Bounded Final Polish

### [ ] T144 — Obtain Founder visual acceptance on the exact release candidate and apply at most one bounded polish successor

**Purpose**: satisfy SC-015/SC-030 without allowing the authoring agent to self-certify aesthetics.

**Required presentation**:
- exact-candidate screenshots/captures at 1280x800, 1440x900, 1920x1080;
- dark + light primary surfaces;
- Project/Session left dock with Codex/Claude runtime treatments;
- single and dual Session modes;
- Files/Changes/Evidence/Needs You right-dock examples;
- keyboard focus/high-contrast/reduced-motion evidence;
- real/representative long-running session content without fabricated authority claims.

**Founder rubric**:
- beautiful and professional;
- feels direct/agent-native like the best modern coding-agent tools but recognizably Winds;
- better organized than flat session history;
- dual-session work is clear and desirable;
- right dock is useful without clutter;
- runtime identity is instantly legible;
- no terminal-only/product-dashboard feel;
- no competitor clone;
- density, typography, spacing, states, and motion feel production-grade.

**Founder acceptance record**:
- an explicit human Founder PASS is required from a human-controlled authenticated review location;
- the record MUST identify the human reviewer/account, exact candidate commit, exact tree, exact desktop build/artifact identity, reviewed capture set, rubric disposition, and decision timestamp;
- the authoring agent or automation MUST NOT create, attest, synthesize, or substitute the Founder decision; automation may only link or reconcile the independently authored record;
- a PASS bound to an earlier candidate becomes stale after any successor commit.

**Authorized polish sub-scope, only after a human Founder review identifies visual defects**:
- at most one forward-only successor batch;
- changes are limited to `desktop/src/**`, `desktop/index.html`, and Winds-owned desktop visual assets/styles/tokens already introduced by earlier tasks;
- allowed change classes are typography, spacing, sizing, layout composition, responsive breakpoints, visual hierarchy, icon placement, color/token tuning, focus treatment, motion timing/easing, copy clarity, and non-behavioral component presentation;
- no Rust core/bridge authority change, migration, dependency/lockfile change, workflow change, runtime launch/input behavior, filesystem/Git behavior, terminal ownership behavior, persistence semantics, security relaxation, or new product capability is authorized;
- any defect requiring behavior, authority, dependency, persistence, or security changes exits T144 and returns through the appropriate earlier task or a separately accepted amendment.

**Process**:
- the initial exact candidate receives the human Founder review described above;
- if that review is PASS with no material visual defect, no polish successor is created;
- if the Founder identifies material visual defects, the single bounded polish sub-scope above may produce one forward-only successor, followed by complete exact-head CI/reviews and a **new** human Founder acceptance record bound to the successor exact commit/tree/build/captures;
- the pre-polish PASS/review cannot qualify the post-polish head;
- no open-ended polish loop.

**Acceptance**:
- exact human Founder acceptance record satisfies every field above and is externally distinguishable from authoring-agent output;
- any polish successor stays entirely within the bounded paths/change classes and passes complete exact-head requalification before the final human re-check;
- final accepted candidate has no material visual defect under the Founder rubric.

**Closes to authorize**: T145.

---

## Phase 19 — Final Spec 010 Reconciliation

### [ ] T145 — Final requirement/evidence reconciliation and first desktop program closeout

**Authorized paths**:
- this `tasks.md` final checked-state reconciliation;
- `specs/010-winds-desktop-agentic-workspace/t145-final-reconciliation.md`;
- focused acceptance/evidence artifacts;
- proven README/docs updates;
- no new product behavior.

**Acceptance**:
- T127..T144 exact canonical merges and post-merge verification reconciled;
- FR-001..FR-133 each classified to deterministic/platform/security/performance/human-visual/governance evidence or explicit truthful nonclaim where permitted;
- SC-001..SC-031 reconciled to exact evidence;
- Project/Session aliases remain non-authoritative;
- dual-session zero-broadcast and right-dock immutable-binding proofs remain intact;
- renderer/Rust/child-process authority domains remain exact;
- migration 0011 unchanged and presentation migration inventory reconciled;
- dependency/provenance inventory exact;
- Spec 006 live-runtime historical nonclaims preserved or explicitly superseded only by separately accepted exact evidence;
- every historical failed/rejected/superseded candidate remains inspectable and not relabelled;
- final exact implementation state passes repository/frontend/desktop/platform/security/performance gates;
- Founder visual acceptance is exact-candidate bound;
- fresh author/Ponytail/independent review reports zero material findings;
- guarded expected-head landing and post-merge push verification complete.

Only after canonical landing and post-merge proof may repository truth state:

```text
T127..T145=CLOSED_CANONICAL
SPEC_010_ENTRY=CLOSED_CANONICAL
SPEC_010_SPEC=CLOSED_CANONICAL
SPEC_010_PLAN=CLOSED_CANONICAL
SPEC_010_TASKS=CLOSED_CANONICAL
SPEC_010_FIRST_DESKTOP_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

**Closes to authorize**: no successor implementation task.
