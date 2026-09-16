# Spec 010 Tasks Amendment 004 — T143 Qualification-Discovered T090 Bounded-Cleanup Fixture Reconciliation

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation process, canonical Spec 010 T143 qualification authority, canonical Spec 007 terminal ownership/fail-closed semantics, canonical Spec 007 Amendments 005/006/007 as governance precedents, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

T143 post-amendment qualification exposed a closed-task regression fixture whose success-only assertion is narrower than Winds' already-canonical terminal ownership truth on macOS. This amendment authorizes only the minimum test-only reconciliation needed to make four T090 cleanup fixtures accept the two existing truthful outcomes without changing production terminal behavior.

It does **not** itself repair T090, close T143, or authorize T144.

## Material evidence

Exact T143 predecessor head `f59b3f24b4e7593f2e9b8ecca8d8d433f00c44ea` ran repository `quality` as run `35106066483`. Ubuntu succeeded. macOS job `104827473547` failed with exactly one Rust test failure after `652 passed; 1 failed; 4 ignored`:

```text
workbench::t090_workbench_terminal_tests::t090_output_reader_error_while_child_is_live_fails_closed_but_retains_owned_cleanup

called `Result::unwrap()` on an `Err` value:
"terminal close could not prove owned child exit inside bounded cleanup window"
```

The same run's native T096 platform fixture passed, confirming that the already-amended T096 assertion correctly accepts bounded-unproven native cleanup. No product source changed in the T143 predecessor that could alter T090 cleanup semantics.

On the authorized macOS development host, the exact predecessor's reader-error fixture reproduced the same bounded-cleanup result in 30/30 isolated executions. Those runs each failed only because the fixture called `unwrap()` on the existing `terminal close could not prove owned child exit inside bounded cleanup window` result.

Separate local reproduction isolated the same stale success-only assumption in neighboring T090 cleanup fixtures without changing source:

```text
t090_start_and_close_bind_a_pane_to_the_exact_owned_terminal_session: 4 failures / 5 isolated runs
t090_output_reader_eof_while_child_is_live_fails_closed_but_retains_owned_cleanup: 5 failures / 5 isolated runs
t090_output_reader_error_while_child_is_live_fails_closed_but_retains_owned_cleanup: 5 failures / 5 isolated runs
```

A subsequent forced-rebuild focused T090 suite on the unchanged 500 ms production path produced `9 passed; 3 failed`; the three failures were `t090_start_and_close_bind_a_pane_to_the_exact_owned_terminal_session`, `t090_output_reader_eof_while_child_is_live_fails_closed_but_retains_owned_cleanup`, and `t090_terminal_aware_pane_close_resolves_reader_failure_before_visual_removal`. The first two returned the exact `terminal close could not prove owned child exit inside bounded cleanup window` text. The terminal-aware pane close returned the corresponding existing `terminal pane close could not prove owned child exit inside bounded cleanup window` text.

Only the reader-error fixture has first-attempt hosted exact-candidate evidence; the other three are local reproduction evidence. This amendment treats that distinction explicitly rather than promoting local observations into hosted qualification evidence. Their inclusion is justified only because each exercises the same already-canonical bounded cleanup truth and each reproduced the exact bounded-cleanup failure signature while production remained fail-closed.

The observed production result remained authority-safe: terminal ownership was revoked, the cleanup call returned the existing bounded-cleanup error, and the pane projected `OwnershipLost` where the visual pane remained present. Diagnostic timing showed the platform can remain in exiting state beyond the canonical 500 ms proof window. That does not justify widening the window or converting an unproven exit into a proven exit.

A later rejected T143 head `ae918bb7af6743949a6f8107fdfdf7ad64e00e1b` demonstrated that a broader test-only reconciliation could make 20/20 focused T090 cycles and the complete local Rust suite pass without changing production code. Its exact-head author governance review correctly rejected that head because Spec 007 Amendment 007 authorized only the T096 fixture. No acceptance claim is taken from that rejected head; it is evidence that explicit authority is required before any T090 repair.

## Canonical production truth that remains unchanged

`WorkbenchTerminals::finish_owned_terminal(...)` already distinguishes:

1. proven cleanup -> ownership removed, pane lifecycle `Exited`, success returned;
2. bounded-unproven cleanup -> ownership removed, pane lifecycle `OwnershipLost`, existing bounded-cleanup error returned;
3. cleanup error -> ownership removed, pane lifecycle `OwnershipLost`, explicit cleanup-failure error returned.

The canonical production cleanup window remains exactly `500 ms`.

This amendment does not reinterpret `OwnershipLost` as success. It only permits a test fixture whose purpose is lifecycle safety to assert the existing fail-closed truth instead of unconditionally requiring proof that the child exited inside the bounded window.

## Exact additional repair authority after canonical landing

Only after this amendment is `CLOSED_CANONICAL` may a repair modify:

```text
src/t090_workbench_terminal_tests.rs
```

The repair may change only the four functions listed below and may add exactly one new private helper named `assert_t090_bounded_close_truth`:

```text
t090_start_and_close_bind_a_pane_to_the_exact_owned_terminal_session
t090_output_reader_eof_while_child_is_live_fails_closed_but_retains_owned_cleanup
t090_output_reader_error_while_child_is_live_fails_closed_but_retains_owned_cleanup
t090_terminal_aware_pane_close_resolves_reader_failure_before_visual_removal
```

`assert_t090_bounded_close_truth` may be called only by the first three functions above. The repair MUST prove by exact source search that no other test calls it. The helper may inspect only the close result, retained terminal ownership, and pane lifecycle needed for the assertions below. No existing shared helper, fixture constructor, profile builder, reader type, function signature, or shared helper body may change. The terminal-aware `close_pane(...)` function must keep its result handling inline and MUST NOT reuse the helper.

For the first three functions, the cleanup assertion may distinguish only these existing outcomes:

### Outcome A — proven cleanup

If `terminals.close(...)` returns `Ok(_)`, the fixture MUST prove:

- the terminal ownership map no longer contains the pane;
- pane lifecycle is exactly `PaneLifecycleView::Exited`.

### Outcome B — bounded cleanup is unproven

If `terminals.close(...)` returns `Err(error)`, the fixture MUST prove all of:

- the error contains the existing exact text `could not prove owned child exit inside bounded cleanup window`;
- the terminal ownership map no longer contains the pane;
- pane lifecycle is exactly `PaneLifecycleView::OwnershipLost`;
- no cleanup retry, terminate retry, sleep-to-green, or lifecycle promotion occurs.

Any other error or lifecycle MUST fail the fixture.

For `t090_terminal_aware_pane_close_resolves_reader_failure_before_visual_removal`, `close_pane(...)` may distinguish only:

- proven cleanup: ownership removed, `Some(exit)` returned, visual pane removed;
- bounded-unproven cleanup: exact bounded-cleanup error, ownership removed, visual pane retained, lifecycle exactly `OwnershipLost`.

Any other error, retained terminal ownership, removed visual pane on unproven cleanup, or lifecycle result MUST fail the fixture.

## Explicitly unchanged T090 fixtures

This amendment does **not** authorize changing cleanup assertions in any other T090 test. In particular, the following remain unchanged unless a later first-attempt exact-candidate failure separately proves a need:

```text
t090_resize_changes_presentation_size_only_after_the_owned_session_accepts_it
t090_interrupt_and_terminate_reuse_the_accepted_owned_session_lifecycle
t090_duplicate_start_never_replaces_the_existing_owned_terminal
```

Natural-exit and already-exited drain fixtures also remain unchanged.

## Values and behavior that MUST remain unchanged

This amendment does not authorize changes to:

- production Rust source;
- the 500 ms cleanup window;
- cleanup ordering, signals, kill/reap behavior, retries, or sleeps;
- `TerminalDropCleanupOutcome`;
- ownership revocation or pane lifecycle mappings;
- ignored tests, platform skips, runner images, or workflow logic;
- dependencies or lockfiles;
- schemas or migrations;
- T143 performance thresholds, sample floors, measurement scope, or evidence semantics;
- security, accessibility, runtime/provider, Git, verification, or landing authority;
- T144 human Founder visual acceptance.

## Required repair evidence after amendment landing

The authorized T090 repair must start from then-current exact canonical `main` and qualify from scratch. It must provide:

- changed source scope exactly `src/t090_workbench_terminal_tests.rs`;
- focused T090 tests PASS on the exact candidate;
- `git diff --check` PASS;
- `cargo fmt --all -- --check` PASS;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` PASS;
- repository `quality` SUCCESS on Ubuntu and macOS;
- `windows-terminal` SUCCESS across every actually-triggered platform job;
- `release-candidate` SUCCESS if triggered or required by the canonical gate;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- guarded expected-head normal merge and successful applicable post-merge verification.

A repair failure is material. It MUST NOT be rerun on the same head to obtain green evidence.

Only after that repair is canonically landed may the active T143 branch forward-integrate then-current `main`, retain only separately authorized T143 work, and requalify the complete T143 campaign from scratch. No CI, performance, or review evidence from `f59b3f2...` or rejected `ae918bb...` qualifies the moved T143 candidate.

## Amendment acceptance gate

This governance-only amendment is not canonical merely because it exists. Its exact final candidate must satisfy:

- changed scope exactly this one amendment document;
- repository `quality` is executed once on the exact amendment head;
- Ubuntu `quality` MUST succeed;
- macOS `quality` MUST succeed **except** that the amendment remains eligible when, and only when, all of the following are simultaneously true:
  - `git diff` proves the amendment head changes exactly this Markdown document and no Rust or workflow source;
  - Ubuntu `quality` succeeds;
  - macOS reports exactly one failing Rust test and zero other failures;
  - that one test is one of the four named T090 functions authorized above;
  - the failure text contains the existing bounded-cleanup signature `could not prove owned child exit inside bounded cleanup window`;
  - no same-head rerun replaces or obscures the first-attempt result;
  - author correctness/safety/governance/evidence-integrity review and fresh independent exact-head review both explicitly classify the failure as the unchanged pre-existing subject blocker and otherwise report zero material findings;
- if any condition above is false, the amendment is blocked and execution must stop for another explicit governance decision;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- exact current main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded normal merge with exact expected head;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered workflow on the amendment's own merge commit is reconciled before the amendment is considered canonical, including `windows-terminal` or `release-candidate` when they are triggered; the same inherited-failure exception may apply only to those workflows on that exact amendment merge, only when the sole failure is one of the four named T090 tests with the same bounded-cleanup signature, and only with zero unrelated failures.

This narrow acceptance exception is part of the governance decision because a documentation-only amendment cannot itself repair the inherited test that blocks authority for the repair. It does not generalize to any other test, workflow, platform, task, candidate, or future failure, and it expires immediately when the authorized T090 repair lands. Reverting this amendment before any repair lands fully removes the added repair authority and restores the prior governance state, so the deviation is reversible until used.

Only after the amendment lands and its applicable post-merge evidence is reconciled may repository truth state:

```text
SPEC_010_TASKS_AMENDMENT_004=CLOSED_CANONICAL
T090_BOUNDED_CLEANUP_FIXTURE_REPAIR=AUTHORIZED_FOUR_FUNCTIONS_ONLY
T090_PRODUCTION_CLEANUP_WINDOW_MS=500_UNCHANGED
T143=BLOCKED_UNTIL_T090_REPAIR_CLOSED_CANONICAL
T144=BLOCKED_UNTIL_T143_CLOSED_CANONICAL
```
