# Changelog

This changelog records release-facing product behavior. Exact CI/review run identifiers remain in pull-request/spec evidence so this file does not become stale operational metadata.

## Unreleased - 0.2 workspace-terminal spine

This section describes accepted implementation slices already present on current `main`. It is **not** a declaration that `0.2.0` has been released or that every future workspace/terminal UX surface is complete.

### Added

- Exact open/registration of existing non-bare Git worktrees with canonical worktree/Git-common-directory identity, HEAD/branch state, and deterministic working-tree observations.
- Explicit system-Git clone into an absolute destination with failure-before-registration behavior, credential-safe persisted remote identity, external Winds state, and no automatic project bootstrap/environment execution.
- Non-executing workspace environment inventory for host OS/architecture, Git paths, selected project/environment manifest presence, and concrete native shell candidates.
- Exact native shell profiles whose launch identity is derived from execution domain, executable, arguments, and cwd strategy rather than UX display names.
- Windows WSL distribution discovery through the supported `wsl.exe` command surface, with bounded output parsing and explicit unavailable/ambiguous failure behavior.
- Real PTY-backed Unix terminal lifecycle control and native-Windows ConPTY lifecycle support for exact session start, input/output, resize, observed exit, terminate/close, and directly owned resource cleanup.
- Fail-closed native-Windows interrupt behavior where an ownership-scoped foreground interrupt primitive could not be proven; Winds does not fall back to process-global console signaling or report false success.
- Explicit selected-distribution WSL launch/path handling that validates effective cwd and Git identity where mapping equivalence is claimed, otherwise records/uses a visible fallback rather than silently pretending equivalence.
- SQLite-backed common execution records plus typed terminal-session, explicit-command, and lightweight before/after Git-observation records kept semantically separate from candidate verification evidence.
- Conservative terminal restart reconciliation to `OWNERSHIP_LOST`/unknown whenever continuing process ownership cannot be proven; a stored PID is not treated as process identity and is not blindly signaled.
- Structured explicit command execution using an absolute executable plus argv semantics, with requested metadata source attribution and Winds-observed lifecycle/exit facts where proven.
- Bounded local history/transcript controls, transcript persistence default-off, supported history disable controls, conservative metadata sanitization, explicit quota/truncation metadata, and no full process-environment persistence.
- Minimal deterministic JSON CLI proof commands: `workspace-open`, `workspace-clone`, `profiles`, `run`, `terminal-proof`, and `execution`.
- Explicit workspace-terminal trust-boundary documentation separating user-directed workspace activity from authoritative `winds verify` evidence.
- Deterministic negative, lifecycle-race, fault-injection, and partial-persistence fixtures for workspace, clone, profile, PTY/ConPTY, history, SQLite, restart, and spoofing failure modes.
- Linux/macOS terminal integration coverage, native-Windows Spec 003 touched-surface CI, real Windows Server 2025 + Ubuntu WSL2 integration evidence, and a cross-platform deterministic 100-cycle terminal lifecycle soak.
- A dedicated regression gate proving the pre-existing `winds verify` / `promote` / `recover` evidence authority and non-destructive candidate behavior remain intact after the Spec 003 runtime additions.

### Trust and platform boundary

- Workspace terminals and explicit `winds run` commands execute with the launching user's permissions and may mutate the primary checkout or access that user's filesystem, network, environment, and credentials.
- PTY/ConPTY ownership is process/session lifecycle ownership for resources Winds can prove it owns; it is not OS, network, filesystem, credential, or descendant-process confinement.
- Workspace execution/history records do not become candidate eligibility, promotion authority, or verification evidence merely because Winds observes or persists them.
- Native Windows workspace/terminal behavior has accepted touched-surface evidence, but native-Windows authoritative required-check execution for `winds verify` / `promote` remains unsupported and fails closed.
- WSL path conversion alone is not proof of repository equivalence; accepted behavior validates effective WSL cwd/Git identity when equivalence is claimed and makes mismatches visible.

### Explicitly not included

- SQL Studio or SQL execution runtime;
- LLM Observatory, model/provider routing, or token/cost accounting runtime;
- a GUI terminal renderer or terminal emulator UI;
- persistent detached terminals or cross-restart live-session attachment;
- daemon / `windsd`, public IPC/runtime protocol, or remote terminal service;
- generic plugin/provider runtime abstractions;
- MCP/ACP/A2A or Agent Fleet runtime behavior;
- broad OS/network/secret sandboxing;
- native-Windows authoritative verification support;
- automatic downstream merge, rebase, cherry-pick, push, or pull-request automation.

## 0.1.0 - 2026-08-15

### Added

- Exact Git base/candidate resolution through the system Git executable.
- Clean-primary preflight that includes non-ignored untracked files even when repository config attempts to hide them.
- Detached, locked candidate worktrees outside the primary checkout and Git common directory.
- Explicit repository-owned required checks with timeout handling, bounded output capture, process-group termination, and bounded stream shutdown.
- SQLite-backed run/event/evidence/promotion state with append-only recovery-required observations.
- Immutable Evidence Reports bound to exact base/candidate identities and check definitions, with content-addressed SHA-256 output blobs.
- Explicit evidence authority separating Winds-observed facts from caller selection intent.
- Fresh candidate/check revalidation before promotion.
- Dedicated `winds/selected/<run-id>` refs for explicitly selected verified candidates without changing the primary checkout.
- Conservative recovery reconciliation that reports ambiguous, missing, dirty, mismatched, or interrupted state as `MANUAL_RECOVERY_REQUIRED` rather than auto-adopting or force-cleaning it.
- Cross-platform deterministic CI for the supported source/test surface on Ubuntu and macOS.
- Negative fixtures for prohibited downstream Git operations and fault-injection fixtures for partial Git/SQLite transitions.
- Explicit 100-cycle create/verify/promote/reconcile soak evidence with zero observed primary-checkout mutations.
- Public-release readiness metadata, dual `MIT OR Apache-2.0` source licensing, and dependency/provenance audit.

### Security and trust boundary

- Agent-reported success is not authoritative verification evidence.
- Worktree separation isolates checkout/index state; it is not an OS, network, process, filesystem, credential, or secret sandbox.
- Local CLI possession is not represented as authenticated human identity.
- Winds 0.1 does not automatically choose winners or integrate selected changes into a source branch.

### Explicitly not included

- merge, rebase, cherry-pick, push, or pull-request automation as product behavior;
- agent adapters or `winds race` orchestration;
- native Windows execution semantics;
- daemon/public runtime protocol, ACP/MCP/A2A, terminal emulator, TUI/dashboard;
- generic plugin system or sandbox framework;
- network/secret/container/service/database isolation;
- package-manager installers, crates.io publication, auto-update, signing/attestation infrastructure.

These deferred surfaces require new specifications and evidence rather than being implied by version 0.1.
