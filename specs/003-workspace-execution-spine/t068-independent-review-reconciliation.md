# T068 Independent Review Findings Reconciliation

Status: **IN PROGRESS — NOT A T068 CLOSEOUT**

This document records the reconciliation of material independent-review findings raised against the complete Spec 003 implementation surface. It does **not** mark T068 complete, authorize T069, or authorize merge by itself.

## Authority and reviewed history

- Canonical Spec 003 task truth remains `tasks.md`.
- T067 is the last closed canonical task.
- T068 remains open until every gate in this document is satisfied.
- T069 remains not started.
- PR #62 is historical review-only evidence and MUST NOT be merged.
- PR #62 reviewed the historical implementation head `8601b7dbb44582a284813bbd50a44aeb1afd24f1` with tree `1d056bead423f02c62ace10b798ceb5c1a1c191c` and demonstrated that that implementation did not satisfy T068.
- Prior T066/T067 reviews and the PR #62 review do not count as the required fresh review of the repaired final T068 implementation.

## Scope and threat-boundary invariants

The reconciliation MUST NOT expand Spec 003 into a daemon, reconnect protocol, sandbox, multiplexer, renderer, plugin/provider framework, SQL Studio, LLM observatory, agent-fleet runtime, MCP/ACP/A2A runtime, or Herdr runtime integration.

The following claims remain explicitly out of scope:

- hostile-repository sandboxing;
- perfect secret detection;
- PID-based reconnect authority;
- native-Windows authoritative verification support;
- protection against an attacker with independent authority to rewrite the Winds SQLite database or race arbitrary filesystem namespace replacement outside the validated supported-operation boundary.

Supported-path correctness still fails closed where Spec 003 owns the operation. Out-of-scope hostile/manual tampering is not converted into a product claim merely to satisfy a reviewer suggestion.

## Material finding reconciliation

### 1. Persisted Git observations hidden from CLI execution snapshots

**Finding:** command-boundary BEFORE/AFTER `execution_git_observations` were persisted but omitted from `winds run` / `winds execution` output.

**Disposition:** REPAIRED.

The execution snapshot now exposes `git_observations` for shell-command executions, preserving boundary, availability, source, HEAD, branch, detached/dirty state, worktree-state format/digest, and observation time. Terminal snapshots intentionally expose an empty array because Spec 003 does not fabricate command-boundary Git observations for terminal sessions.

### 2. Historical authority verification mutated the candidate checkout

**Finding:** release-candidate historical verification temporarily replaced `tests/walking_skeleton.rs` in the candidate checkout; a fail-fast path could skip restoration.

**Disposition:** REPAIRED.

Historical authority verification now operates in an isolated detached temporary worktree and no longer mutates/restores the candidate checkout in place.

### 3. Failed clone poisoned the reserved destination

**Finding:** failed `git clone` could leave the reserved destination behind and prevent an immediate retry with the same destination.

**Disposition:** REPAIRED.

A failed clone now removes only the validated Winds-reserved real destination directory. Cleanup failure is reported fail-closed and the workspace is not registered. Tests prove destination removal and immediate retry.

### 4. Absolute local clone source could diverge from persisted identity

**Finding:** Winds canonicalized an absolute local source for persisted identity but could pass the original pathname to Git, allowing a symlink retarget between those operations.

**Disposition:** REPAIRED.

For absolute local remotes, the same canonical source identity is now used for both the Git CLI argument and persisted clone-origin identity. The supported-path test covers symlink retargeting between identity capture and Git-argument construction.

### 5. Clone destination pathname TOCTOU

**Finding:** the reviewer challenged replacement of the clone destination between validation and Git invocation.

**Disposition:** RECONCILED WITH THE ESTABLISHED THREAT BOUNDARY.

Winds reserves the destination itself, requires it to remain a real directory, canonicalizes and revalidates its identity before Git invocation and again before registration, and refuses observed identity drift. A replacement that is visible at a validation boundary fails closed.

Spec 003 does **not** claim an OS sandbox or hostile concurrent filesystem-namespace containment against an actor independently able to rename/replace path components between every userspace check and syscall. This finding therefore does not authorize a filesystem broker, sandbox, daemon, or broader runtime redesign. The accepted claim is bounded supported-operation validation, not hostile-filesystem security.

### 6. T057 fixture setup could continue after failed Git setup

**Disposition:** REPAIRED.

Fixture initialization is fail-closed so test setup cannot silently continue with invalid repository authority.

### 7. Terminal termination/drop could block without a bounded proof

**Disposition:** REPAIRED.

Owned-terminal cleanup is bounded. It distinguishes exit observed before cleanup, exit proven after Winds termination, and unproven cleanup. Unproven process state records ownership loss rather than fabricating an exit/interrupt claim.

### 8. Natural exit could be mislabeled as controlled termination

**Disposition:** REPAIRED.

Terminal lifecycle persistence differentiates `ExitedBeforeCleanup` from `Terminated`. Tests hold the shell live before exercising controlled termination so `Interrupted` is only asserted when Winds actually proves termination of the owned child.

### 9. Obsolete deferred terminal finalization could poison future starts

**Disposition:** REPAIRED.

Deferred-finalization retry is resilient to obsolete/already-final rows while preserving fail-closed behavior for material persistence errors.

### 10. Git observation object IDs accepted insufficiently constrained values

**Disposition:** REPAIRED.

Persisted Git observation object IDs are validated before admission rather than accepting arbitrary non-empty values.

### 11. Historical dependency-status wording diverged from the actual portable-pty state

**Disposition:** REPAIRED.

The Spec 003 dependency-status documentation was reconciled with the actual approved `portable-pty` dependency state without broadening runtime scope.

### 12. Windows history ACL claim exceeded the implementation boundary

**Disposition:** REPAIRED / CLAIM NARROWED.

Documentation now states the Windows inheritance boundary explicitly. Spec 003 does not claim a bespoke Windows ACL hardening system that it does not implement.

### 13. T062 real-WSL proof could pass without proving the exact requested test

**Disposition:** REPAIRED.

The exact Cargo-test guard now requires exactly one matching test start and one one-test success summary, is task-marker neutral, and the T062 proof uses the guard for both mapped and fallback production-path launches.

### 14. T062 mismatch proof could misclassify a general WSL outage

**Disposition:** REPAIRED.

A failing mapped `--cd` probe is cross-checked with an independent control WSL command before it can be classified as mapped-workspace rejection. A general distribution failure no longer satisfies the mismatch proof.

### 15. T062 temporary `/etc/wsl.conf` restoration was not sufficiently fail-closed

**Disposition:** REPAIRED.

The backup path is unique per proof invocation, pre-existence is rejected, restoration is in `finally`, and cleanup/restore failure is fatal rather than silently accepted.

### 16. ConPTY proof markers could be satisfied by terminal echo rather than shell execution

**Disposition:** REPAIRED.

Native-Windows markers are assembled by `cmd.exe` rather than sent literally as the input marker, and the start-cwd assertion uses an exact output line. This prevents input echo alone from satisfying the proof.

### 17. WSL discovery command lifetime was unbounded

**Disposition:** REPAIRED.

WSL discovery captures stdout/stderr within fixed per-stream memory bounds, applies a bounded command lifetime, and kills/reaps the owned discovery child on timeout.

### 18. Git observation/status subprocess output or lifetime could be unbounded

**Disposition:** REPAIRED.

Read-only Git observation/status commands now use bounded stdout/stderr capture and a bounded lifetime with owned-child kill/reap. Porcelain-v2 parsing also fails closed on malformed or unsupported record shapes instead of interpreting arbitrary bytes as valid dirty-state evidence.

### 19. `winds execution --repo` compared only the worktree root

**Finding:** a stored execution could share a root string while carrying a different Git common-directory identity.

**Disposition:** REPAIRED.

CLI execution lookup now requires the complete registered Git identity: canonical worktree root **and** Git common directory. A regression test proves root-only equality is insufficient.

### 20. Command `requested_cwd` source attribution lost caller intent

**Disposition:** REPAIRED.

The ledger persists the caller-requested cwd with `CallerRequested` source while execution uses the validated canonical location. This preserves intent without weakening workspace containment validation.

### 21. Observation/lifecycle wall-clock values could regress

**Disposition:** REPAIRED.

Supported lifecycle/observation paths reject a regressing wall-clock sample by recording unknown timing rather than persisting timestamps earlier than already-known request/start boundaries.

### 22. First concurrent history writers could race on `history/` creation

**Disposition:** REPAIRED.

Creation of the shared history root treats only a benign `AlreadyExists` race as idempotent and then revalidates that the path is a real non-symlink directory. Per-session directories remain strict create-new ownership boundaries.

### 23. Native Windows canonical cwd could be rejected by `cmd.exe` and silently fall back

**Finding discovered during reconciliation CI:** Windows canonicalization can yield a verbatim drive path (`\\?\C:\...`), which `cmd.exe` can interpret as an unsupported UNC-style cwd and silently fall back to `C:\Windows`.

**Disposition:** REPAIRED.

Winds keeps the canonical path as terminal identity, converts only an ordinary verbatim drive path to a Win32 drive path at the PTY spawn boundary, and rejects UNC/device forms that cannot be represented safely for this shell-launch contract. Silent fallback to an unrelated cwd is not accepted.

### 24. PR workflows could test GitHub's synthetic merge ref instead of the candidate head

**Finding discovered during reconciliation:** some PR jobs relied on default checkout behavior, which can test `refs/pull/<n>/merge` rather than the PR branch's exact head.

**Disposition:** REPAIRED.

`quality` and every `windows-terminal` job now bind checkout to `github.event.pull_request.head.sha || github.sha` and immediately verify `git rev-parse HEAD` equality. `release-candidate` already used explicit candidate-head binding. Only runs containing the repaired exact-head checkout contract may satisfy T068.

## Suggestions not accepted as new Spec 003 product scope

The following classes of suggestions do not justify runtime expansion in T068:

- adding migration-era constraints solely to defend against direct/manual mutation of an existing SQLite database outside supported Store APIs;
- claiming a reason string for every unavailable environmental observation when the current contract only requires explicit availability/unknown truth;
- turning crate-internal/dormant helper cleanup into a public runtime protocol;
- attempting to make Windows native execution an authoritative `winds verify` path;
- adding hostile-filesystem or hostile-repository sandboxing to close generic pathname TOCTOU claims.

If a supported production/API path can produce false lifecycle, identity, or evidence truth, it remains a T068 bug and must be repaired. The boundary above only rejects claims that require a new threat model or product surface.

## Mandatory remaining gates

T068 remains OPEN until **all** of the following are true on one final repaired exact head/tree:

1. `quality` passes on the exact candidate head.
2. `windows-terminal` passes on the exact candidate head, including native Windows and real WSL2 evidence.
3. `release-candidate` passes on the exact candidate head.
4. Every material Qodo/CodeRabbit/Cubic finding is reconciled on that same final surface.
5. A **fresh independent review** evaluates that repaired exact head/tree after CI is green; historical PR #62 and T066/T067 reviews do not count.
6. Any new material finding from that fresh review is repaired, followed by another complete exact-head CI and fresh review cycle.
7. Zero unresolved material review threads remain.
8. Only then may a separate canonical `tasks.md` closeout check T068.

Until these gates are satisfied:

- PR #63 remains unmerged;
- PR #62 remains historical and unmerged;
- T068 remains unchecked;
- T069 remains unchecked / NOT_STARTED;
- Spec 003 remains NOT_COMPLETE.
