# T066 Correctness / Safety Review

Status: **ACTIVE — final acceptance pending exact-head CI and review**

Scope: Spec 003 T066 only. This document does not close T066 and does not start or satisfy T067-T069.

## Review target

The review began from canonical main:

`03bd305e9a5a4c7141ae6976e73aefb8ad2fc4ea`

The exact repaired acceptance head is intentionally recorded in GitHub review/CI evidence and the later canonical task-truth reconciliation rather than self-referenced inside this file.

## Required review axes

T066 requires explicit review of:

1. PTY/process ownership;
2. stale PID reuse;
3. Windows/Unix close and interrupt semantics;
4. WSL path/domain truth;
5. SQLite partial transitions;
6. shell-telemetry source attribution;
7. secret/history persistence;
8. separation from verification authority.

## Blocking finding discovered

### T066-F1 — restart reconciliation was not connected to the user-facing process lifecycle

**Classification:** blocking correctness / falsely-live execution truth.

The accepted backend already contained conservative restart reconciliation for terminal and explicit shell-command records, including `OWNERSHIP_LOST` semantics and no PID-based recovery. The T060 fixtures exercised those functions directly. However, the T057 CLI paths opened the Store without invoking restart reconciliation before `workspace-open`, `profiles`, `run`, `terminal-proof`, or `execution` returned or started new work.

A process crash could therefore leave a persisted `REQUESTED` or `RUNNING` execution row that a later CLI process could expose as if it were still live. This violates the Spec 003 requirement that unprovable post-restart ownership reconcile to `OWNERSHIP_LOST` rather than remain falsely live.

A naive repair that reconciles every non-final row at every CLI start is also incorrect: two live Winds CLI processes may overlap, and one process must not mark an execution still owned by another live Winds process as stale.

## Repair design

The T066 repair keeps reconciliation at the **user-facing CLI process boundary**, not inside generic `Store::open`, so a second Store connection inside one live process cannot silently revoke owned execution state.

For current Spec 003 CLI execution surfaces:

- `run` and `terminal-proof` acquire a per-execution cross-process ownership lease before creating the execution record and hold it until their owned execution path has finalized or unwound;
- lease names are deterministic SHA-256 identifiers rather than caller-controlled paths;
- the lease uses a dedicated SQLite file and a retained `BEGIN IMMEDIATE` transaction, so no PID is persisted and no stale PID is ever signaled;
- independent execution IDs use independent lease files, so this is not a global single-execution mutex;
- CLI restart reconciliation checks non-final execution IDs and reconciles a kind only when none of those IDs has a provable live Winds owner;
- if `winds execution` encounters a non-final row whose own lease is not live while another same-kind execution keeps bulk reconciliation deferred, it fails closed rather than displaying that row as truthfully `RUNNING`;
- when no live owner blocks reconciliation, existing Store semantics preserve durable observed shell-command exits as `EXITED` and convert otherwise-unowned non-final terminal/command rows to `OWNERSHIP_LOST`.

This is a process-ownership proof only. It is not a daemon, detached-session protocol, PID recovery mechanism, sandbox, or remote runtime.

## Review results by required axis

### 1. PTY/process ownership

The retained `TerminalSession` child handle remains the authority for live terminal lifecycle operations. Unix interrupt validates the PTY foreground process group against the retained child session before `killpg`. Terminate/close act only through directly retained child handles. Bounded close that cannot prove reaping becomes ownership loss rather than success.

T066-F1 repaired the missing transition between in-process ownership and later CLI-process restart truth. Cross-process leases prove only whether another Winds process still owns the execution lifecycle; they do not grant process signaling authority.

### 2. Stale PID reuse

No PID column was added and no PID lookup/signaling path was introduced. Existing T060 coverage proves PID-shaped stale metadata cannot cause an unrelated live process to be signaled. The T066 repair uses SQLite lock ownership keyed by a hashed execution identity, not OS PID identity.

### 3. Windows / Unix close and interrupt semantics

Unix interrupt remains ownership-scoped to the verified PTY foreground process group. Native Windows continues to fail explicitly for interrupt because an ownership-scoped ConPTY foreground-interrupt primitive was not proven. Native Windows terminate/close remains direct-child-handle based. T066 does not broaden any Windows verification claim.

### 4. WSL path/domain truth

WSL launch remains bound to the selected distribution, exact system `wsl.exe`, exact shell, explicit execution domain, and a mapped-workspace-or-visible-home-fallback strategy. Mapped launch performs effective cwd, Git worktree/common-dir, and exact HEAD attestation before and after terminal start. Mapping mismatch remains visible/fail-closed rather than silently equivalent.

### 5. SQLite partial transitions

Terminal request/session creation remains atomic. Child-start/RUNNING persistence failure retains owned cleanup and conservative repair. Exit-finalization failures do not return false success and are deferred for retry. Restart reconciliation includes partial terminal rows through the existing LEFT JOIN behavior. Explicit shell-command durable exit observations are finalized before remaining non-final rows are classified ownership-lost.

The T066 binary-facing fixture additionally proves the user-facing restart path exercises these semantics rather than leaving them as test-only backend functions.

### 6. Shell telemetry source attribution

Explicit command intent/cwd stays `CALLER_REQUESTED`; process exit and Git boundary observations are `WINDS_OBSERVED`. There is no PTY marker parser or arbitrary keystroke inference. Marker-like child output cannot create `SHELL_REPORTED` authority.

### 7. Secret/history persistence

Terminal transcript history remains disabled by default, explicitly opt-in, bounded by per-session and total quotas, and labeled best-effort metadata redaction rather than perfect secret detection. Command history can be disabled. Obvious secret options/assignments and credential-bearing URL metadata are conservatively redacted/sanitized. Full environment snapshots are not persisted. History deletion remains constrained to validated owned descendants.

Execution ownership lease files contain no command, environment, credential, transcript, PID, repository path, or user-provided execution ID text; the filename contains only a domain-separated SHA-256 digest.

### 8. Separation from verification authority

Spec 003 workspace execution, command history, terminal history, Git observations, and the T066 ownership lease remain separate from candidate/evidence/eligibility/promotion tables and from `winds verify/promote/recover` authority. No execution-history fact is promoted into verification eligibility. No 0.1 required-check behavior is weakened.

## T066-specific deterministic fixtures

`tests/t066_restart_reconciliation.rs` is binary-facing and covers:

- stale terminal `RUNNING` -> `OWNERSHIP_LOST`, unknown process end/duration, typed ownership-lost close reason;
- stale explicit command `RUNNING` -> `OWNERSHIP_LOST`;
- explicit command with a durable `WINDS_OBSERVED` exit fact -> `EXITED` rather than discarded truth;
- a concurrently live `winds run` remains `RUNNING` when inspected from another Winds CLI process;
- a stale same-kind row encountered while a different live owner prevents safe bulk reconciliation is refused rather than displayed falsely live;
- after the live owner finishes, the stale row reconciles to `OWNERSHIP_LOST`.

## Boundaries preserved

This T066 work does **not** add or authorize:

- T067 Ponytail completion;
- T068 independent-review completion;
- T069 final Spec 003 reconciliation/completion;
- detached live terminals across Winds restarts;
- daemon/server/socket/public runtime protocol;
- remote terminal service;
- plugin/provider runtime;
- MCP/ACP/A2A or Agent Fleet;
- Herdr runtime integration;
- terminal renderer or GUI terminal emulator;
- SQL Studio or LLM Observatory;
- native-Windows authoritative required-check execution;
- broad OS/network/filesystem sandboxing.

Herdr remains future donor reference only.

## Acceptance gate

T066 may be closed only after the repaired exact head passes the repository-required deterministic CI and the final T066 correctness/safety review finds no remaining blocking issue. Any later finding must be reconciled on a new exact head before canonical task truth is updated.
