# Spec 009 Tasks Amendment 002 — T126 Inherited T060 Proven-Exit Fixture Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 003 T053/T060 terminal lifecycle truth, canonical Spec 007 Amendments 005 and 006 bounded-cleanup fixture precedents, canonical Spec 009 T126 exact-head acceptance requirements, canonical Spec 009 Amendment 001 evidence-preservation discipline, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

Spec 009 T126 has a documentation-only final closeout candidate on PR #184, but that candidate cannot close because its exact-head repository `quality` attempt 1 failed on an inherited T060 terminal lifecycle fixture outside T126's authorized documentation surface.

The blocked T126 candidate is:

```text
PR=184
BASE=d6de382082d79bc939733588749986d6ca8a2fe6
HEAD=f3465500dfdb4a44855419843871e6bf2ebbe2fc
TREE=2daf07d5f1989f9c7a9745e49781d261e9a622ef
QUALITY_RUN=34709700248
QUALITY_RUN_ATTEMPT=1
QUALITY_RESULT=FAILURE
FAILED_JOB=103596067025
FAILED_PLATFORM=ubuntu-latest
FAILED_TEST=git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session
FULL_TEST_RESULT=617 passed; 1 failed; 6 ignored
OBSERVED_STATUS=Exited
OVER_STRONG_EXPECTATION=Interrupted
MACOS_JOB=SUCCESS
```

The failed attempt remains material evidence. It MUST NOT be rerun into acceptance, erased, or classified as a flake.

Fresh CodeRabbit review of exact T126 head `f3465500dfdb4a44855419843871e6bf2ebbe2fc` found no material defect in the T126 reconciliation artifact or checked-state reconciliation, verified all T114-T125 ledger identities and signatures, verified 95/95 FR rows and 25/25 SC rows, and explicitly identified the failed exact-head quality gate as unresolved. That review does not waive the failed gate.

## Diagnosis

The failure is an inherited fixture expectation defect, not a T126 implementation defect. T126 changes only:

```text
specs/009-model-mesh-multi-provider-continuity/tasks.md
specs/009-model-mesh-multi-provider-continuity/t126-final-reconciliation.md
```

It changes no production/runtime/test behavior.

Canonical production `TerminalExecution::controlled_cleanup(...)` already distinguishes three truthful bounded-cleanup outcomes:

1. `TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit)` -> durable `EXITED` / `WINDS_OBSERVED` / `PROCESS_EXITED` truth;
2. `TerminalDropCleanupOutcome::Terminated(exit)` -> durable `INTERRUPTED` / `WINDS_OBSERVED` with the explicit controlled close reason;
3. `TerminalDropCleanupOutcome::Unproven` or cleanup failure -> ownership revocation, durable `OWNERSHIP_LOST` / `WINDS_OBSERVED`, no fabricated end/duration, and an explicit bounded-cleanup failure.

The failing T060 fixture already accepts the third outcome because canonical Spec 007 Amendment 005 repaired its original success-only assumption. However, its successful `terminate()` branch still asserts `INTERRUPTED` / `TERMINATED_BY_WINDS` only.

That remaining assumption is stronger than canonical production truth. The child can exit naturally after the fixture's preceding live observation and before `controlled_cleanup()` performs its own cleanup observation. In that interval, production must preserve `ExitedBeforeCleanup` as `EXITED`; coercing it to `INTERRUPTED` would fabricate controlled termination.

Canonical Spec 007 Amendment 006 documents the same lifecycle race for the neighboring T060 fixture and requires natural `EXITED/PROCESS_EXITED`, controlled `INTERRUPTED/CLOSED_BY_WINDS`, and bounded-unproven `OWNERSHIP_LOST` to remain distinct.

No evidence justifies changing production cleanup, the 500 ms cleanup budget, PTY behavior, kill/reap ordering, ownership revocation, persistence, workflow behavior, Model Mesh behavior, or T126 reconciliation content.

## Why the simpler paths are insufficient

Spec 009 requires repository `quality = SUCCESS` on every platform job defined by that workflow for the exact final candidate. T126 therefore cannot merge with run `34709700248` failed.

The following shortcuts are prohibited:

- rerunning `quality` on unchanged T126 head until the inherited race happens to produce green;
- classifying the observed `EXITED` state as a flake;
- suppressing, ignoring, filtering, or weakening the T060 fixture;
- forcing natural `EXITED` truth to `INTERRUPTED`;
- widening cleanup timeouts or adding sleeps/retries;
- changing T126 documentation to influence terminal timing;
- merging PR #184 while its exact-head quality gate is red.

The smallest forward-only path is: land this governance-only amendment; qualify and land one test-only fixture truth repair on canonical `main`; forward-integrate that repaired main into PR #184 through a normal merge commit; then requalify T126 from scratch on the new exact head.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` is `d6de382082d79bc939733588749986d6ca8a2fe6` or a governance-only forward descendant;
- T125 remains `CLOSED_CANONICAL`;
- T126 remains open and not `CLOSED_CANONICAL`;
- PR #184 remains the active T126 candidate or a forward-only successor;
- exact-head `quality` run `34709700248` remains preserved as attempt-1 `FAILURE`;
- the only failed test in the Ubuntu job is `git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session`, with observed durable status `Exited` and fixture expectation `Interrupted`;
- the same exact T126 head's macOS quality job remains successful;
- canonical production `TerminalExecution::controlled_cleanup(...)` still maps natural exit, controlled termination, and unproven cleanup to distinct `EXITED`, `INTERRUPTED`, and `OWNERSHIP_LOST` truth;
- Spec 007 Amendments 005 and 006 remain canonical historical precedents and no later canonical repair has already removed the need for this amendment.

No later Spec 009 implementation phase exists. T126 closeout remains blocked until the repair and full requalification complete.

## Exact additional repair authority

Only after this amendment is canonically landed may one repair candidate modify:

```text
src/t060_fault_tests.rs
```

and only inside:

```rust
#[test]
fn input_and_resize_racing_with_exit_never_reopen_final_session()
```

The authorized repair is limited to preserving the existing setup, input/resize race exercise, successful-final-control rejection, exact returned-exit stability, and existing ownership-loss branch while replacing the successful `INTERRUPTED`-only assertion with strict verification of the two already-existing successful cleanup subtypes.

### Truthful outcome A — child exited before cleanup observed termination

If `execution.terminate()` returns `Ok(final_exit)` and durable finalization follows the existing natural-exit path, the fixture MUST prove:

- post-final input and resize remain rejected;
- `execution.try_wait()` remains `Some(final_exit)`;
- durable execution status is `EXITED`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` are present;
- terminal close reason is `PROCESS_EXITED`;
- no `TERMINATED_BY_WINDS`, `CLOSED_BY_WINDS`, `INTERRUPTED`, or `OWNERSHIP_LOST` claim is fabricated for the naturally exited child.

### Truthful outcome B — Winds proves controlled termination

If `execution.terminate()` returns `Ok(final_exit)` and durable finalization follows the existing controlled-termination path, the fixture MUST prove:

- post-final input and resize remain rejected;
- `execution.try_wait()` remains `Some(final_exit)`;
- durable execution status is `INTERRUPTED`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` are present;
- terminal close reason is `TERMINATED_BY_WINDS`.

### Truthful outcome C — bounded cleanup cannot prove child exit

If `execution.terminate()` returns `Err(error)`, the fixture MUST preserve its current fail-closed assertions:

- the error equals the existing bounded cleanup-window failure text;
- no retry of terminate/close/cleanup is performed to obtain green;
- post-revocation input, resize, and `try_wait()` remain rejected;
- durable execution status is `OWNERSHIP_LOST`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` remain absent;
- terminal close reason is `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`.

Any other lifecycle status, source, timing shape, close reason, cleanup error, or post-final control result MUST fail the fixture.

## Values and behavior that MUST remain unchanged

This amendment does NOT authorize changes to:

- any production statement in `src/execution.rs`, `src/git/terminal.rs`, Store, or lifecycle persistence;
- `TerminalExecution::controlled_cleanup`, `terminate`, `close`, `try_wait`, `wait`, or Drop behavior;
- `TerminalSession` or `TerminalDropCleanupOutcome` semantics;
- `TerminalFinalization`, `ExecutionStatus`, `FactSource`, or `TerminalCloseReason` production mappings;
- the canonical 500 ms terminal cleanup window;
- kill/reap order, polling cadence, process ownership proof, or ownership revocation;
- SQLite schema, migrations, persistence, restart reconciliation, dependencies, lockfiles, or workflows;
- T114-T125 accepted Model Mesh source, schema, CLI, continuity, authority, projection, or adversarial behavior;
- T126 reconciliation semantics except the later forward-integration evidence needed after repair;
- Spec 006 live-runtime nonclaims;
- platform, runtime/provider, verification, acceptance, Git, or landing authority.

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

If the amendment candidate itself encounters a material repository gate failure, that failure remains evidence and MUST NOT be rerun away or silently waived.

## Required evidence after amendment use

After this amendment is canonical, the test-only repair MUST start from then-current exact canonical `main` and qualify from scratch.

Required repair evidence includes:

- changed implementation scope exactly the authorized test function in `src/t060_fault_tests.rs`;
- focused execution of `git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session` PASS;
- repository `quality` SUCCESS on Ubuntu and macOS on the exact repair head;
- every actually-triggered applicable terminal/platform/release/performance workflow qualified on that exact repair head;
- no production source semantic change;
- no timeout, sleep, retry, workflow, dependency, schema, migration, or authority change;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate;
- zero unresolved material findings and review threads;
- live main/base/head/tree/ruleset/mergeability reconciliation;
- guarded expected-head landing;
- successful applicable post-merge push checks on repaired canonical main.

The historical failed T126 `quality` run `34709700248` remains material evidence after repair and MUST be retained in final Spec 009 reconciliation.

## T126 continuation after repair

Landing this amendment does not close T126. Landing the inherited fixture repair does not close T126.

Only after the repair is canonical and post-merge verified may PR #184 continue. It MUST then:

1. fetch the repaired canonical `main`;
2. forward-integrate canonical `main` into the existing T126 branch through a normal merge commit, without rebase, force-push, squash-history rewrite, or dropping historical T126 commits;
3. preserve the T126 reconciliation artifact and checked-state commit history;
4. treat all pre-integration T126 CI and candidate-bound reviews as historical/stale evidence;
5. update the T126 reconciliation artifact only as needed to record the new failed-run/amendment/repair history and then requalify the resulting exact head;
6. obtain repository `quality = SUCCESS` on every platform job actually defined by that workflow for the exact final T126 head;
7. obtain fresh exact-head author, Ponytail/YAGNI, and independent substantive review;
8. reconcile exact changed scope against the repaired canonical base;
9. merge only through guarded expected-head normal-merge discipline;
10. verify canonical post-merge main/tree/ordered parents/signature and every actually-triggered push workflow before asserting T126 closure.

## Explicit non-authorization

This amendment does not authorize:

- rerunning failed T126 `quality` run `34709700248` as a substitute for repair;
- classifying the observed `EXITED` result as a flake;
- forcing natural `EXITED` truth to `INTERRUPTED`;
- changing any production terminal behavior;
- weakening cleanup proof or ownership-loss semantics;
- widening cleanup timeouts or observation budgets;
- retrying cleanup or control operations;
- disabling, ignoring, filtering, or suppressing the failing test;
- changing CI workflow semantics;
- changing Model Mesh product behavior;
- provider/framework/gateway/credential/daemon/IPC/browser/remote-execution/learning expansion;
- automatic provider routing, winner selection, verification, acceptance, Git mutation, merge, or landing.

## Acceptance of this amendment

Only after exact-candidate governance qualification, guarded expected-head landing, and post-merge verification may repository truth state:

```text
SPEC_009_TASKS_AMENDMENT_002=CLOSED_CANONICAL
T126_INHERITED_T060_PROVEN_EXIT_FIXTURE_REPAIR=AUTHORIZED
T126_INHERITED_T060_PROVEN_EXIT_FIXTURE_REPAIR_LANDED=NO
T126=CANDIDATE_BLOCKED_PENDING_REPAIR
SPEC_009_FIRST_IMPLEMENTATION_PROGRAM=CANDIDATE_NOT_CLOSED
```
