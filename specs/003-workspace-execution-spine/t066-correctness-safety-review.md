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

## Blocking findings discovered and reconciled

### T066-F1 — restart reconciliation was not connected to the user-facing process lifecycle

**Classification:** blocking correctness / falsely-live execution truth.

The accepted backend already contained conservative restart reconciliation for terminal and explicit shell-command records, including `OWNERSHIP_LOST` semantics and no PID-based recovery. The T060 fixtures exercised those functions directly. However, the T057 CLI paths opened the Store without invoking restart reconciliation before `workspace-open`, `profiles`, `run`, `terminal-proof`, or `execution` returned or started new work.

A process crash could therefore leave a persisted `REQUESTED` or `RUNNING` execution row that a later CLI process could expose as if it were still live. This violates the Spec 003 requirement that unprovable post-restart ownership reconcile to `OWNERSHIP_LOST` rather than remain falsely live.

### T066-F2 — first lease repair raced bulk reconciliation against new execution startup

**Classification:** blocking correctness / live-owner revocation race.

The first T066 repair probed all non-final per-execution leases and then called the existing Store-wide reconciliation method only when no live owner was seen. That still left a snapshot-to-update race: after the probe released its temporary lease, another process could acquire its own execution lease and create a new non-final row before the bulk Store update. Because the Store method updates every matching non-final row, the new live row could be marked `OWNERSHIP_LOST`.

The final repair removes Store-wide reconciliation from the CLI restart path. The CLI now reconciles only the execution IDs captured in its snapshot, one row at a time, while retaining that row's successfully acquired ownership lease for the entire targeted transition. New execution IDs created after the snapshot are therefore outside the update set, while a live owner of a captured ID keeps its lease busy and is skipped.

### T066-F3 — deleting lease files after unlock could create dual owners

**Classification:** blocking correctness / lease pathname split-brain.

The first lease implementation rolled back and closed SQLite, then removed the lease file. On Unix another process could open and lock that pathname between close and unlink; unlink would detach the new owner's live inode from the pathname, allowing a third process to create a replacement file and obtain a second apparent owner.

The final repair never deletes hashed lease database files during normal lease release. Liveness is represented only by the retained SQLite `BEGIN IMMEDIATE` transaction. The stable pathname remains the common rendezvous object for all future probes, preventing unlink/recreate split-brain.

### T066-F4 — same-kind deferral left unrelated stale rows non-final

**Classification:** specification compliance / restart-truth completeness.

The first repair conservatively deferred all reconciliation for an execution kind if any same-kind live owner existed. Although display failed closed, this left unrelated unowned rows internally non-final and introduced a user-visible refusal branch rather than performing FR-019's required ownership-loss transition.

The final repair reconciles each captured non-final execution independently. A live row is skipped because its own lease is busy; an unrelated stale row is reconciled immediately while its acquired probe lease prevents a same-ID owner from appearing during the targeted transaction.

### T066-F5 — probe-lease lifetime was made explicit across targeted reconciliation

**Classification:** ownership-proof clarity / reviewer-blocking ambiguity.

A fresh exact-head review challenged whether a probe lease bound as `_lease` was visibly retained across the targeted SQLite transition. Rust drop scopes already keep a named local alive through its match arm, but this is a security-sensitive ownership invariant and must not depend on an implicit reading of lifetime/drop behavior.

The final code binds the acquired probe as `lease`, executes the complete targeted reconciliation first, then calls `drop(lease)` explicitly. Startup reconciliation therefore makes the required ordering mechanically obvious: acquire lease -> reconcile/finalize exact execution ID -> release lease.

### T066-F6 — display ownership proof must follow the snapshot read

**Classification:** correctness / point-in-time falsely-live display window.

A pre-read ownership check is insufficient on its own: a live owner can finish or crash after the check but before `winds execution` reads the non-final row. The final display path therefore reads the snapshot first and, only if that snapshot remains `REQUESTED` or `RUNNING`, probes the exact execution lease afterward. A busy lease proves the just-read non-final snapshot had a live Winds owner at the point immediately after the read. If the lease can instead be acquired, Winds retains it, reconciles/finalizes that exact execution ID, releases the lease, and loops to read a new final snapshot.

This removes the owner-disappears-between-proof-and-read window without claiming that a process can never exit immediately after any point-in-time observation.

### T066-F7 — active-owner proof still required a post-proof snapshot refresh

**Classification:** correctness / stale non-final JSON after owner finalization.

A later exact-head review found a narrower display race: `winds execution` could build a non-final snapshot, observe the exact lease as busy, and then return the earlier snapshot even if the owner completed finalization between those two operations. The lease proof was valid at its instant, but the returned JSON could unnecessarily lag durable final truth.

The final display branch therefore performs a fresh `execution_snapshot` after `LeaseProbe::Active` rather than returning the pre-probe snapshot. If the owner finalized in that window, the durable final state is returned. If the refreshed state is still non-final, it remains a point-in-time observation made immediately after a live-owner proof; Winds still does not claim that a process cannot exit after any observation.

### T066-F8 — nested ownership-directory validation/open introduced a new path-swap surface

**Classification:** blocking correctness / lease rendezvous path integrity.

An earlier repair created and validated a dedicated `WINDS_HOME/execution-ownership/` directory and then opened each SQLite lease by a child pathname. Exact-head review correctly observed that validation and open were separate pathname operations: replacing that intermediate ownership directory between them could redirect different Winds processes to different lease rendezvous objects.

The final repair removes that independently mutable intermediate directory from the protocol. Each lease now uses a deterministic `execution-ownership-<domain-separated-sha256>.sqlite3` filename **directly under the already-canonical `WINDS_HOME` state root**, alongside the existing Winds state database boundary. `SQLITE_OPEN_NOFOLLOW` still applies to the lease file itself, and lease files remain retained rather than unlinked.

This closes the **new child-directory TOCTOU introduced by T066** without inventing an unstable platform-specific directory-handle abstraction. It does not claim protection against a hostile actor that can concurrently replace the entire canonical Winds state root itself; such a filesystem-sandbox threat would also apply to `winds.db` and the pre-existing state model and is outside Spec 003/T066. T066 therefore does not broaden the repository's filesystem-adversary claim.

## Final repair design

The T066 repair keeps reconciliation at the **user-facing CLI process boundary**, not inside generic `Store::open`, so a second Store connection inside one live process cannot silently revoke owned execution state.

For current Spec 003 CLI execution surfaces:

- `run` and `terminal-proof` acquire a per-execution cross-process ownership lease before creating the execution record and hold it until their owned execution path has finalized or unwound;
- lease names are deterministic domain-separated SHA-256 identifiers rather than caller-controlled paths;
- each lease is a dedicated stable SQLite file directly under canonical `WINDS_HOME`; its retained `BEGIN IMMEDIATE` transaction represents liveness, no PID is persisted, and no stale PID is ever signaled;
- no independently validated/opened ownership child directory remains, removing the T066-introduced child-directory path-swap surface;
- lease files are retained after release rather than unlinked, preventing pathname/inode split-brain races;
- independent execution IDs use independent lease files, so this is not a global single-execution mutex;
- CLI restart reconciliation snapshots non-final execution IDs, then handles each ID independently;
- a busy lease proves another Winds process currently owns that execution lifecycle, so that row is not modified;
- an acquired probe lease is explicitly retained until after the targeted exact-ID SQLite transition completes, preventing a same-ID owner from appearing while the row is reconciled;
- targeted reconciliation updates only that exact execution ID, so executions created after the snapshot cannot be swept into a bulk ownership-loss update;
- durable `WINDS_OBSERVED` shell-command exit facts are finalized to `EXITED` rather than discarded as ownership loss;
- otherwise-unowned `REQUESTED`/`RUNNING` terminal and explicit-command rows transition immediately to `OWNERSHIP_LOST` with unknown process end/duration;
- `winds execution` reads a snapshot first, proves a live owner after any non-final read, refreshes the snapshot after a busy-owner proof, and otherwise performs targeted reconciliation under the acquired lease before re-reading final truth.

The targeted transition intentionally remains local to the T057 CLI restart boundary. It writes only the existing execution/session/event schema and reproduces the already-canonical restart state vocabulary; it does not change candidate/evidence authority or introduce a new public runtime protocol.

This is a process-ownership proof only. It is not a daemon, detached-session protocol, PID recovery mechanism, sandbox, hostile state-root replacement defense, or remote runtime.

## Review results by required axis

### 1. PTY/process ownership

The retained `TerminalSession` child handle remains the authority for live terminal lifecycle operations. Unix interrupt validates the PTY foreground process group against the retained child session before `killpg`. Terminate/close act only through directly retained child handles. Bounded close that cannot prove reaping becomes ownership loss rather than success.

T066 repairs the missing transition between in-process ownership and later CLI-process restart truth. Cross-process leases prove only whether another Winds process still owns the execution lifecycle; they do not grant process signaling authority. Per-ID targeted reconciliation prevents both revoking a distinct live owner and sweeping a newly created live row into a stale-state update. Display-time proof is taken after reading any non-final snapshot, and the snapshot is refreshed after a busy-owner proof.

### 2. Stale PID reuse

No PID column was added and no PID lookup/signaling path was introduced. Existing T060 coverage proves PID-shaped stale metadata cannot cause an unrelated live process to be signaled. The T066 repair uses SQLite transaction ownership keyed by a hashed execution identity, not OS PID identity.

### 3. Windows / Unix close and interrupt semantics

Unix interrupt remains ownership-scoped to the verified PTY foreground process group. Native Windows continues to fail explicitly for interrupt because an ownership-scoped ConPTY foreground-interrupt primitive was not proven. Native Windows terminate/close remains direct-child-handle based. T066 does not broaden any Windows verification claim.

### 4. WSL path/domain truth

WSL launch remains bound to the selected distribution, exact system `wsl.exe`, exact shell, explicit execution domain, and a mapped-workspace-or-visible-home-fallback strategy. Mapped launch performs effective cwd, Git worktree/common-dir, and exact HEAD attestation before and after terminal start. Mapping mismatch remains visible/fail-closed rather than silently equivalent.

### 5. SQLite partial transitions

Terminal request/session creation remains atomic. Child-start/RUNNING persistence failure retains owned cleanup and conservative repair. Exit-finalization failures do not return false success and are deferred for retry. Existing backend restart behavior retains partial terminal coverage; the T066 user-facing path additionally performs exact-ID ownership-loss transitions without touching unrelated rows.

Explicit shell-command durable exit observations are finalized before an unowned non-final row is classified ownership-lost. The targeted ownership-loss update and its execution event are committed in one immediate SQLite transaction. Terminal close reason is updated in that same transaction when the typed session row exists.

### 6. Shell telemetry source attribution

Explicit command intent/cwd stays `CALLER_REQUESTED`; process exit and Git boundary observations are `WINDS_OBSERVED`. There is no PTY marker parser or arbitrary keystroke inference. Marker-like child output cannot create `SHELL_REPORTED` authority.

### 7. Secret/history persistence

Terminal transcript history remains disabled by default, explicitly opt-in, bounded by per-session and total quotas, and labeled best-effort metadata redaction rather than perfect secret detection. Command history can be disabled. Obvious secret options/assignments and credential-bearing URL metadata are conservatively redacted/sanitized. Full environment snapshots are not persisted. History deletion remains constrained to validated owned descendants.

Execution ownership lease files contain no command, environment, credential, transcript, PID, repository path, or user-provided execution ID text; the filename contains only a domain-separated SHA-256 digest. Empty lease databases may persist locally as coordination artifacts, but their transaction lock—not file existence—is the liveness fact.

### 8. Separation from verification authority

Spec 003 workspace execution, command history, terminal history, Git observations, and the T066 ownership lease remain separate from candidate/evidence/eligibility/promotion tables and from `winds verify/promote/recover` authority. No execution-history fact is promoted into verification eligibility. No 0.1 required-check behavior is weakened.

## T066-specific deterministic fixtures

`tests/t066_restart_reconciliation.rs` is binary-facing and covers:

- stale terminal `RUNNING` -> `OWNERSHIP_LOST`, unknown process end/duration, typed ownership-lost close reason;
- stale explicit command `RUNNING` -> `OWNERSHIP_LOST`;
- explicit command with a durable `WINDS_OBSERVED` exit fact -> `EXITED` rather than discarded truth;
- a concurrently live `winds run` remains `RUNNING` when inspected from another Winds CLI process;
- an unrelated stale same-kind command reconciles to `OWNERSHIP_LOST` immediately while the live owner remains protected;
- the live fixture uses an explicit release-file condition rather than fixed-duration liveness or stdin propagation, and a RAII guard releases/reaps the child on both normal completion and assertion unwinding;
- after the live execution finishes, it remains truthfully `EXITED`.

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
- hostile concurrent replacement of the canonical Winds state root;
- broad OS/network/filesystem sandboxing.

Herdr remains future donor reference only.

## Acceptance gate

T066 may be closed only after the final repaired exact head passes the repository-required deterministic CI and fresh exact-head correctness/safety review finds no remaining blocking issue. Earlier green heads invalidated by later race findings are not acceptance evidence. Any later finding must be reconciled on a new exact head before canonical task truth is updated.
