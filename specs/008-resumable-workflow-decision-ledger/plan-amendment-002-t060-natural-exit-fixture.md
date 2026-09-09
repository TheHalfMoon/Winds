# Spec 008 Plan Amendment 002 — Inherited T060 Natural-Exit Fixture Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process; canonical Spec 008 entry/spec authority; canonical Spec 008 Plan Amendment 001; canonical Spec 007 Amendment 005's explicit requirement to stop for another governance decision if a new material failure proves its bounded T060 fixture repair insufficient; canonical Spec 003/007 terminal lifecycle truth; and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

PR #144 is the bounded T057 fixture repair authorized by canonical Spec 008 Plan Amendment 001. Exact candidate `f02ed73ecc0129ab1945641f6d4691484ad8c2b5` / tree `62db00187de9375b823bbdac15c90e48942dd574` produced a new material first-attempt failure in repository `quality` run `34401356369`, Ubuntu job `102633761720`.

The full-suite failure was:

```text
FAILED_TEST=git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session
left=Exited
right=Interrupted
```

The macOS `quality` job on the same exact head succeeded. Same-head successes do not erase the Ubuntu failure. Run `34401356369` is material historical evidence, is not classified as a flake, and MUST NOT be rerun into acceptance.

## Why a new governance decision is required

Canonical Spec 007 Amendment 005 authorized only the existing fixture:

```rust
fn input_and_resize_racing_with_exit_never_reopen_final_session()
```

to distinguish proven controlled termination from fail-closed ownership loss. Amendment 005 also states that if a new material failure proves that exact bounded fixture repair insufficient, execution must stop for another explicit governance decision rather than silently widening scope.

The current failure is that explicit condition.

## Diagnosis

Canonical production behavior already distinguishes three terminal cleanup truths without changing the 500 ms cleanup budget:

1. `TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit)` -> durable `EXITED` / `WINDS_OBSERVED` / `PROCESS_EXITED`;
2. `TerminalDropCleanupOutcome::Terminated(exit)` -> durable `INTERRUPTED` / `WINDS_OBSERVED` / `TERMINATED_BY_WINDS` for `terminate()`;
3. `TerminalDropCleanupOutcome::Unproven` -> ownership revoked, durable `OWNERSHIP_LOST` / `WINDS_OBSERVED`, no fabricated end/duration, and `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`.

`TerminalExecution::terminate()` returns `Ok(final_exit)` for both `ExitedBeforeCleanup` and `Terminated`. The current Amendment-005-repaired fixture treats every successful `terminate()` as controlled interruption and therefore asserts `INTERRUPTED` unconditionally.

The observed Ubuntu result `EXITED` is therefore a truthful production state that the inherited fixture does not yet represent. Converting that natural exit into `INTERRUPTED` would be less truthful and is not authorized.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` is `7a935767fa16a6c3c8b64606368802b45f945f32` or a governance-only forward descendant while this amendment is qualifying;
- canonical Spec 008 Plan Amendment 001 remains closed and its bounded T057 repair authority remains active but not yet canonically landed;
- PR #144 remains blocked by exact-head `quality` run `34401356369` rather than being rerun into acceptance;
- the failed Ubuntu job remains attributable to `git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session` with observed `EXITED` versus expected `INTERRUPTED`;
- canonical production `TerminalExecution::controlled_cleanup(...)` still maps `ExitedBeforeCleanup`, `Terminated`, and `Unproven` as described above;
- canonical Spec 007 Amendment 005 remains the authority currently governing this exact fixture;
- no later canonical repair has already removed the need for this amendment.

If any prerequisite changes materially, this amendment must be re-reconciled before use.

## Exact additional repair authority

Only after this amendment is qualified, guarded-merged, and post-merge verified may a separate repair candidate modify:

```text
src/t060_fault_tests.rs
```

and only inside:

```rust
#[test]
fn input_and_resize_racing_with_exit_never_reopen_final_session()
```

The repair is limited to making the existing `execution.terminate()` result branch distinguish exactly the three production lifecycle truths below. Test setup, command/input/resize behavior, helper functions, other T060 tests, production code, and workflow configuration remain unchanged.

### Truthful outcome A — child exited before terminate cleanup

If `execution.terminate()` returns `Ok(final_exit)` and durable finalization is the natural-exit path, the fixture MUST prove:

- post-final input remains rejected;
- post-final resize remains rejected;
- repeated final observation remains stable at `Some(final_exit)`;
- durable execution status is exactly `EXITED`;
- status source is exactly `WINDS_OBSERVED`;
- `ended_unix_ms` is present;
- terminal close reason is exactly `PROCESS_EXITED`;
- no `INTERRUPTED` / `TERMINATED_BY_WINDS` claim is fabricated for the already-exited child.

### Truthful outcome B — terminate controls a still-live owned child

If `execution.terminate()` returns `Ok(final_exit)` and durable finalization is the controlled-cleanup path, the fixture MUST prove:

- post-final input remains rejected;
- post-final resize remains rejected;
- repeated final observation remains stable at `Some(final_exit)`;
- durable execution status is exactly `INTERRUPTED`;
- status source is exactly `WINDS_OBSERVED`;
- `ended_unix_ms` is present;
- terminal close reason is exactly `TERMINATED_BY_WINDS`.

### Truthful outcome C — bounded cleanup remains unproven

If `execution.terminate()` returns `Err(error)`, the fixture MUST preserve the existing Amendment-005 fail-closed path:

- the error is exactly `terminal terminate could not prove owned child exit inside bounded cleanup window`;
- no terminate/close/cleanup retry is attempted;
- post-revocation input and resize remain rejected;
- `try_wait()` remains rejected because ownership is no longer proven;
- durable execution status is exactly `OWNERSHIP_LOST`;
- status source is exactly `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` remain absent;
- terminal close reason is exactly `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`.

Any other result, lifecycle status, source, timing shape, close reason, control success, or termination error MUST fail the fixture.

## Values and behavior that MUST remain unchanged

This amendment does NOT authorize changes to:

- `src/execution.rs`, `src/terminal.rs`, `src/store.rs`, or any production source file;
- the canonical 500 ms terminal cleanup window;
- kill/reap order or polling cadence;
- `TerminalDropCleanupOutcome`, `TerminalFinalization`, `ExecutionStatus`, `FactSource`, or `TerminalCloseReason` production semantics;
- ownership revocation or drop-cleanup suppression after ownership loss;
- T057 fixture behavior or its exact-stderr repair;
- any other T060 fixture;
- T063/T096/T099 behavior;
- dependencies, lockfiles, schemas, migrations, runner images, workflows, retries, performance thresholds, platform authority, release behavior, or verification authority;
- Spec 008 Plan contents, Tasks, implementation, runtime/provider/browser behavior, daemon/IPC, remote execution, learning, plugins, automatic Git mutation, PR creation, or landing.

No sleep, retry loop, timeout widening, probabilistic allowance, ignored test, platform skip, assertion suppression, rerun-until-green strategy, or lifecycle coercion is authorized.

## Governance-amendment qualification

This amendment file itself is governance-only and is not canonical merely because it exists.

Its exact final candidate must satisfy the governance-only Standard Acceptance Gate:

- exact-head repository `quality` SUCCESS;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review bound to the exact candidate;
- zero unresolved material findings and review threads;
- exact one-file scope reconciliation;
- live main/base/head/tree/ruleset/mergeability race reconciliation;
- guarded expected-head merge;
- post-merge canonical main/tree and every actually triggered applicable push check verified.

If this governance candidate itself encounters the inherited T060 natural-exit failure before repair authority exists, that failure remains material and MUST NOT be rerun away. Execution must stop for another explicit governance decision.

## Required evidence after amendment use

After this amendment is canonical, the T060 repair must start from then-current exact canonical `main` and must be qualified in a separate PR from scratch.

Required repair evidence includes:

- exact-head repository `quality` SUCCESS on Ubuntu and macOS;
- the exact repaired T060 test demonstrably executes and passes;
- the full existing test graph remains green;
- `windows-terminal` and `release-candidate` remain green if triggered or required;
- no production, timeout, dependency, schema, workflow, or performance change;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate, or a reaching review stack permitted by canonical governance;
- zero unresolved material findings/review threads;
- exact one-file / one-function scope reconciliation;
- `behind_by=0`, live main/ruleset/mergeability race reconciliation;
- guarded expected-head landing;
- post-merge canonical main/tree and every actually triggered applicable push check verified.

The original #144 failure `34401356369` remains historical material evidence and MUST NOT be relabelled as a flake or erased by later green repair evidence.

## PR #144 qualification after T060 repair

Only after the T060 repair is canonically landed and post-merge verified may PR #144 continue:

- #144 must forward-integrate repaired canonical `main` without rebase, force-push, or history rewrite;
- the inherited T060 diff must disappear from #144's effective diff because the repair is already canonical;
- #144 must reconcile exact base/head/tree/scope to live truth;
- every CI/review result from pre-integration #144 heads remains historical only;
- #144 must restart qualification from scratch on its exact new head;
- only a guarded-merged and post-merge-verified #144 may permit PR #142 to forward-integrate repaired canonical main;
- Spec 008 Tasks and implementation remain unauthorized until the Plan itself later closes canonically.

## Explicit non-authorization

This amendment does not authorize rerunning failed #144 run `34401356369`, classifying it as a flake, changing production terminal semantics, widening timeouts, weakening child-exit proof, mixing the T060 repair into #144, or advancing Spec 008 Tasks/implementation authority.

Only canonical governance qualification, guarded landing, and post-merge verification activate the narrow T060 fixture authority above.
