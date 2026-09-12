# Spec 009 Tasks Amendment 001 — T121 Inherited Terminal Drop Fixture Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 003 terminal lifecycle truth, canonical Spec 007 Amendments 005-007 bounded-cleanup fixture precedents, canonical Spec 009 Tasks exact-head acceptance requirements, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

Spec 009 T121 is implemented on PR #177, but it cannot close because exact-head repository `quality` attempt 1 failed on an inherited terminal lifecycle fixture outside the T121 authorized source surface.

The blocked T121 candidate is:

```text
PR=177
BASE=53006852765083f582adf6754e95b4a843c6847c
HEAD=ac1b8de9aba52431290e27a054edaf0dc20c4a4d
TREE=f98b72949731df107c540eaf5090c955f34f90ae
QUALITY_RUN=34677828080
QUALITY_RUN_NUMBER=1243
QUALITY_RUN_ATTEMPT=1
QUALITY_RESULT=FAILURE
FAILED_JOB=103510765044
FAILED_PLATFORM=ubuntu-latest
FAILED_TEST=execution::tests::dropping_live_terminal_records_only_proven_cleanup_truth
FULL_TEST_RESULT=609 passed; 1 failed; 6 ignored
OBSERVED_UNEXPECTED_STATUS=Exited
```

The same exact T121 head passed:

```text
QUALITY_MACOS_JOB=SUCCESS
WINDOWS_TERMINAL_RUN=34677828086 / #831 / SUCCESS / attempt 1
RELEASE_CANDIDATE_RUN=34677828084 / #842 / SUCCESS / attempt 1
T097_PERFORMANCE_RUN=34677828130 / #69 / SUCCESS / attempt 1
INDEPENDENT_CODERABBIT_REVIEW=NO_NEW_BLOCKING_ISSUE
```

The failed `quality` attempt remains material evidence. It MUST NOT be rerun into acceptance, erased, or classified as a flake.

## Diagnosis

The failure is an inherited test expectation defect, not a T121 implementation defect.

`src/execution.rs` is byte-identical between canonical T121 base `53006852765083f582adf6754e95b4a843c6847c` and blocked T121 head `ac1b8de9aba52431290e27a054edaf0dc20c4a4d`. Canonical base push `quality #1238` previously succeeded. That earlier green result does not erase the current failure; it only proves T121 did not introduce or modify this terminal lifecycle code.

Canonical production `TerminalExecution::drop` already maps the three bounded cleanup outcomes distinctly:

1. `TerminalDropCleanupOutcome::ExitedBeforeCleanup` -> durable `EXITED` with `PROCESS_EXITED` close reason;
2. `TerminalDropCleanupOutcome::Terminated` -> durable `INTERRUPTED` with `CLOSED_BY_WINDS` close reason;
3. `TerminalDropCleanupOutcome::Unproven` or cleanup error -> durable `OWNERSHIP_LOST` with unknown process state and no fabricated end/duration.

Canonical Spec 003 reconciliation explicitly requires this distinction. It states that owned-terminal cleanup differentiates exit observed before cleanup, exit proven after Winds termination, and unproven cleanup, and that natural exit MUST NOT be mislabeled as controlled termination.

The historical commit that introduced `dropping_live_terminal_records_only_proven_cleanup_truth`, `1a95834a320a703597937459fe6827751f3a6fe7`, was itself titled `test(winds): preserve truthful terminal cleanup outcomes`. The fixture was changed to accept `INTERRUPTED` and `OWNERSHIP_LOST`, but it did not add the already-canonical `EXITED` / `PROCESS_EXITED` branch.

The T121 Ubuntu run then observed exactly that omitted truthful production branch. The product did not claim false cleanup success or false ownership. The fixture rejected valid durable truth.

No evidence justifies changing production terminal cleanup, the 500 ms cleanup budget, PTY behavior, kill/reap ordering, ownership revocation, lifecycle persistence, or T121 product behavior.

## Why the simpler paths are insufficient

The Spec 009 Standard Acceptance Gate requires repository `quality = SUCCESS` on every platform job defined by that workflow for the exact final candidate. Therefore T121 cannot merge with `quality #1243` failed.

The following apparent shortcuts are prohibited or insufficient:

- rerunning `quality #1243` or rerunning the same T121 head until the inherited race produces green;
- classifying the observed `EXITED` result as a flake;
- suppressing or ignoring the failing test;
- weakening the quality workflow;
- widening production cleanup timing;
- changing T121 Model Mesh code to influence unrelated terminal timing;
- merging PR #177 while the exact-head quality gate is red.

The smallest reversible path is a separately qualified governance amendment authorizing one test-only truth reconciliation, followed by a separately qualified repair on canonical `main`, then forward-only integration of that repaired main into PR #177 and complete T121 requalification.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` is `53006852765083f582adf6754e95b4a843c6847c` or a governance-only forward descendant;
- T120 remains `CLOSED_CANONICAL`;
- T121 remains open and not `CLOSED_CANONICAL`;
- PR #177 remains the active T121 candidate or a forward-only successor;
- failed exact-head `quality` run `34677828080` remains preserved as attempt-1 FAILURE;
- the only failed test in that Ubuntu job is `execution::tests::dropping_live_terminal_records_only_proven_cleanup_truth` with observed durable status `Exited`;
- `src/execution.rs` remains unchanged by the blocked T121 candidate relative to its canonical base;
- canonical production `TerminalExecution::drop` still maps `ExitedBeforeCleanup`, `Terminated`, and unproven/error cleanup to distinct `EXITED`, `INTERRUPTED`, and `OWNERSHIP_LOST` truth;
- no later canonical repair has already removed the need for this amendment.

T122 and every later Spec 009 task remain dependency-blocked while T121 is open.

## Exact additional repair authority

Only after this amendment is canonically landed may one repair candidate modify:

```text
src/execution.rs
```

and only inside:

```rust
#[test]
fn dropping_live_terminal_records_only_proven_cleanup_truth()
```

The repair may add exactly the missing truthful `ExecutionStatus::Exited` branch to the existing final-status match.

For the `EXITED` branch, the fixture MUST prove:

- `final_record.status == ExecutionStatus::Exited`;
- `final_record.status_source == FactSource::WindsObserved` remains satisfied by the existing outer assertion;
- `final_record.started_unix_ms` remains present;
- `final_record.ended_unix_ms` is present;
- `final_record.duration_ms` is present;
- `terminal.close_reason == Some(TerminalCloseReason::ProcessExited)`;
- zero `CLOSED_BY_WINDS`, `INTERRUPTED`, or `OWNERSHIP_LOST` claim is fabricated for a child observed exited before cleanup.

The existing `INTERRUPTED` branch MUST continue to require `CLOSED_BY_WINDS` plus present end/duration truth.

The existing `OWNERSHIP_LOST` branch MUST continue to require `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN` plus absent end/duration truth.

Any other status MUST continue to fail the fixture.

The repair SHOULD be the smallest branch-only source change. Renaming the test, adding sleeps, adding retries, changing setup timing, adding a liveness barrier, changing process commands, or changing production implementation is not authorized by this amendment.

## Values and behavior that MUST remain unchanged

This amendment does NOT authorize changes to:

- any production statement in `src/execution.rs`;
- `TerminalExecution::drop`, `controlled_cleanup`, `try_wait`, `wait`, `terminate`, or `close`;
- `TerminalSession` or `TerminalDropCleanupOutcome` semantics;
- `TerminalFinalization`, `ExecutionStatus`, `FactSource`, or `TerminalCloseReason` production mappings;
- the 500 ms terminal cleanup window;
- kill/reap order, polling cadence, process ownership proof, or ownership revocation;
- SQLite schema, migrations, persistence logic, or restart reconciliation;
- T121 Model Mesh implementation, CLI behavior, schema, tests, or authority boundaries;
- any workflow, runner image, dependency, lockfile, performance threshold, platform claim, provider/runtime authority, verification authority, Git authority, or landing authority;
- Spec 006 live-runtime nonclaims.

No timeout increase, sleep, retry loop, rerun-until-green strategy, ignored test, platform skip, assertion suppression, or conversion of unproven cleanup into proven success is authorized.

## Governance amendment acceptance gate

This file is governance-only and is not canonical merely because it exists.

Its exact final candidate MUST satisfy:

- changed scope exactly this one amendment document;
- repository `quality` SUCCESS on the exact amendment head;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact amendment candidate;
- zero unresolved material findings and review threads;
- exact current main/base/head/tree/ruleset/mergeability reconciliation immediately before landing;
- guarded normal merge using the exact expected head;
- merge tree, ordered parents, and GitHub signature verification;
- every actually-triggered applicable post-merge push workflow verified before the amendment is considered canonical.

If the amendment candidate itself encounters the same terminal fixture failure before this repair authority exists, that failure remains material and execution MUST stop for another explicit governance decision. It MUST NOT be rerun away.

## Required evidence after amendment use

After this amendment is canonical, the test-only repair MUST start from then-current exact canonical `main` and qualify from scratch.

Required repair evidence includes:

- changed implementation scope exactly the authorized test function in `src/execution.rs`;
- focused execution of `execution::tests::dropping_live_terminal_records_only_proven_cleanup_truth` PASS;
- repository `quality` SUCCESS on Ubuntu and macOS on the exact repair head;
- all actually-triggered terminal/platform/release/performance workflows qualified on that exact repair head;
- no production source semantic change;
- no timeout, sleep, retry, workflow, dependency, schema, migration, or authority change;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate;
- zero unresolved material findings and review threads;
- live main/base/head/tree/ruleset/mergeability race reconciliation;
- guarded expected-head landing;
- successful applicable post-merge push checks on repaired canonical main.

The historical failed T121 `quality #1243` attempt remains material evidence after repair and MUST be retained in final Spec 009 reconciliation.

## T121 continuation after repair

Landing this amendment does not close T121. Landing the inherited fixture repair does not close T121.

Only after the repair is canonical and post-merge verified may PR #177 continue. It MUST then:

1. fetch the repaired canonical `main`;
2. forward-integrate canonical `main` into the existing T121 branch through a normal merge commit, without rebase, force-push, squash-history rewrite, or dropping historical T121 commits;
3. preserve the T121 implementation and every prior forward-only CodeRabbit repair;
4. treat all pre-integration T121 CI and reviews as historical/stale candidate evidence;
5. re-run the complete T121 local focused/regression qualification on the new exact head;
6. obtain repository `quality` SUCCESS and all other applicable exact-head workflow evidence from scratch;
7. obtain fresh exact-head author, Ponytail/YAGNI, and independent substantive review;
8. reconcile exact changed scope against the repaired canonical base;
9. merge only through the existing guarded expected-head normal-merge discipline;
10. verify canonical post-merge push checks before T122 authority is asserted.

No T122 work becomes authorized merely because this amendment or the inherited fixture repair lands.

## Explicit non-authorization

This amendment does not authorize:

- rerunning failed T121 `quality #1243` as a substitute for repair;
- classifying the observed `EXITED` result as a flake;
- forcing natural `EXITED` truth to `INTERRUPTED`;
- changing any production terminal behavior;
- weakening cleanup proof or ownership-loss semantics;
- widening cleanup timeouts or observation budgets;
- retrying cleanup or control operations;
- disabling, ignoring, filtering, or suppressing the failing test;
- changing CI workflow semantics;
- changing T121 Model Mesh behavior or scope;
- starting T122 before T121 canonical closure;
- daemon/IPC/server/plugin/provider-framework/browser/remote-execution/learning expansion;
- automatic provider routing, automatic winner selection, or automatic Git landing.

## Acceptance of this amendment

Only after exact-candidate governance qualification, guarded expected-head landing, and post-merge verification may repository truth state:

```text
SPEC_009_TASKS_AMENDMENT_001=CLOSED_CANONICAL
T121_INHERITED_TERMINAL_DROP_FIXTURE_REPAIR=AUTHORIZED
T121_INHERITED_TERMINAL_DROP_FIXTURE_REPAIR_LANDED=NO
T121=CANDIDATE_BLOCKED_PENDING_REPAIR
T122=BLOCKED_BY_DEPENDENCY
```
