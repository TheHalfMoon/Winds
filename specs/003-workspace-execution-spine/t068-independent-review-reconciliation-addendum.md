# T068 Independent Review Reconciliation Addendum

Status: **IN PROGRESS — NOT A T068 CLOSEOUT**

This addendum records material dispositions discovered after the initial T068 reconciliation record was created. It does not check T068, start T069, authorize merge of PR #62 or PR #63, or change the Spec 003 runtime scope.

## Additional repaired supported-path findings

### A1. Shell-command completion could be persisted without an observed exit fact

**Disposition: REPAIRED.**

`Store::record_shell_command_exit_observation` now rejects an observation when both `exit_code` and `observed_end_unix_ms` are absent. `finalize_shell_command_from_observation` independently requires a durable `WINDS_OBSERVED` exit fact before it can transition the execution to `EXITED`.

A regression test proves that an empty observation leaves the execution `RUNNING` and cannot be finalized, while an observed exit code remains sufficient even when end time is unknown.

### A2. Restart-reconciliation event time could precede request time after wall-clock regression

**Disposition: REPAIRED.**

Shell-command and terminal restart reconciliation now clamp the ownership-loss event timestamp to at least the persisted execution request time. The lifecycle status remains conservative: `OWNERSHIP_LOST` does not fabricate process liveness, death, end time, or duration.

A regression test supplies a deliberately regressed `now_ms` and proves that shell and terminal ownership-loss events are not recorded before their respective request times.

### A3. `create_terminal_session` could attach a terminal child row to a non-terminal execution

**Disposition: REPAIRED.**

The Store API now resolves the referenced execution kind before inserting a `terminal_sessions` row and requires `TERMINAL`. A `SHELL_COMMAND` execution cannot acquire a terminal child row through the supported Store API.

This is intentionally enforced at the API boundary rather than by expanding T068 into a historical-schema rewrite against arbitrary direct SQLite mutation.

### A4. T062 `/etc/wsl.conf` backup did not survive the restart it was intended to prove

**Disposition: REPAIRED.**

Reconciliation CI showed that keeping the temporary `/etc/wsl.conf` backup under `/tmp` was not durable across the WSL terminate/restart cycle. The proof now creates a unique, pre-existence-checked, root-owned backup under `/etc`, requests restrictive creation permissions, restores it in `finally`, and treats restore failure as fatal.

The real Windows Server 2025 + Ubuntu WSL2 proof passed after this repair, including mapped launch, deliberate mapping mismatch, fallback launch, and configuration restoration. Only exact-head runs on the eventual final candidate may satisfy T068.

### A5. Native-Windows canonical drive cwd needed a shell-safe spawn representation

**Disposition: REPAIRED.**

Rust canonicalization may produce a verbatim drive path such as `\\?\C:\...`. That value remains the canonical terminal identity, but an ordinary verbatim drive path is converted to the equivalent Win32 drive path only at the PTY child spawn boundary because `cmd.exe` may otherwise treat the verbatim form as an unsupported UNC-style cwd and silently fall back.

Verbatim UNC/device forms and ordinary UNC forms that cannot satisfy the current native-shell cwd contract remain rejected. The ConPTY test proves the effective cwd through an output-only marker assembled by `cmd.exe`, so input echo or surrounding ANSI terminal traffic cannot satisfy the assertion.

### A6. Unix fallback cleanup could leave an unreaped direct-child zombie

**Disposition: REPAIRED.**

`OwnedProcess::drop` still refuses to signal an unproven numeric process-group identity after ownership may have been lost, but a directly owned child that is still live is now killed and then reaped through a short bounded `try_wait` loop. The destructor therefore does not introduce an unbounded wait while also avoiding a permanent zombie when the direct child can be reaped promptly after `SIGKILL`.

### A7. macOS `RLIMIT_NPROC` containment broke legitimate Git descendants

**Disposition: REPAIRED / CLAIM NARROWED.**

The previous macOS path used `RLIMIT_NPROC=2`, but that limit is accounted per real user rather than per Winds-owned process tree and can prevent Git from creating legitimate subprocesses during `--ignore-submodules=none` status scans. That limit is removed.

On macOS, the supported-path ownership contract is the session/process-group boundary created by `setsid`; normal Git descendants inherit that group and bounded cleanup terminates/reaps the group. Winds does not claim hostile descendant-escape containment on macOS. Linux retains the narrower seccomp rule that denies descendant `setsid`/`setpgid` escape for the bounded read-only Git path.

A macOS regression permits a normal descendant, proves that it keeps the owned process scope non-quiescent, then proves bounded group termination and reap.

### A8. Clone staging shells and foreign staging entries could create persistent availability problems

**Disposition: REPAIRED.**

A successful clone now removes its proven-empty private staging shell with a non-recursive `remove_dir`; no recursive cleanup is introduced. Failed clone payload remains retained for recovery when safe cleanup cannot be proven.

The retained-payload admission gate is now scoped to staging names owned by the current Winds process identity. Another process or user's `0700` staging directory is not read and cannot become a global availability gate for every clone under a shared parent. Current-process retained payload still bounds repeated allocation during that process lifetime.

### A9. Clone publication guarantees exceeded what every supported platform documents

**Disposition: REPAIRED / SUPPORTED FILESYSTEM CONTRACT EXPLICIT.**

Linux continues to use `renameat2(..., RENAME_NOREPLACE)` and now reports `ENOSYS`, `EINVAL`, `EOPNOTSUPP`, and `EXDEV` as an explicit unsupported kernel/filesystem publication boundary instead of collapsing them into a generic failure. macOS continues to use `renamex_np(..., RENAME_EXCL)`.

On Windows, staging and requested destination are siblings under the same canonical parent and `MoveFileExW` is invoked without `MOVEFILE_REPLACE_EXISTING` and without `MOVEFILE_COPY_ALLOWED`. The supported claim is therefore **single same-parent no-replace rename plus post-publication filesystem-identity verification**, not a formal cross-filesystem atomicity guarantee that Microsoft does not document for `MoveFileExW`. If the platform/filesystem cannot perform that rename, the clone fails before workspace registration.

Microsoft documents the no-replace behavior for its handle-based rename surface (`FILE_RENAME_INFO.ReplaceIfExists = FALSE` returns an error when the target exists) and documents that file-information-class behavior can vary by underlying driver. Those facts reinforce the narrowed Winds claim: no separate check/delete/replace fallback is accepted as equivalent to a stronger universal atomicity guarantee.

Primary references re-verified 2026-08-20:

- https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_rename_info
- https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-setfileinformationbyhandle

### A10. Post-exit WSL pipe drain could consume an already-expired command deadline

**Disposition: REPAIRED.**

Once the direct `wsl.exe` child has exited, stdout/stderr drain and owned-scope quiescence are cleanup work. They now use the reserved cleanup deadline rather than the command-phase deadline, avoiding spurious zero-budget reader failures when the child exits near the execution deadline.

### A11. Large dirty worktrees were conflated with complete Git evidence capture

**Disposition: REPAIRED.**

The bounded Git reader now keeps only the configured byte cap while continuing to drain the pipe, and records whether stdout was truncated. Callers that require complete Git bytes or exact worktree-state digest still fail closed on truncation. Cleanliness checks use a distinct presence semantic: any stdout, including truncated stdout, proves the worktree is dirty instead of turning a large dirty repository into an infrastructure error.

### A12. Stricter object-ID validation could make historical Git observations unreadable

**Disposition: REPAIRED.**

New Git observation writes continue to require full lowercase 40- or 64-hex object IDs. The read path now preserves historical compatibility by accepting a non-empty legacy stored object-ID string, including pre-T068 abbreviated or uppercase values, while retaining the rest of the stored-observation consistency checks. New admission is not weakened.

### A13. One deferred terminal-finalization persistence failure could poison unrelated future starts

**Disposition: REPAIRED.**

Deferred-finalization retry still preserves the affected historical execution in the in-memory retry queue when Store load or finalization persistence fails; it does not fabricate a terminal final state. The retry sweep now reports the residual failure but returns success to the unrelated terminal-start path, so one permanently unfinalizable historical row cannot block every subsequent terminal session. Obsolete/already-final rows continue to be discarded as completed.

## Historical evidence attribution clarifications

### H1. `SC-001 100-cycle soak` references in Spec 003 task evidence

**Disposition: RECONCILED AS HISTORICAL ATTRIBUTION, NOT RENAMED TO SPEC 003 SC-005.**

The repeated historical phrase `SC-001 100-cycle soak` in task evidence refers to **Spec 001 / SC-001**, whose pre-release gate is 100 create/verify/promote/reconcile cycles with zero source-checkout mutation. It is not a claim that Spec 003 / SC-001 is the terminal soak.

Spec 003 defines its terminal lifecycle soak as **SC-005**. T063 separately records the dedicated 100-cycle terminal lifecycle soak and also records the legacy Spec 001 SC-001 verification soak as an additional gate. This separation is preserved rather than rewriting historical evidence to a criterion it did not execute.

### H2. `docs/research/006-agent-fleet-donor-audit.md` portable-pty wording

**Disposition: HISTORICAL T043 SNAPSHOT; CURRENT STATUS IS SUPERSEDED BY CANONICAL T050 EVIDENCE.**

The donor-audit paragraph stating that `portable-pty 0.9.0` was not yet landed records the T043 decision state at the time of that research audit. It must not be read as current dependency status or as an outstanding instruction.

Current repository truth is the later canonical T050 state: `portable-pty = "=0.9.0"` is landed, the resolved `Cargo.lock` is committed, and the exact transitive/license audit is recorded in `docs/provenance/portable-pty-0.9.0-lock-audit.md`. The reconciled Spec 003 dependency-decision documentation reflects that current state.

The historical donor audit remains a provenance snapshot; it does not override later canonical task evidence or authorize a bespoke PTY implementation.

## Findings that do not authorize new T068 scope

Direct/manual mutation of the local SQLite file is outside the supported Store API and is not converted into a hostile-database security claim. Therefore T068 does not rewrite already-landed migrations solely to add constraints against arbitrary direct SQLite inserts or contradictory manual row edits. Supported API paths that could create false typed or lifecycle truth remain bugs and have been repaired above.

Likewise, T068 does not add a daemon, broker, sandbox, renderer, multiplexer, public runtime protocol, plugin/provider system, SQL/LLM runtime, Agent Fleet runtime, or Herdr integration to answer generic pathname, hostile-repository, or local-database tampering scenarios outside the established Spec 003 boundary.

## Remaining mandatory gate

This addendum is evidence of reconciliation only. T068 remains open until one unchanged final candidate head/tree simultaneously has:

1. complete exact-head `quality`, `windows-terminal`, and `release-candidate` success;
2. all material Qodo, CodeRabbit, Cubic, and reconciliation-discovered findings accounted for on that same surface;
3. a **fresh independent exact-head review performed after the final CI-green repair head exists**;
4. any new material finding repaired followed by a new full CI and fresh-review cycle; and
5. zero unresolved material review threads.

Until then, PR #63 remains unmerged, PR #62 remains historical and unmerged, T068 remains unchecked, T069 remains NOT_STARTED, and Spec 003 remains incomplete.
