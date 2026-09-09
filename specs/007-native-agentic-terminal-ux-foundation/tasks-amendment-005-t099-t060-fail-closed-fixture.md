# Spec 007 Tasks Amendment 005 — T099 T060 Fail-Closed Fixture Repair

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007 T099 exact-head acceptance requirements, canonical Spec 003 T060 lifecycle/fault semantics, canonical Spec 007 Amendment 004's requirement to stop for a new explicit governance decision when a new material failure appears, and the Founder directive in the active project session to continue the authorized Winds program without fabricating or suppressing evidence.

## Purpose

T099 requires repository `quality`, all focused T099 tests, applicable platform workflows, `release-candidate`, and frozen T097 performance evidence to be green on one exact final candidate.

After canonical Amendment 004 was used, exact T099 head `c38686d2ddf749d5cbe5ed27c3918c77676c298c` produced a new material failure in `release-candidate` run `34270793781`, macOS job `102211574096`. The full-suite `Test` step failed only at:

```text
git::t060_fault_tests::input_and_resize_racing_with_exit_never_reopen_final_session
```

The failure occurred because `TerminalExecution::terminate()` could not prove owned-child exit inside the existing bounded 500 ms cleanup window. The production implementation did not claim false success: on `TerminalDropCleanupOutcome::Unproven`, it revokes session ownership, persists `OWNERSHIP_LOST`, records `OwnershipLostProcessStateUnknown`, and returns an error.

The current T060 fixture nevertheless calls `execution.terminate().unwrap()` and therefore treats the fail-closed `OWNERSHIP_LOST` outcome as a test failure even though canonical T060 requires no false-success, falsely-live, falsely-owned, or falsely-`WINDS_OBSERVED` record and explicitly accepts ownership-loss truth when cleanup cannot be proven.

This is not classified as a flake and MUST NOT be bypassed through rerun-to-green. The original failed attempt remains material historical evidence.

Independent same-head evidence also matters to diagnosis but does not erase the failure:

- repository `quality` run `34270793792` passed the full suite on macOS and Ubuntu on the same exact T099 head;
- `windows-terminal` run `34270793800` passed native Windows/ConPTY, Ubuntu, macOS, and real Windows Server + Ubuntu WSL2 including T062 and T096 on the same exact head;
- `t097-performance` run `34270793780` passed the frozen FR-045..FR-053 qualification on the same exact head;
- the `release-candidate` T063 macOS 100-cycle terminal lifecycle soak passed on the same exact head;
- all nine focused T099 adversarial tests executed and passed in the failed macOS full-suite job before the T060 failure.

The narrow defect is therefore the fixture's success-only assertion, not evidence that the canonical 500 ms production cleanup budget should be widened or that cleanup proof should be weakened.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- T098 is `CLOSED_CANONICAL` on canonical `main` or a forward descendant;
- Spec 007 Amendment 004 is `CLOSED_CANONICAL` and its exact bounded WSL test authority has been used by the active T099 candidate;
- T099 remains the currently authorized Spec 007 task and is not canonically closed;
- exact T099 head `c38686d2ddf749d5cbe5ed27c3918c77676c298c` or its governance-only forward descendant retains the campaign-proven false-LIVE repair and Amendment 004's two test-only `8s -> 15s` host-budget changes;
- the original `release-candidate` macOS failure described above remains preserved and attributable to the success-only T060 fixture assertion after product code has already failed closed to `OWNERSHIP_LOST`;
- no later canonical repair has already removed the need for this amendment.

At amendment creation, canonical `main` is `a07ec0f23a4831c65bb250a0c8eb0ecd9ef9da62`, the guarded Amendment 004 landing. The active T099 PR is #121 at head `c38686d2ddf749d5cbe5ed27c3918c77676c298c`.

T100 remains dependency-blocked until T099 closes canonically.

## Exact additional T099 authority

When this amendment is canonical and the mandatory gate above remains true, T099 may additionally modify only:

```text
src/t060_fault_tests.rs
```

and only inside:

```rust
#[test]
fn input_and_resize_racing_with_exit_never_reopen_final_session()
```

The authorized repair is limited to replacing the fixture's unconditional successful-termination assumption with explicit verification of the two truthful outcomes already permitted by canonical T060 production semantics:

1. **Proven termination path**
   - `execution.terminate()` returns `Ok(final_exit)`;
   - post-final input and resize remain rejected;
   - `try_wait()` remains stable at `Some(final_exit)`;
   - the durable execution record remains `INTERRUPTED` with `WINDS_OBSERVED` source;
   - the terminal close reason remains `TERMINATED_BY_WINDS`.

2. **Unproven bounded-cleanup path**
   - `execution.terminate()` returns only the existing exact fail-closed cleanup-window error;
   - the fixture MUST NOT retry terminate/close or attempt to obtain a green result;
   - post-revocation input, resize, and `try_wait()` must all remain rejected because ownership is no longer proven;
   - the durable execution record must be `OWNERSHIP_LOST` with truthful `WINDS_OBSERVED` ownership-loss observation;
   - `ended_unix_ms` and `duration_ms` must remain absent because child exit was not proven;
   - the terminal close reason must be `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`.

Any other termination error, persistence result, lifecycle state, close reason, or post-revocation control success MUST fail the fixture.

No other T060 fixture or production behavior is authorized by this amendment unless a new material failure proves this exact fixture repair insufficient, in which case execution MUST stop for another explicit governance decision rather than silently widening scope.

## Values and behavior that MUST remain unchanged

This amendment does **not** authorize changing any of the following:

- the canonical 500 ms bounded terminal cleanup window in `TerminalSession::terminate`, `TerminalSession::close`, `TerminalExecution::controlled_cleanup`, or drop cleanup;
- terminal kill/reap semantics;
- `TerminalDropCleanupOutcome` semantics;
- ownership revocation or `OWNERSHIP_LOST` persistence semantics;
- `WINDS_OBSERVED` fact-source rules;
- T060 production implementation or any other T060 fixture;
- T063 soak behavior;
- Amendment 004's WSL test-only values or production WSL limits;
- workflows, runner images, retries, dependencies, lockfiles, persistence schema, migrations, performance thresholds, or release behavior;
- any Git, verification, evidence, acceptance, or landing authority;
- any Spec 003 or Spec 007 FR/SC requirement.

The fixture repair MUST NOT convert unproven cleanup into success. It must make the existing fail-closed `OWNERSHIP_LOST` truth an explicit tested outcome.

## Required evidence after use

If this amendment is used, candidate movement invalidates all prior T099 candidate-bound CI and review evidence. The repaired T099 candidate must be requalified from scratch under the Standard Acceptance Gate, including:

- repository `quality` SUCCESS on the exact repaired head;
- all focused T099 tests registered, executed, and green;
- the repaired T060 fixture executed and green while preserving strict assertions for both truthful branches;
- `windows-terminal` SUCCESS on the same exact head, including native Windows/ConPTY and the real Windows Server + Ubuntu WSL2 T062 production proof;
- Linux and macOS terminal integration SUCCESS;
- `release-candidate` SUCCESS on the same exact head;
- frozen T097 performance evidence SUCCESS where the current repository workflow applies it;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review bound to the exact repaired candidate;
- zero unresolved material findings and review threads;
- exact changed-path reconciliation proving that `src/t060_fault_tests.rs` changed only inside the fixture authority granted here;
- `behind_by=0` and live main/ruleset/mergeability revalidation immediately before landing;
- guarded expected-head landing and applicable post-merge verification before T100 is authorized.

The original failed `release-candidate` attempt on `c38686d2ddf749d5cbe5ed27c3918c77676c298c` MUST remain recorded as the evidence that exposed the over-strong fixture assumption. A later green candidate MUST NOT retroactively classify that attempt as a flake.

## Explicit non-authorization

This amendment does not authorize:

- increasing the production terminal cleanup timeout;
- weakening, skipping, or retrying cleanup proof;
- rerun-until-green classification of the failed macOS attempt;
- suppressing `OWNERSHIP_LOST` or converting it to `INTERRUPTED`/`EXITED` without proof;
- retrying control operations after ownership revocation;
- workflow or dependency changes;
- schema or persistence changes;
- daemon/service/socket/RPC/IPC behavior;
- provider/model/browser/network runtime behavior;
- remote execution;
- automatic Git mutation, winner selection, merge, or landing;
- T100 implementation or closeout before T099 is canonically landed and post-merge verified.

## Acceptance of this amendment

This file is governance-only and is not canonical merely because it exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review bound to the exact candidate, zero unresolved material findings/threads, exact one-file scope reconciliation, guarded `expected_head_sha` landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing may the additional T099 test-only authority above be used.