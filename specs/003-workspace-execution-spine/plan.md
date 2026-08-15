# Implementation Plan: Workspace Execution Spine

## Summary

Build the smallest trustworthy foundation for Winds to become a verification-native developer workspace: exact workspace registration/open/clone, explicit environment and shell discovery, real PTY-backed interactive terminals, Windows/WSL execution-domain switching, and a local SQLite execution ledger that records lifecycle, timing, source, and authority without weakening the existing verification model.

Do **not** build SQL Studio, LLM routing, a GUI terminal renderer, a daemon, persistent detached sessions, or a generic plugin/runtime system in this feature. SQL and LLM come later as typed execution domains on the same workspace/ledger foundation.

## Constitution Check

- **Evidence over claims**: direct process/Git observations may be `WINDS_OBSERVED`; caller intent and shell-reported telemetry remain source-labeled and are not silently elevated: REQUIRED.
- **Non-destructive Git safety**: `winds verify/promote/recover` invariants remain unchanged. User-driven interactive terminals may mutate the primary checkout, but those mutations are workspace activity, not verification evidence: REQUIRED.
- **Spec -> Plan -> Tasks before implementation**: PASS.
- **Ponytail/YAGNI**: one process, existing SQLite/WAL, system Git, no daemon/public IPC/plugin runtime/custom renderer/provider abstraction: REQUIRED.
- **Independent review before acceptance**: REQUIRED.

## Explicit 0.2 Guardrail Amendment

The 0.1 guardrails excluded terminal and native-Windows product work. Spec 003 narrowly authorizes:

- native Windows workspace/terminal support;
- PTY/ConPTY lifecycle control;
- host-shell and WSL profile discovery/launch;
- local workspace/execution records.

Still out of scope:

- daemon / `windsd`;
- public IPC/runtime protocol;
- persistent live sessions across Winds restarts;
- generic plugin/provider runtime;
- remote terminal service;
- custom VT/terminal renderer in the Rust core;
- broad sandboxing;
- MCP/A2A;
- SQL or LLM runtime implementation.

## Canonical Baseline

Specification base:

`8e92c5612a9ddc32996ed5e08475e3c9baa5e161`

Baseline facts:

- Winds `v0.1.0` is released from `041140c6093ad59ac51d523051f5dabe170b784d`.
- Spec 001 and Spec 002 are complete.
- Current product is one Rust binary using system Git, SQLite WAL, and bounded filesystem blobs.
- Existing source is `check.rs`, `domain.rs`, `git.rs`, `main.rs`, and `store.rs`.
- Existing persistence contains only candidate verification/evidence/promotion state.
- Existing accepted runtime evidence covers Ubuntu/macOS; native Windows is not a 0.1 verification claim.

## Research Decisions

Detailed primary-source notes are in `research.md`.

1. Prefer a proven Rust PTY/ConPTY primitive over hand-writing separate Unix and Windows terminal creation. `portable-pty` is the leading candidate, subject to T043 exact-version/license/MSRV/dependency audit.
2. Borrow PTY/session lifecycle concepts from WezTerm, Zed, and Zellij; do not copy their multiplexer, daemon, plugin, or UI architectures wholesale.
3. Defer Ghostty/libghostty or any other renderer until the graphical terminal slice.
4. Use Microsoft's supported WSL CLI as the normative discovery/launch surface.
5. Borrow Atuin's pre-exec/post-command lifecycle idea only where it can be injected ephemerally and source-labeled correctly.
6. Treat mise/project environment files as trust-sensitive inventory inputs, not auto-executed workspace bootstrap.
7. Harlequin/usql/sqlparser-rs and OpenTelemetry GenAI/Langfuse/LiteLLM are follow-on design references, not Spec 003 runtime dependencies.

## Architecture

### Keep one Rust package

Do not create a Cargo workspace or service architecture merely because Winds is growing. Add concrete modules only when implementation needs them:

- `src/workspace.rs` — canonical Git workspace registration/open/clone and safe inventory;
- `src/execution.rs` — common execution identity/status/source and Git observations;
- `src/terminal.rs` — shell profiles and in-process PTY session lifecycle;
- extend `src/store.rs` for persistence.

Platform-specific submodules are acceptable when they make Unix/Windows behavior clearer. Do not turn them into a generic runtime framework.

### Workspace registry

Add a forward-only migration after `0001_init.sql`.

Persist stable local identity only:

- local workspace id;
- canonical worktree root;
- canonical Git common directory;
- created/last-opened timestamps.

Refresh mutable observations when needed rather than treating them as immutable identity:

- HEAD OID;
- branch/detached state;
- dirty-state observation;
- host/execution domain;
- discovered shells/WSL distributions;
- detected manifests.

No full environment snapshot is stored.

### Execution ledger

Use one small common `executions` table plus typed child tables.

Common execution fields should cover only facts genuinely shared across execution types:

- execution id;
- workspace id;
- execution kind;
- source/authority classification;
- execution domain;
- start/end timestamps and duration when known;
- final lifecycle status;
- optional compact references to directly observed before/after Git state.

Add a typed `terminal_sessions` table keyed by execution id for shell executable/arguments identity, profile/domain, initial cwd, terminal dimensions, and close reason.

If T054 proves reliable command lifecycle instrumentation, add a typed `shell_commands` record. Do not add nullable SQL/LLM columns now; later specs add `sql_queries` and `llm_calls` referencing the same execution/workspace identity.

### Source and authority model

The execution ledger must distinguish source from semantic truth.

- **`WINDS_OBSERVED`**: facts directly observed by Winds, such as successful PTY allocation, child process start/exit that Winds owns and waits for, elapsed time measured by Winds, or exact system-Git observations.
- **Caller intent**: requested shell/profile, explicit clone destination, explicit Winds-run command. Existing `CALLER_REQUESTED` naming may be reused only where semantically correct.
- **Shell-reported telemetry**: command text, cwd, exit code, or lifecycle data emitted by shell hooks. This is not `WINDS_OBSERVED` merely because Winds receives it. It remains source-labeled shell telemetry unless an independently protected observation channel proves the fact.
- Terminal/model/command output is observed bytes, but assertions inside that output are not authoritative verification facts.
- Local CLI possession is not authenticated-human identity.

Do not reuse candidate-run event ownership in a way that requires fake candidate `run_id` values. Add execution-scoped events or another concrete execution table when audit history is needed.

### Git observations

Reuse system Git discipline from `src/git.rs`.

For workspace open/inspect:

- resolve top-level worktree and common directory;
- read exact HEAD and branch/detached state;
- record deterministic machine-readable dirty-state observation.

For supported command boundaries, use lightweight system-Git state before/after. Do not recursively hash the entire repository after every prompt. An interactive mutation may be recorded but never becomes `ELIGIBLE` verification evidence merely because it appears in history.

### Clone behavior

Use system Git with explicit URL and destination. Do not build a credential manager in Spec 003.

Persist only sanitized remote identity; credential-bearing user-info/query material must not enter the ledger. Do not auto-run project bootstrap after clone.

System Git can still invoke user-configured credential helpers or filters. Document that boundary rather than claiming hostile-clone sandboxing.

### Environment inventory

Inventory is descriptive, not executable.

Initial safe facts:

- host OS/architecture;
- canonical repository/Git paths;
- shell profile candidates;
- WSL distributions on Windows;
- presence/path of known project manifests.

Do not read `.env` values or evaluate project scripts/configuration to populate the workspace summary.

### PTY session lifecycle

The terminal controller owns exact in-process session resources.

Normal lifecycle:

1. resolve profile, execution domain, and initial cwd;
2. allocate PTY/ConPTY;
3. spawn shell/command attached to the terminal;
4. persist exact known identity/source/lifecycle state;
5. stream output and accept input/resize/interrupt requests;
6. observe the directly owned child/session exit;
7. close/reap resources that Winds can prove it owns;
8. persist final state and duration.

Launch/persistence failures must remain explicit. Raw PTY handles remain private in-process seams, not a stable public protocol.

### Crash and ownership loss

Spec 003 does not provide cross-restart live-session persistence.

If Winds crashes or is forcibly terminated, a child may survive. On the next start:

- a persisted session whose ownership cannot still be proven becomes `OWNERSHIP_LOST` / interrupted;
- process liveness is recorded as unknown unless independently proven;
- a stored PID alone is never sufficient identity because PIDs can be reused;
- Winds must not blindly signal, terminate, or reap a process using only a stale persisted PID;
- recovery is descriptive/fail-closed, not destructive cleanup.

Persistent detached sessions require a later explicit design with a long-lived owner plus versioned reconnection semantics.

### Shell profiles

A shell profile is concrete launch data, not a plugin:

- execution domain;
- exact executable;
- arguments;
- cwd resolution strategy;
- UX display name.

Supported profile families may include POSIX shells, PowerShell/pwsh, `cmd.exe`, and explicit WSL distribution shells. Validate executables at launch time because discovery can become stale.

Switching profile creates a new terminal session; it never pretends an already-running process changed operating-system domain.

### WSL strategy

On Windows:

- discover installed distributions through supported `wsl.exe` behavior;
- represent native Windows and each WSL distribution as distinct execution domains;
- select a distribution explicitly when launching;
- map cwd only when mapping can be validated;
- after launch, verify effective cwd/repository identity inside the selected environment when required for the support claim;
- expose mapping mismatch instead of claiming equivalence.

Do not silently mix Windows Git and WSL Git semantics or treat path-string conversion as proof of identical workspace identity.

### Native Windows boundary

Spec 003 targets native Windows for workspace/terminal execution. This does **not** automatically certify pre-existing `winds verify/promote/recover` behavior on native Windows.

When Windows support lands, add official Windows compile/test jobs for the touched surface. If Unix-specific verification code blocks compilation, isolate platform behavior minimally rather than weakening Unix process-group safety.

### Command lifecycle integration

Command history must favor exactness over convenience.

For a shell integration to be accepted it must:

- be injected ephemerally into Winds-created sessions;
- avoid modifying persistent user dotfiles/profiles;
- provide unambiguous lifecycle markers;
- remain source-labeled shell telemetry unless a protected independent observation proves the fact;
- tolerate unsupported/disabled shell integrations by falling back to session-only telemetry.

Never infer exact commands by scraping arbitrary PTY keystrokes.

If a private marker channel is used, test spoofing/confusion by ordinary child output. Do not turn the marker format into a public runtime protocol.

### Output, retention, and secrets

Live output must remain responsive and persistence bounded.

Prefer:

- bounded in-memory rolling output for live consumption;
- bounded optional transcript artifacts only when justified;
- explicit byte quotas/truncation metadata;
- reuse of existing content-addressed blob discipline where appropriate without pretending terminal transcripts are verification evidence.

Never persist the full process environment by default. Sanitize clone URLs and launch metadata. Command-history secret detection cannot be perfect, so broad persistence requires clear local/private semantics and a per-session history/transcript disable mechanism.

### Minimal CLI proof surface

Before GUI dependency, prove the backend with the smallest CLI surface that can:

- inspect/register an existing workspace;
- clone/register a workspace;
- list shell/execution profiles;
- launch a selected terminal/profile or focused interactive proof flow;
- inspect deterministic execution/session metadata.

Exact command spelling should follow existing CLI conventions during implementation. Avoid a speculative large command tree.

## SQL Studio Follow-On: Spec 004

Spec 004 should reuse workspace/execution identity and add typed SQL records. Quality target:

- secret-safe connection profiles;
- schema/catalog browser and strong context completion;
- multi-statement editor/history;
- explicit transaction behavior;
- conservative read/write classification and visible destructive-write confirmation where classification is reliable;
- query timing, row counts/result metadata, cancellation, bounded export/persistence;
- EXPLAIN/EXPLAIN ANALYZE artifacts where supported;
- dialect-aware parsing with server truth taking precedence over parser guesses;
- every query linked to the workspace timeline.

Do not implement this in Spec 003.

## LLM Observatory Follow-On: Spec 005

Spec 005 should reuse execution identity and adopt a version-pinned observability contract. Quality target:

- exact provider, requested model, and actual response model when available;
- provider-reported input/output/cache/reasoning tokens;
- total duration and streaming time-to-first-token/chunk where available;
- retries, rate limits, and error classification;
- exact cost or `UNKNOWN`, with pricing source/version recorded;
- tool/subagent child spans where later required;
- per-workspace/session aggregates only after raw accounting is correct;
- prompt/response/tool payload privacy controls;
- OpenTelemetry-aligned export/interoperability.

Do not add provider routing/gateway logic in Spec 003.

## Deterministic Gates

Every implementation PR must run:

1. existing quality suite on exact head;
2. focused workspace/store/terminal tests;
3. Linux/macOS regression jobs;
4. Windows job once native terminal code lands;
5. real Windows+WSL2 proof before claiming WSL support;
6. correctness/safety review;
7. Ponytail v4.9.0 review;
8. independent reviewer pass;
9. evidence reconciliation.

## Negative and Fault Testing

At minimum exercise:

- invalid/non-Git/bare/symlinked workspace;
- credential-bearing clone URL sanitization and clone failure before registration;
- environment manifests proving no auto-execution;
- shell executable disappearance/immediate exit;
- PTY allocation/start/read failure;
- input/resize racing with exit;
- interrupt/close escalation while ownership is still proven;
- SQLite failure before/after child spawn;
- stale persisted active session -> `OWNERSHIP_LOST`, process state unknown, no blind PID signaling;
- explicit PID-reuse fixture proving stale PID is never treated as owned identity;
- shell-hook marker spoof/confusion proving shell-reported telemetry is not promoted to `WINDS_OBSERVED`;
- huge output retention bound;
- WSL absent/list failure/distro missing/path mapping mismatch;
- command output claiming success without creating verification authority.

## Soak

After terminal semantics stabilize, run 100 deterministic cycles:

create session -> focused command/input -> resize -> observe exit -> close/reap -> reconcile store.

Require:

- zero leaks of directly owned children during controlled lifecycle;
- zero falsely-live DB records;
- bounded retained output;
- zero corruption/regression in existing verification tables.

The soak must not claim that an unexpected application crash can always kill unknown surviving descendants; crash recovery instead proves conservative `OWNERSHIP_LOST` truth.

## Review Strategy

### Correctness and safety

Review for:

- PTY/process resource leaks and race conditions;
- Windows/Unix close/interrupt differences;
- stale PID reuse and ownership-loss mistakes;
- WSL path/domain overclaims;
- accidental environment/credential/history persistence;
- shell-hook telemetry misattributed as `WINDS_OBSERVED`;
- SQLite partial-transition false success/liveness;
- accidental coupling between workspace history and verification eligibility.

### Ponytail

Challenge every dependency and delete:

- custom multiplexer logic;
- home-grown terminal renderer;
- daemon/protocol machinery;
- generic plugin/provider interfaces;
- environment-manager reimplementation;
- speculative SQL/LLM schema;
- duplicate persistence systems.

### Independent review

At least one reviewer other than the authoring agent must inspect the exact final implementation head. Reviews bound only to an older head do not satisfy acceptance.

## Task Sequencing

Implement in this order:

1. Spec 003 + research;
2. exact PTY dependency/provenance decision;
3. workspace registry/execution ledger;
4. open/inspect;
5. clone/register;
6. environment/shell/WSL discovery;
7. Unix PTY lifecycle;
8. native Windows/ConPTY;
9. WSL launch/path verification;
10. execution persistence plus ownership-loss reconciliation;
11. reliable ephemeral shell lifecycle telemetry where justified;
12. retention/secret controls;
13. minimal CLI proof;
14. cross-platform/fault/negative tests and soak;
15. docs, correctness/safety, Ponytail, independent review, evidence reconciliation.

This order proves identity and truth before convenience, and keeps SQL/LLM ambition from turning the first 0.2 slice into an unreviewable platform rewrite.
