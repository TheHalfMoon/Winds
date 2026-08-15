# Implementation Plan: Workspace Execution Spine

## Summary

Build the smallest trustworthy foundation for Winds to become a verification-native developer workspace: exact workspace registration/open/clone, explicit environment/shell discovery, real PTY-backed interactive terminal sessions, Windows/WSL execution-domain switching, and a local SQLite execution ledger that records lifecycle/time/authority without weakening the existing verification model.

Do **not** build SQL Studio, LLM routing, a GUI terminal renderer, a daemon, persistent detached sessions, or a generic plugin/runtime system in this feature. Instead, make the execution identity/timing/authority model strong enough that SQL and LLM can be added later through typed child records.

## Constitution Check

- **Evidence over claims**: workspace/Git/process facts recorded as `WINDS_OBSERVED` only when Winds actually observes them; caller-entered commands/profile choices remain caller intent: PASS / REQUIRED.
- **Non-destructive Git safety**: `winds verify/promote/recover` invariants remain unchanged. Interactive workspace terminals may intentionally mutate the user's primary checkout because that is explicit workspace behavior, but those mutations cannot become verification evidence merely by appearing in the ledger: EXPLICIT SCOPE DISTINCTION REQUIRED.
- **Spec -> Plan -> Tasks precede implementation**: PASS.
- **Ponytail/YAGNI**: one process, existing SQLite/WAL, system Git, no daemon/public IPC/plugin system/custom renderer, no SQL/LLM provider abstraction: REQUIRED.
- **Independent review before acceptance**: REQUIRED.

### Explicit 0.2 Guardrail Amendment

The 0.1 guardrails excluded native Windows and terminal emulation/runtime work. This active Spec 003 explicitly authorizes **workspace terminal execution**, including PTY/ConPTY and WSL integration, because the founder has requested full terminal control and easy Windows/Ubuntu switching.

This amendment is intentionally narrow:

- allowed now: native Windows workspace/terminal support, PTY/ConPTY lifecycle, shell/WSL profile discovery, local execution records;
- still not authorized: terminal-renderer implementation in the Rust core, daemon/`windsd`, public IPC/runtime protocol, remote terminal service, generic plugin system, MCP/A2A, automatic agent orchestration, broad sandboxing.

## Canonical Baseline

Canonical base at specification start:

`8e92c5612a9ddc32996ed5e08475e3c9baa5e161`

Baseline facts verified before this plan:

- Winds `v0.1.0` is publicly released from exact commit `041140c6093ad59ac51d523051f5dabe170b784d`.
- Spec 001 and Spec 002 are complete; T001-T041 are closed.
- Current `main` contains only the 0.1 verification runtime plus release/public documentation.
- Source remains one Rust binary with `src/check.rs`, `src/domain.rs`, `src/git.rs`, `src/main.rs`, and `src/store.rs`.
- Persistence is SQLite WAL plus content-addressed filesystem blobs under Winds-owned home.
- `migrations/0001_init.sql` contains candidate verification/evidence/promotion tables only.
- Existing CI proves Ubuntu/macOS; native Windows is not currently a 0.1 support claim.
- No current product daemon, PTY terminal controller, workspace registry, SQL engine/client surface, or LLM provider layer exists.

## Research Decisions

Detailed donor/reference notes live in `research.md`. The implementation decisions are:

1. **Use the operating system's real terminal primitive rather than pipes.** Prefer a proven Rust PTY/ConPTY dependency instead of hand-writing separate Unix and Windows terminal creation unless the dependency audit finds a blocker.
2. **`portable-pty` is the leading dependency candidate**, because it comes from the WezTerm Rust terminal ecosystem and is used specifically for portable pseudoterminal creation. T043 must pin an exact version/source/license and validate behavior before adoption.
3. **Do not copy WezTerm/Zellij/Zed terminal architecture wholesale.** Their session/multiplexer/UI systems are broader than this slice. Borrow only tested concepts such as exact session identity, PTY lifecycle ownership, resize, explicit close, and bounded/recoverable state.
4. **Do not adopt `libghostty` yet.** It is promising for future terminal rendering, but its public API is still described upstream as in flux and a terminal renderer is not required to prove this backend spine.
5. **Use Microsoft's supported WSL CLI as the normative Windows↔WSL discovery/launch surface.** Do not reverse-engineer distribution registry state if the supported CLI can provide the fact.
6. **Borrow Atuin's shell lifecycle idea, not its whole history system.** Reliable pre-exec/post-command hooks can supply command/cwd/exit/duration for supported shells; no dotfile mutation or keystroke scraping.
7. **Treat environment managers such as mise as discoverable/trust-sensitive project inputs.** Detect manifest presence first; do not automatically execute project bootstrap/configuration.
8. **SQL and LLM references are design inputs only.** Harlequin/usql/sqlparser-rs inform SQL Studio; OpenTelemetry GenAI/Langfuse/LiteLLM inform LLM observability. None become Spec 003 runtime dependencies.

## Architecture

### Keep one Rust package

Do not split the repository into a workspace or service architecture merely because more capabilities are coming. Add small modules to the existing package when implementation requires them:

- `src/workspace.rs` — canonical repository/worktree registration, clone/open, safe inventory.
- `src/execution.rs` — common execution identity/status/authority and repository observations.
- `src/terminal.rs` — terminal profiles and in-process PTY session lifecycle.
- platform-specific code should stay inside `terminal.rs`/small submodules unless cross-platform code becomes unreadable; do not create a generic runtime framework.
- extend `src/store.rs` rather than creating a second persistence subsystem.

Names may adjust during implementation if existing code structure makes a smaller organization obvious. The important constraint is separation by concrete responsibility, not speculative extensibility.

### Workspace registry

Add a forward-only migration after `0001_init.sql` for a small workspace registry. Prefer a simple integer primary key plus a unique canonical worktree path over inventing a globally distributed workspace identifier.

A workspace row should persist stable local identity only:

- canonical worktree root;
- canonical Git common directory;
- created/last-opened timestamps.

Mutable facts such as HEAD, branch, dirty state, host/domain, discovered shells, and manifest inventory should be refreshed observations/events rather than treated as immutable workspace identity.

No full environment snapshot is stored.

### Execution ledger

Add a common `executions` table for facts shared by concrete execution types. Keep it intentionally small:

- execution id;
- workspace id;
- kind (initially terminal session / shell command when supported);
- source/authority classification;
- execution domain identity;
- start/end time and duration;
- final status;
- optional before/after Git observation references or compact fields where justified.

Add a typed `terminal_sessions` table keyed by execution id containing terminal-specific launch facts such as profile, shell executable/args identity, initial cwd, terminal dimensions, and final close reason.

If command-level shell integration is implemented, add a typed `shell_commands` record rather than stuffing command fields into a generic JSON payload.

Do not add nullable SQL columns or LLM token/provider columns now. Later specifications add `sql_queries` and `llm_calls` typed records referencing the same execution/workspace identity.

### Event authority

Continue the existing Winds habit of append-only observations where it adds audit value. However, do not reuse candidate-run `events` rows for unrelated terminal sessions if doing so would require fake candidate `run_id` ownership. The migration should introduce execution-scoped events or an equivalent concrete table.

Authority rules:

- `WINDS_OBSERVED`: process start/exit, PTY allocation result, timing, exact Git command observations, discovered WSL list output after validated parsing.
- caller intent (existing `CALLER_REQUESTED` naming may be reused where semantically correct): requested shell profile, entered explicit `winds run` command, clone URL/destination selection.
- never label terminal command text or local CLI possession as authenticated `HUMAN_DECIDED` unless a later identity policy actually proves that authority.
- terminal output is observed bytes, but its **semantic claims** are not automatically true. A model/command printing “tests passed” remains output, not verification evidence.

### Git observations

Reuse the system-Git discipline already in `src/git.rs` where possible.

For open/inspect:

- resolve worktree top-level and common directory;
- read exact HEAD/branch state;
- produce a deterministic dirty-state representation/fingerprint suitable for before/after comparison.

For interactive terminal commands, before/after Git observations are useful but must remain cheap. Do not recursively hash the whole repository after every prompt. Prefer system Git's machine-readable status plus HEAD/tree identity and hash the normalized observation if a compact fingerprint is needed.

### Clone behavior

Use system Git. The initial implementation should accept explicit URL + destination and avoid building a credential manager.

Persist only sanitized remote identity. Strip/avoid user-info and other credential-bearing URL data before writing the ledger.

Do not auto-run project bootstrap after clone. The first terminal may be opened only after the workspace identity is established.

System Git may still invoke behavior configured by the user (credential helpers, global filters, etc.); document this boundary rather than claiming hostile-clone sandboxing.

### Environment inventory

Inventory should be descriptive, not executable.

Initial safe facts:

- OS/architecture;
- canonical repository/Git paths;
- shell profile candidates;
- WSL distributions on Windows;
- presence/path of known project manifests.

Avoid reading `.env` values or evaluating arbitrary TOML/scripts merely to display a workspace summary. Later environment activation can be a separate explicit trust action.

### Terminal backend

The terminal controller owns an exact terminal session object in-process.

Required lifecycle:

1. resolve profile + execution domain + initial cwd;
2. allocate PTY/ConPTY;
3. spawn shell/command attached to the slave side;
4. persist start only after identity is known, while preserving launch-failure evidence;
5. stream output to the consumer and accept input/resize operations;
6. observe child/session exit;
7. close/reap owned resources;
8. persist final state/duration or explicit interrupted/persistence-failure state.

Do not make raw PTY reader/writer handles a public stable protocol. Keep them behind private in-process seams so the implementation can evolve when the UI arrives.

### Shell profiles

A profile is concrete launch data, not a plugin.

Examples of profile kinds:

- POSIX shell executable discovered locally (`bash`, `zsh`, `fish`, etc.);
- PowerShell/pwsh on Windows when available;
- `cmd.exe` on Windows when available;
- WSL distribution + selected/default Linux shell.

Profiles should expose a stable display name for UX, but persisted identity includes the exact executable/domain/arguments needed to reproduce the launch decision.

### WSL strategy

Use `wsl.exe` discovery and explicit distribution selection. On a Windows host:

- discover installed distributions in a machine-parseable way supported by WSL;
- record distribution name and WSL version when available;
- map cwd only when mapping can be validated;
- launch the selected distribution explicitly;
- once inside WSL, verify the effective cwd/repository identity rather than trusting path-string conversion alone.

Do not silently run Windows Git against a WSL-native repository or WSL Git against a Windows path and call the environments equivalent. Record the execution domain and let performance/semantic differences remain visible.

### Native Windows boundary

Spec 003 targets native Windows for workspace/terminal execution because that is required for seamless PowerShell↔Ubuntu usage. This does not automatically certify the pre-existing verification engine on Windows.

CI should first make the package compile/test on Windows for the touched surfaces. If pre-existing Unix-only verification code blocks a Windows build, isolate platform-specific behavior minimally; do not weaken Unix process-group safety just to make a compiler green.

### Shell lifecycle integration

Command-level history is valuable, but exactness matters more than pretending every byte sequence is a command.

Implement only shell integrations that can reliably emit command lifecycle markers without modifying persistent user config. A supported integration should provide:

- command start marker;
- cwd at start/end;
- command text when safely available;
- exit code;
- duration.

Inject ephemeral hooks through process environment/startup configuration. Unsupported shells still get session-level timing/control.

Use a documented private marker channel (for example a reserved OSC sequence or side-channel file descriptor) only if it can be parsed unambiguously and cannot be confused with ordinary child output without validation. Do not create a public terminal protocol in this feature.

### Output and retention

Live terminal output must remain responsive; persistence must remain bounded.

Initial policy should prefer:

- in-memory rolling output for the live terminal consumer;
- bounded optional local transcript chunks/artifacts if required for history/debugging;
- explicit byte quotas and truncation metadata;
- no unbounded SQL rows or LLM payload assumptions in this feature.

Reuse the existing content-addressed blob discipline if persisted terminal artifacts are added, but do not force every interactive byte through the verification evidence blob format.

### Secret handling

Never persist the full process environment by default.

Sanitize:

- clone remote URLs;
- shell launch metadata;
- any captured command fields where known secret-bearing wrappers can be identified.

Do not claim perfect command-secret detection. Provide a clear local/private-history boundary and a session/history disable mechanism before broad automatic command capture is enabled by default.

### CLI proof surface

Before relying on a GUI, provide the smallest CLI that proves the backend:

- inspect/register an existing workspace;
- clone/register a workspace;
- list discovered execution/shell profiles;
- launch/attach the current CLI to one chosen terminal session or run a focused terminal integration command;
- inspect execution/session metadata in deterministic JSON.

Exact command spelling should follow existing CLI style during implementation. Avoid a large command tree solely for future UI features.

## SQL Studio Follow-On

Spec 004 should build on the execution spine rather than embedding a separate SQL application.

Expected product quality bar, informed by Harlequin/usql/sqlparser-rs research:

- first-class connection profiles without storing raw secrets;
- schema/catalog browser and strong autocomplete context;
- multi-statement editor/history;
- explicit auto/manual transaction behavior;
- read/write classification and visible write-risk confirmation where classification is reliable;
- query timing, row count, result metadata, export, and bounded result persistence;
- EXPLAIN/EXPLAIN ANALYZE plan capture where supported;
- dialect-aware parsing, with server truth taking precedence over parser guesses;
- each query linked to workspace/execution timeline.

Do not implement these in Spec 003.

## LLM Observatory Follow-On

Spec 005 should use the same execution identity and adopt an explicit observability contract rather than ad-hoc token counters.

Expected quality bar, informed by OpenTelemetry GenAI, Langfuse, and LiteLLM research:

- exact provider + requested/actual model;
- input/output/cache/reasoning token counts from provider usage when available;
- total duration plus streaming time-to-first-token/chunk where available;
- retries/rate limits/error classification;
- cost with explicit price-table/source/version and unknown cost when usage/pricing cannot be proven;
- tool-call and agent/subagent timing as typed child observations when later needed;
- per-workspace/session budgets and spend summaries only after raw accounting is correct;
- secrets never in trace metadata by default;
- export/interoperability path compatible with OpenTelemetry concepts rather than a Winds-only opaque format.

Do not implement provider routing/gateway logic in Spec 003.

## Deterministic Gates

Every implementation PR must run:

1. existing quality suite on exact head;
2. focused workspace/store/terminal tests;
3. Linux/macOS jobs for regression;
4. Windows job once native terminal code lands;
5. WSL integration proof on an actual Windows+WSL environment before making the WSL support claim;
6. correctness/safety review;
7. Ponytail v4.9.0 review;
8. independent reviewer pass;
9. evidence reconciliation.

## Fault and Negative Testing

At minimum exercise:

- invalid/non-Git/bare workspace;
- symlinked/case-variant workspace path;
- credential-bearing clone URL sanitization;
- clone failure before registration;
- shell executable disappears/fails immediately;
- PTY allocation failure;
- output stream closes unexpectedly;
- resize/input racing with exit;
- interrupt followed by escalation/close;
- SQLite failure before and after child spawn;
- stale ACTIVE session on restart;
- huge output retention bound;
- WSL absent/list failure/distro missing/path mapping mismatch;
- environment manifest fixture proving no auto-execution;
- command output claiming success without creating verification authority.

## Soak

Add a deterministic 100-cycle terminal lifecycle soak after core semantics are stable:

create session -> run focused command -> resize -> observe output/exit -> close/reap -> reconcile store.

Success requires no directly owned orphan process, no falsely-live DB row, no unbounded retained output, and no regression/corruption in existing verification state.

## Review Strategy

### Correctness and safety

Focus on:

- process/PTY resource leaks;
- Windows/Unix signal and close semantics;
- race conditions between reader, writer, resize, and child exit;
- incorrect WSL path/domain claims;
- accidental secret/environment persistence;
- shell-hook command misattribution;
- SQLite partial-transition truth;
- accidental coupling between interactive workspace activity and verification eligibility.

### Ponytail

Challenge every dependency and abstraction. Delete:

- custom multiplexer logic not needed for one-process sessions;
- home-grown terminal emulator;
- generic provider/plugin interfaces;
- daemon/protocol machinery;
- environment-manager reimplementation;
- speculative SQL/LLM columns;
- duplicated storage systems.

### Independent review

At least one reviewer other than the authoring agent must inspect the exact final implementation head. Prefer multiple systems for process/terminal code when available, but one reconciled independent pass plus deterministic/safety evidence remains the acceptance minimum.

## Task Sequencing

Implement in this order:

1. canonical Spec 003 + donor research;
2. exact PTY dependency/provenance decision;
3. workspace registry/execution-ledger migration;
4. open/inspect existing workspace;
5. clone/register workflow;
6. shell/environment/WSL discovery;
7. terminal PTY lifecycle on existing supported Unix platforms;
8. native Windows/ConPTY support;
9. WSL launch/path verification;
10. execution/session persistence and interrupted-state reconciliation;
11. reliable ephemeral shell lifecycle integration where justified;
12. retention/secret controls;
13. minimal CLI proof surface;
14. cross-platform/fault/negative tests and 100-cycle soak;
15. docs + correctness/safety + Ponytail + independent review + final evidence reconciliation.

This order proves identity and storage before making interactive process control broad, then proves Windows/WSL before adding convenience instrumentation.
