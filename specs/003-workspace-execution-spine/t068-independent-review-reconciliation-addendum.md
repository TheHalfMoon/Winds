# T068 Independent Review Reconciliation Addendum

Status: **T068 CLOSEOUT EVIDENCE RECORDED — PR #63 REMAINS UNMERGED; T069 NOT STARTED**

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

### A14. History pruning could recursively delete a foreign replacement after validation

**Disposition: REPAIRED / OBJECT-BOUND NON-RECURSIVE PRUNING RESTORED.**

A fresh independent exact-head review of candidate `5c3a646d196abd33b96468bb95b597fc5da6fdd8`, tree `428d0949647b75d626dc048382770f9740e7bce0`, found one new material P1: `remove_owned_history_session` validated a retained history directory and then called `fs::remove_dir_all` through that mutable pathname. A concurrent pathname replacement after the last validation could therefore redirect recursive deletion to a foreign replacement directory. The reviewer reported no additional material finding in the other inspected T068 surfaces.

The first repair removed automatic pruning entirely and failed closed when retained history could not fit the next record. That removed the recursive-delete race, but it was not behaviorally acceptable: exact-head `release-candidate #379` on candidate `c25e6fe9a479e368c2c45ad3452ab292d9172866` failed the Ubuntu T063 100-cycle terminal soak when retained history reached `64962` bytes and the next `1203`-byte record exceeded the `65536`-byte total quota. That result proved oldest-session rollover is load-bearing behavior and that the no-prune fallback could not become the T068 disposition.

The current repair therefore restores the original oldest-session retention policy while changing the destructive primitive. Production history pruning no longer uses `remove_dir_all`. Each retained session is snapshotted as a flat, content-addressed session directory with filesystem identity, logical size, modification time, and direct known history files. Unexpected directory entries, nested objects, symlinks/reparse points, duplicate transcript/manifest blobs, invalid names, and identity changes fail closed.

On Unix, Winds opens the history root and selected retained session as no-follow directory handles, verifies their filesystem identities, validates each direct regular-file entry relative to the already-open session directory, unlinks only direct names through `unlinkat`, revalidates the session entry from the already-open root directory, and finally removes the now-empty session directory non-recursively with `unlinkat(..., AT_REMOVEDIR)`. The security claim is deliberately scoped to containment inside the already-bound session-directory object and to the supported Winds writer path. POSIX `unlinkat` remains name-based: Winds does **not** claim protection when an external same-principal process concurrently replaces an individual direct child name inside that private session directory between validation and unlink. Such hostile same-principal filesystem mutation is outside the Spec 003 isolation claim. Winds still guarantees that pruning performs no recursive traversal and cannot redirect deletion into another directory tree through that child-name race.

On Windows, the same flat-session policy is bound to filesystem object identity using no-follow/reparse-point-aware handles and `GetFileInformationByHandleEx`; direct files and the empty session directory are marked for deletion by handle with `SetFileInformationByHandle`. Unsupported object types or identity changes fail closed.

A regression hook runs immediately after the last pathname-based session identity proof. The test moves the originally observed session directory, creates a foreign replacement at the original pathname, and then enters the destructive stage. Pruning rejects the identity mismatch; the foreign replacement and the moved owned session both remain unchanged. Additional regression coverage proves a valid owned flat session is pruned non-recursively and that oldest-session rollover again permits the next bounded history record.

The repair was generated, formatted with pinned Rust `1.97.1`, and tested in GitHub Actions before publication. `quality #592` / run `32396853626` produced artifact `t068-history-prune-repair` with GitHub-recorded digest `sha256:2763fab5f03482940b313f83daeb720f323af2603b4600553263e9a25b1cde3a`; its focused history suite passed `17/17`. The published Git blobs for `src/command/history.rs` and `src/command/history/history_prune.rs` were independently matched to the exact formatted artifact before the temporary repair scaffolding was removed.

All deterministic CI and independent-review results from earlier candidates remain historical and MUST NOT satisfy the T068 final gate. The cleaned candidate that includes this repair and this addendum requires a complete new exact-head `quality`, `windows-terminal`, and `release-candidate` cycle followed by a fresh independent exact-head review.

### A15. WSL post-exit drain could spin indefinitely while inherited pipes remained continuously readable

**Disposition: REPAIRED / FINAL EXACT-HEAD REVIEW CLEAN.**

A fresh CodeRabbit review of implementation head `f77362ec658c2b3ac1c5c2a99c454eb59a0b7448` identified one remaining material post-exit availability defect in `src/wsl_launch.rs`: after the direct `wsl.exe` child exited, `drain_pair` could continue returning progress while descendants kept inherited stdout/stderr pipes readable, allowing the post-exit drain loop to outlive the reserved cleanup window. The same review separately raised a detached-`setsid` concern, then withdrew it after tracing the production call graph and confirming that the arbitrary-command fixture was not reachable through the supported WSL launch surface. No containment expansion was required for that withdrawn concern.

The drain repair reserves only half of the remaining cleanup budget for post-exit pipe draining and routes the loop through `drain_until_idle_or_deadline`. The helper checks its deadline before each drain attempt, returns `Ok(false)` if continuous progress reaches the drain deadline, and returns `Ok(true)` only when the drain reports no further progress. A drain-deadline miss immediately invokes bounded `terminate_and_prove(cleanup_deadline, ...)` and returns an explicit error stating that WSL-side cleanup proof cannot be trusted; the subsequent Windows process-scope quiescence check is also bounded by `cleanup_deadline`.

The first real-WSL regression fixture attempted to force continuous progress with an escaped writer. Exact-head candidate `ee79131a9752146b072fb60b176b8d5db21f2fad` correctly remained bounded but the fixture was scheduling-dependent: real Windows+Ubuntu WSL2 T062 returned after about five seconds through bounded Windows-scope cleanup rather than the specific drain-deadline branch the test expected. That candidate was rejected rather than weakening the gate. The final regression is deterministic and directly exercises the load-bearing loop property: `post_exit_drain_stops_at_deadline_under_continuous_progress` supplies a drain closure that returns `Ok(true)` continuously and proves the helper exits at its deadline instead of spinning indefinitely. Real WSL2 integration remains separately covered by T062.

The final reviewed implementation candidate is HEAD `badfa984d7aa5552478aaba5b7da5819290253df`, tree `d5e6ffcdd97af9cf0281c2606f799fb88b9e6b0e`, against unchanged canonical base `29c394084631afd6d1890362372b8a162dac083a`, with `behind_by=0`. On that exact head:

- `quality #613` / run `32407334800` = **SUCCESS**;
- `windows-terminal #338` / run `32407334815` = **SUCCESS**, including native Windows, Unix terminal integration, and real Windows Server 2025 + Ubuntu WSL2 T062 production proof/evidence;
- `release-candidate #405` / run `32407334775` = **SUCCESS**, including T063 100-cycle terminal lifecycle soak on Ubuntu/macOS/Windows, T064 regression gates, SC-001, native-Windows authority refusal, quality, and release builds;
- the final CodeRabbit post-exit-drain material thread was reconciled against this exact head and resolved by CodeRabbit; zero material review threads remain unresolved; and
- fresh independent Qodo full-implementation review, explicitly bound to HEAD `badfa984d7aa5552478aaba5b7da5819290253df`, tree `d5e6ffcdd97af9cf0281c2606f799fb88b9e6b0e`, and base `29c394084631afd6d1890362372b8a162dac083a`, returned **NO MATERIAL FINDING REMAINING**. Qodo specifically re-evaluated the bounded WSL drain, object-bound history pruning, clone staging cleanup, and the complete current implementation surface.

An additional CodeRabbit incremental re-review of the final `src/wsl_launch.rs` delta was requested after the clean Qodo verdict. It is not required or counted as the independent pass unless it completes on the bound head; any later material finding from that run still invalidates closeout and must be reconciled before merge.

This closes the T068 independent-review requirement on the reviewed implementation head. The documentation-only closeout commit that records this fact is not a new runtime implementation candidate and does not authorize merge by itself: PR #63 remains draft/unmerged until its own final exact-head CI/review gate is green. T069 remains **NOT STARTED**, and Spec 003 remains incomplete until T069 is separately executed and canonically reconciled.

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

## T068 gate result

**T068 independent-review gate: SATISFIED on reviewed implementation head `badfa984d7aa5552478aaba5b7da5819290253df`.**

The required exact-head implementation evidence is complete: all three deterministic CI/platform workflows succeeded on the same head; all material Qodo, CodeRabbit, Cubic, and reconciliation-discovered findings are accounted for; fresh independent Qodo review returned `NO MATERIAL FINDING REMAINING` on the exact head/tree/base; and zero material review threads remain unresolved.

This addendum and the matching `tasks.md` update are documentation-only closeout evidence. They do not merge PR #63, do not start T069, and do not make the Spec 003 completion claim. Because they create a new documentation-only PR head, that final head must still pass the repository's exact-head CI/review landing gate before merge authorization can be considered. Any new material finding invalidates the closeout candidate and requires reconciliation plus a new exact-head cycle.
