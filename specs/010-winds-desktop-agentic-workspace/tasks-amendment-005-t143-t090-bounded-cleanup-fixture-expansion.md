# Spec 010 Tasks Amendment 005 — T143 T090 Bounded-Cleanup Fixture Authority Expansion

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation process, canonical Spec 010 T143 qualification authority, canonical Spec 010 Amendment 004 (`4afa6dca5682a01cc56efd01167f14164d90cd8a`), canonical Spec 007 bounded-cleanup truth, and the active Founder directive to continue without fabricating, suppressing, or rerunning away material evidence.

## Purpose

Canonical Amendment 004 authorized test-only bounded-cleanup truth reconciliation for four named T090 fixtures and explicitly left three neighboring cleanup fixtures unchanged unless later material evidence proved a need.

The first local qualification of the authorized repair candidate proved that all three excluded fixtures can independently reach the same already-canonical bounded-unproven cleanup outcome on macOS while production remains fail-closed. Subsequent hosted exact-head CI happened to pass those same paths. This timing-sensitive green result does not erase the earlier failure, and rerunning local qualification on the same candidate to obtain green is prohibited.

This amendment therefore expands only the existing test-only repair authority to the three newly proven fixtures. It does not change production terminal behavior, the 500 ms cleanup window, retry policy, process ownership semantics, T143 performance thresholds, or T144 human acceptance.

## Material evidence

Canonical Amendment 004 landed as merge `4afa6dca5682a01cc56efd01167f14164d90cd8a`, with post-merge `quality` run `35113347555` successful on Ubuntu and macOS.

The first repair source candidate then modified only the four Amendment-004-authorized fixtures plus the authorized private helper. Before commit, its first focused T090 qualification failed because two explicitly excluded fixtures independently reached bounded-unproven cleanup:

```text
t090_interrupt_and_terminate_reuse_the_accepted_owned_session_lifecycle
  error: terminal terminate could not prove owned child exit inside bounded cleanup window

t090_duplicate_start_never_replaces_the_existing_owned_terminal
  error: terminal close could not prove owned child exit inside bounded cleanup window
```

The complete first Rust qualification on the same source then produced:

```text
650 passed; 3 failed; 4 ignored
```

with exactly these three failures:

```text
t090_resize_changes_presentation_size_only_after_the_owned_session_accepts_it
  error: terminal close could not prove owned child exit inside bounded cleanup window

t090_interrupt_and_terminate_reuse_the_accepted_owned_session_lifecycle
  error: terminal terminate could not prove owned child exit inside bounded cleanup window

t090_duplicate_start_never_replaces_the_existing_owned_terminal
  error: terminal close could not prove owned child exit inside bounded cleanup window
```

No production Rust source, timeout, cleanup order, retry, workflow, dependency, or lockfile differed from canonical main.

That narrow repair source was then committed unchanged as exact candidate `b01cee2a51dd81a1fa39cc6ae3be559242c3041a` / tree `0746d0d3776037570f5f963d862e302f6f8b3df0` on PR #215 solely to preserve exact-candidate evidence. Its first-attempt hosted `quality` run `35114423539` succeeded on Ubuntu and macOS, and first-attempt `release-candidate` run `35114423369` also succeeded. `windows-terminal` macOS integration succeeded on the same head.

Those hosted successes are not evidence that the local failures were false or that another local run should replace them. They prove the cleanup result is timing-sensitive across executions and reinforce the need for fixtures to assert the two already-existing truthful outcomes rather than require one timing-dependent result.

## Why the existing Amendment 004 scope is insufficient

Amendment 004 intentionally did not authorize changes to these three tests because, at amendment time, the material evidence was strongest for four neighboring fixtures. It required new evidence before scope expansion.

That evidence now exists. Leaving the three tests success-only would preserve known timing-sensitive false negatives in the repository gate. Re-running until they happen to pass would violate exact-evidence discipline. Changing production timing or widening the 500 ms proof window would weaken the canonical fail-closed model merely to satisfy stale test assumptions.

The smallest reversible path is to expand the same test-only truth reconciliation to these three fixtures and nothing else.

## Exact additional repair authority after canonical landing

Only after this amendment is `CLOSED_CANONICAL` may the active T090 repair candidate additionally modify these three functions in `src/t090_workbench_terminal_tests.rs`:

```text
t090_resize_changes_presentation_size_only_after_the_owned_session_accepts_it
t090_interrupt_and_terminate_reuse_the_accepted_owned_session_lifecycle
t090_duplicate_start_never_replaces_the_existing_owned_terminal
```

The existing Amendment-004 helper `assert_t090_bounded_close_truth` may then be called by these three additional functions. After expansion, exact source search MUST prove the helper has exactly six call sites, all inside the six `close`/`terminate` fixtures authorized by Amendments 004 and 005. `t090_terminal_aware_pane_close_resolves_reader_failure_before_visual_removal` remains inline as required by Amendment 004.

No helper body, shared fixture constructor, profile builder, reader type, function signature, or production function may change beyond the helper body already authorized by Amendment 004.

### Resize fixture

All resize assertions before cleanup MUST remain unchanged. Final `close(...)` may accept only:

- `Ok(_)` + ownership removed + lifecycle `Exited`; or
- exact bounded-cleanup error + ownership removed + lifecycle `OwnershipLost`.

### Interrupt/terminate fixture

The interrupt assertion and proof that it does not itself lose ownership MUST remain unchanged. Final `terminate(...)` may accept only:

- `Ok(_)` + ownership removed + lifecycle `Exited`; or
- exact bounded-cleanup error + ownership removed + lifecycle `OwnershipLost`.

No retry of terminate/close/cleanup is authorized.

### Duplicate-start fixture

The duplicate-start rejection, retained original ownership, and `Live` lifecycle assertion before final cleanup MUST remain unchanged. Final `close(...)` may accept only the same two exact bounded cleanup outcomes above.

Any unrelated error, retained ownership after cleanup returns, lifecycle other than `Exited`/`OwnershipLost` as specified, retry, sleep-to-green, or lifecycle promotion MUST fail the fixture.

## Explicit non-authorization

This amendment does not authorize changes to:

- any T090 fixture other than the seven total functions jointly authorized by Amendments 004 and 005;
- any production Rust source;
- the 500 ms cleanup window;
- cleanup order, signals, kill/reap behavior, polling cadence, sleeps, retries, or drop semantics;
- `TerminalDropCleanupOutcome`, ownership revocation, or pane lifecycle mappings;
- ignored tests, platform skips, runner images, workflow logic, dependencies, or lockfiles;
- schemas or migrations;
- T143 performance thresholds, sample floors, measurement semantics, or evidence rules;
- security, accessibility, runtime/provider, Git, verification, or landing authority;
- T144 human Founder visual acceptance.

## Required repair evidence after amendment landing

After this amendment lands canonically, PR #215 must move forward-only from exact canonical `main` and requalify from scratch. Required evidence includes:

- repair scope remains exactly `src/t090_workbench_terminal_tests.rs`;
- helper call sites are exactly six and all are within the six Amendment-004/005 authorized `close`/`terminate` fixtures;
- the terminal-aware `close_pane` fixture remains inline;
- focused T090 tests PASS on the final exact candidate;
- `git diff --check` PASS;
- `cargo fmt --all -- --check` PASS;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` PASS;
- complete Rust tests PASS;
- repository `quality` SUCCESS on Ubuntu and macOS;
- `windows-terminal`, `release-candidate`, and every other actually-triggered applicable workflow SUCCESS on the exact final head;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- exact current main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered post-merge workflow SUCCESS.

The failed local qualification evidence above remains material historical evidence after repair and MUST NOT be replaced by a same-head rerun claim.

## Amendment acceptance gate

This governance-only amendment is not canonical merely because it exists. Its exact final candidate must satisfy:

- changed scope exactly this one amendment document;
- repository `quality` SUCCESS on Ubuntu and macOS;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- exact current main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded expected-head normal merge;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered applicable post-merge workflow SUCCESS.

Only after those gates may repository truth state:

```text
SPEC_010_TASKS_AMENDMENT_005=CLOSED_CANONICAL
T090_BOUNDED_CLEANUP_FIXTURE_REPAIR=AUTHORIZED_SEVEN_FUNCTIONS_ONLY
T090_PRODUCTION_CLEANUP_WINDOW_MS=500_UNCHANGED
T143=BLOCKED_UNTIL_T090_REPAIR_CLOSED_CANONICAL
T144=BLOCKED_UNTIL_T143_CLOSED_CANONICAL
```
