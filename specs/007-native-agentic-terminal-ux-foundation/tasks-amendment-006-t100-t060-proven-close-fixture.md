# Spec 007 Tasks Amendment 006 — T100 Post-Merge T060 Proven-Close Fixture Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007 T100 post-merge acceptance requirements, canonical Spec 003 T053/T060 lifecycle truth, canonical Spec 007 Amendment 005's requirement to stop for a new explicit governance decision if a new material failure proves its bounded fixture repair insufficient, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

T100 was guarded-merged from exact candidate `ebb908c72c6da09fce7dbab5540ab3632c676516` / tree `522ea718cce2f3472a8eacd0c67342b25d91ac25` as merge commit `6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7`, whose tree is exactly the accepted candidate tree and whose ordered parents are the former canonical main `e3e5d98c36bb4289e8e4aebcc2dbb07c97abaa98` and exact T100 candidate `ebb908c72c6da09fce7dbab5540ab3632c676516`.

T100 is not `CLOSED_CANONICAL` because its required post-merge push `quality` run `34284403842` failed on exact canonical main `6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7`.

The original push attempt is material evidence and MUST remain preserved. It MUST NOT be reclassified as a flake or bypassed through rerun-to-green.

Exact post-merge result:

```text
RUN=34284403842
EVENT=push
HEAD=6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7
TREE=522ea718cce2f3472a8eacd0c67342b25d91ac25
CONCLUSION=FAILURE
UBUNTU_JOB=102256499747 FAILURE
MACOS_JOB=102256499893 SUCCESS
```

The Ubuntu job checked out the exact merge SHA, passed Format and Clippy, then failed the full `cargo test --locked --all-targets --all-features` suite with exactly one failing unit test while 432 tests passed and 6 remained ignored:

```text
git::t060_fault_tests::interrupt_then_close_escalates_only_while_session_is_still_owned
```

The failing assertion observed:

```text
left: Exited
right: Interrupted
```

The neighboring Amendment-005-repaired T060 fixture `input_and_resize_racing_with_exit_never_reopen_final_session` passed in the same failed Ubuntu job. The macOS full `quality` job passed on the same exact canonical main/tree.

## Diagnosis

The failure exposes an over-strong fixture assumption rather than evidence that production cleanup semantics should be widened.

Canonical `TerminalExecution::controlled_cleanup(...)` already distinguishes three truthful outcomes:

1. `TerminalDropCleanupOutcome::ExitedBeforeCleanup(exit)` -> durable `EXITED` / `WINDS_OBSERVED` / `PROCESS_EXITED` truth;
2. `TerminalDropCleanupOutcome::Terminated(exit)` -> durable `INTERRUPTED` / `WINDS_OBSERVED` with the explicit controlled close reason;
3. `TerminalDropCleanupOutcome::Unproven` -> revoke ownership, durable `OWNERSHIP_LOST` / `WINDS_OBSERVED`, no fabricated end/duration, and an explicit bounded-cleanup error.

The failing fixture already proves the shell is owned and live immediately before close with:

```rust
assert_eq!(execution.try_wait().unwrap(), None);
```

That observation does not prove the child cannot exit in the interval before `close()` obtains its own cleanup observation. If the child exits in that interval, production must preserve `ExitedBeforeCleanup` as `EXITED`; converting that observed natural exit into `INTERRUPTED` would be less truthful.

Canonical Spec 003 T060 acceptance describes this exact trapped-child interrupt -> bounded close fixture as accepting only truthful **proven-close versus ownership-lost** outcomes. It does not require every successful `close()` to be recorded as `INTERRUPTED`. Canonical T053 likewise distinguishes natural exit-before-cleanup, proven termination, and unproven cleanup.

Therefore the narrow defect is the fixture branch `if close_proven { assert_eq!(record.status, ExecutionStatus::Interrupted); ... }`, which collapses both proven-close subtypes into the controlled-termination subtype.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` contains T099 and the guarded T100 merge commit `6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7` or a governance-only forward descendant;
- T100 remains not `CLOSED_CANONICAL` because post-merge `quality` run `34284403842` failed as recorded above;
- the original failed run remains preserved and has not been replaced by a rerun-based completion claim;
- canonical Spec 003 T053/T060 production cleanup semantics remain unchanged;
- Amendment 005 remains canonical and its repair is limited to the separate `input_and_resize_racing_with_exit_never_reopen_final_session` fixture;
- no later canonical repair has already removed the need for this amendment.

At amendment creation, canonical `main` is `6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7`.

No Spec 008 or other successor implementation is authorized while T100 post-merge closure remains unresolved.

## Exact additional repair authority

Only after this amendment is canonically landed may the T100 post-merge repair modify:

```text
src/t060_fault_tests.rs
```

and only inside:

```rust
#[test]
fn interrupt_then_close_escalates_only_while_session_is_still_owned()
```

The authorized repair is limited to preserving the existing setup, interrupt proof, shell-resume marker, bounded-close measurement, exact close-error check, and ownership-loss assertions while replacing the proven-close `INTERRUPTED`-only assumption with strict verification of the two already-existing proven-close subtypes plus the existing fail-closed subtype.

### Truthful outcome A — child exited before close cleanup

If `execution.close()` returns `Ok(final_exit)` and durable finalization is the existing natural-exit path, the fixture MUST prove:

- durable execution status is `EXITED`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` are present;
- terminal close reason is `PROCESS_EXITED`;
- repeated final observation, if performed before ownership is released by dropping the wrapper, remains consistent with the returned `final_exit`;
- no `CLOSED_BY_WINDS` or `INTERRUPTED` claim is fabricated for a child already observed exited before cleanup.

### Truthful outcome B — close terminates the still-live owned child

If `execution.close()` returns `Ok(final_exit)` and durable finalization is the existing controlled-cleanup path, the fixture MUST prove:

- durable execution status is `INTERRUPTED`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` are present;
- terminal close reason is `CLOSED_BY_WINDS`;
- repeated final observation, if performed before ownership is released by dropping the wrapper, remains consistent with the returned `final_exit`.

### Truthful outcome C — bounded cleanup cannot prove exit

If `execution.close()` returns `Err(error)`, the fixture MUST preserve the existing fail-closed assertions:

- the error must contain the existing bounded cleanup-window failure text;
- the fixture MUST NOT retry `close()`, `terminate()`, or any other cleanup solely to obtain a green result;
- durable execution status is `OWNERSHIP_LOST`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` remain absent;
- terminal close reason is `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`;
- the existing `TerminalOwnershipLostAfterCleanupFailure` event remains present with `WINDS_OBSERVED` source.

Any other close error, lifecycle status, fact source, end/duration shape, close reason, or ownership-loss event result MUST fail the fixture.

## Values and behavior that MUST remain unchanged

This amendment does **not** authorize changing any of the following:

- production `TerminalExecution`, `TerminalSession`, `Store`, lifecycle, or persistence code;
- the canonical 500 ms terminal cleanup window;
- kill/reap behavior or cleanup ordering;
- `TerminalDropCleanupOutcome` variants or their meanings;
- `TerminalFinalization` variants or their meanings;
- `ExecutionStatus`, `FactSource`, or `TerminalCloseReason` production mappings;
- interrupt signaling semantics;
- ownership revocation or `OWNERSHIP_LOST` behavior;
- any other T060 fixture, including the Amendment-005-repaired fixture;
- workflows, runner images, retry policy, dependencies, lockfiles, schema, migrations, performance thresholds, platform authority, release behavior, or verification authority;
- any Spec 003 or Spec 007 FR/SC requirement.

No sleep, retry loop, timeout widening, probabilistic allowance, ignored-test marker, platform skip, assertion deletion, or status coercion is authorized.

## Required evidence after amendment landing

The governance amendment itself must pass the repository governance Standard Acceptance Gate before use.

After the amendment is canonical, the repair must start from the then-current exact `main` and must be requalified from scratch on its exact final head. Required repair evidence includes:

- repository `quality` SUCCESS on Ubuntu and macOS;
- the exact repaired T060 test demonstrably executes and passes;
- the full existing test graph remains green;
- applicable terminal/platform workflows triggered by the repair path remain green, including `windows-terminal` if path filters trigger it;
- `release-candidate` remains green if the repair candidate triggers or the canonical gate requires it;
- no performance threshold or performance implementation change;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate, or a reaching review stack permitted by the canonical Standard Acceptance Gate;
- zero unresolved material findings and review threads;
- exact changed-path reconciliation proving only the authorized T060 function changed;
- `behind_by=0`, ruleset/mergeability/main-race revalidation;
- guarded expected-head landing;
- successful applicable post-merge push `quality` on the repaired canonical main.

The failed T100 post-merge run `34284403842` MUST remain recorded as the evidence that exposed the over-strong proven-close assertion. A later green repair MUST NOT retroactively classify that run as a flake or erase it.

## T100 closure after repair

Landing this amendment does not close T100. Landing the fixture repair does not by itself close T100.

Only after the repaired canonical main passes all applicable post-merge checks may repository truth perform a final T100 closure reconciliation. That reconciliation must preserve:

- guarded T100 merge `6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7`;
- failed original post-merge `quality` run `34284403842` as material evidence;
- canonical Amendment 006 landing;
- exact fixture-repair candidate and merge evidence;
- successful post-repair canonical push evidence;
- no successor implementation authority beyond what then-canonical governance explicitly grants.

A later final closure record may update or add only Spec 007 documentation/evidence surfaces necessary to state the complete post-merge repair chain truthfully. It may not alter production behavior or silently authorize Spec 008.

## Explicit non-authorization

This amendment does not authorize:

- rerunning failed run `34284403842` as a substitute for repair;
- classifying `EXITED` as a flake when it was actually observed;
- forcing natural `EXITED` truth to `INTERRUPTED`;
- changing production cleanup or persistence semantics;
- weakening ownership proof;
- retrying cleanup after ownership loss;
- suppressing or ignoring the failing test;
- workflow/dependency/schema/performance changes;
- daemon/service/socket/RPC/IPC behavior;
- provider/model/browser/network runtime behavior;
- remote execution;
- automatic Git mutation, winner selection, merge, or landing;
- Spec 008 or any later roadmap implementation.

## Acceptance of this amendment

This file is governance-only and is not canonical merely because it exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review or permitted reaching review stack, zero unresolved material findings/threads, exact one-file scope reconciliation, live main/ruleset/mergeability race reconciliation, guarded `expected_head_sha` landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing may the exact test-only repair authority above be used.