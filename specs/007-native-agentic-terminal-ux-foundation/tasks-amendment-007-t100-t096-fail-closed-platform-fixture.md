# Spec 007 Tasks Amendment 007 — T100 Post-Repair T096 Fail-Closed Platform Fixture

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007 T096 platform qualification truth, canonical T100 post-merge acceptance requirements, canonical Spec 007 Amendment 006, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

The Amendment-006-authorized T060 repair landed canonically as merge commit `6fb8782bea3831e7b8f94902082783631bcaf8ad` from exact reviewed head `70de319a55e022ca233d6ce8c2555d0f296854d0` / tree `f9e113cdfb24d3aa3fd68f2fb8a13b720639b720` against canonical base `6c69a64560e37e0411bf5403b36e21a60e285cf3`.

Post-merge `quality` run `34289087688` succeeded on Ubuntu and macOS. The separately triggered post-merge `windows-terminal` run `34289087658` failed on the same canonical merge and therefore remains material first-attempt evidence. It MUST NOT be rerun or reclassified as a flake to obtain closure.

Exact post-merge result:

```text
RUN=34289087658
EVENT=push
HEAD=6fb8782bea3831e7b8f94902082783631bcaf8ad
TREE=f9e113cdfb24d3aa3fd68f2fb8a13b720639b720
CONCLUSION=FAILURE
UBUNTU_TERMINAL=SUCCESS
MACOS_TERMINAL=FAILURE
NATIVE_WINDOWS_TERMINAL=SUCCESS
REAL_WINDOWS_WSL2=SUCCESS
FAILED_JOB=102271344402
FAILED_STEP=T096 native Unix workbench qualification
```

The macOS job first passed the six production PTY lifecycle tests and the T057 CLI terminal proof integration. It then failed only:

```text
workbench::t096_workbench_platform_tests::t096_native_workbench_path_directly_qualifies_current_host_domain
```

with the exact existing production error:

```text
terminal close could not prove owned child exit inside bounded cleanup window
```

The failure occurred because the T096 fixture unconditionally calls `.expect("T096 native owned terminal must close with proven cleanup")` and then requires `PaneLifecycleView::Exited`.

## Diagnosis

The observed production behavior is already fail-closed and authority-safe.

`WorkbenchTerminals::finish_owned_terminal(...)` removes the retained terminal from the ownership map before bounded cleanup. It then maps:

1. `ExitedBeforeCleanup` or `Terminated` -> `PaneLifecycleView::Exited` and `Ok(exit)`;
2. `Unproven` -> suppress further drop cleanup, set `PaneLifecycleView::OwnershipLost`, and return the existing bounded-cleanup error;
3. cleanup error -> suppress further drop cleanup, set `PaneLifecycleView::OwnershipLost`, and return an explicit cleanup-failure error.

Canonical T096 requires close/terminate behavior to be directly qualified per platform and forbids unsupported or unproven behavior from being silently inferred. It does not authorize converting an unproven cleanup result into a proven exit.

The failing fixture therefore contains an over-strong expectation: it assumes the native platform proof must always obtain proven cleanup within the 500 ms production window. On macOS the canonical post-merge run demonstrated that the truthful outcome may instead be bounded cleanup with ownership revoked and `OWNERSHIP_LOST` presentation truth.

No evidence justifies widening the production cleanup timeout, retrying cleanup, or changing ownership semantics.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` is `6fb8782bea3831e7b8f94902082783631bcaf8ad` or a governance-only forward descendant;
- post-merge `quality` run `34289087688` remains SUCCESS on that repair landing;
- post-merge `windows-terminal` run `34289087658` remains preserved as FAILURE and is not replaced by rerun-to-green evidence;
- its only failed platform job is macOS job `102271344402` at the T096 native Unix workbench qualification step;
- Ubuntu terminal, native Windows, and real Windows+Ubuntu WSL2 jobs remain SUCCESS on that same run;
- production 500 ms cleanup, ownership revocation, and workbench lifecycle mappings remain unchanged;
- T100 remains not `CLOSED_CANONICAL`;
- no later canonical repair has already removed the need for this amendment.

No Spec 008 or later-roadmap implementation is authorized while T100 remains open.

## Exact additional repair authority

Only after this amendment is canonically landed may a repair modify:

```text
src/t096_workbench_platform_tests.rs
```

and only inside:

```rust
#[test]
fn t096_native_workbench_path_directly_qualifies_current_host_domain()
```

The setup, host-TUI render, native profile selection, terminal launch, resize, shell dispatch, output observation, Unicode parser proof, and fail-closed OSC52 proof MUST remain unchanged.

The final `terminals.close(&mut state, pane)` assertion may be changed only to distinguish the following existing truthful outcomes.

### Truthful outcome A — cleanup is proven

If `close()` returns `Ok(_)`, the fixture MUST prove:

- `terminals.has_owned_terminal(pane)` is false;
- pane lifecycle is exactly `PaneLifecycleView::Exited`.

### Truthful outcome B — bounded cleanup is unproven

If `close()` returns `Err(error)` containing the existing exact bounded-cleanup text, the fixture MUST prove:

- the error contains `terminal close could not prove owned child exit inside bounded cleanup window`;
- `terminals.has_owned_terminal(pane)` is false;
- pane lifecycle is exactly `PaneLifecycleView::OwnershipLost`;
- no retry of `close()`, `terminate()`, or cleanup is performed to manufacture success.

Any other error or lifecycle result MUST fail the fixture.

## Values and behavior that MUST remain unchanged

This amendment does not authorize changes to production source, the 500 ms cleanup window, cleanup ordering, kill/reap behavior, `TerminalDropCleanupOutcome`, ownership revocation, pane lifecycle mappings, retries, sleeps, ignored tests, platform skips, workflows, runner images, dependencies, lockfiles, schemas, migrations, performance thresholds, Git authority, runtime authority, or successor-specification authority.

The separate T096 WSL2 fixture remains unchanged because it passed on the exact failed post-merge run. No speculative repair is authorized for that path.

## Required evidence after amendment landing

The governance amendment itself must satisfy the repository Standard Acceptance Gate before use: exact-head `quality`, author correctness/safety/governance/evidence-integrity review, Ponytail/YAGNI review, fresh independent substantive review or a canonically permitted reaching stack, zero unresolved material findings/threads, exact one-file scope reconciliation, live main/ruleset/mergeability race reconciliation, guarded expected-head landing, and applicable post-merge push verification.

After canonical amendment landing, the test-only repair must start from then-current exact `main` and be requalified from scratch on its exact final head. Required evidence includes:

- repository `quality` SUCCESS on Ubuntu and macOS;
- the exact repaired T096 native platform fixture demonstrably executes and passes;
- `windows-terminal` SUCCESS across Ubuntu, macOS, native Windows, and real Windows+Ubuntu WSL2;
- `release-candidate` SUCCESS if triggered or required by the canonical gate;
- no performance implementation or threshold change;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate;
- zero unresolved material findings and review threads;
- exact one-file / one-function scope reconciliation;
- `behind_by=0`, ruleset/mergeability/main-race revalidation;
- guarded expected-head landing;
- successful applicable post-merge push checks, including `quality` and any actually triggered terminal/platform workflow.

Historical failed runs `34284403842` and `34289087658` remain material evidence and MUST NOT be erased or reclassified by later green evidence.

## T100 closure after repair

Landing this amendment does not close T100. Landing the T096 fixture repair does not close T100 by itself.

Only after the repaired canonical main passes all applicable post-merge checks may a separate final T100 closure reconciliation update Spec 007 documentation/evidence surfaces to record the complete chain, including the original T100 merge, both preserved post-merge failures, Amendments 006 and 007, both bounded fixture repairs, their exact candidate/review/CI/landing evidence, and successful final post-repair canonical push evidence.

That closure may not modify production behavior or authorize Spec 008.

## Explicit non-authorization

This amendment does not authorize rerun-to-green, flake classification, timeout widening, forced `EXITED` truth, forced cleanup success, retries after ownership loss, assertion suppression, ignored tests, workflow weakening, dependency/schema/performance changes, daemon/IPC, provider/model/browser/network runtime, remote execution, automatic Git mutation/selection/landing, Spec 008, or any later roadmap implementation.

## Acceptance of this amendment

This file is governance-only and is not canonical merely because it exists. Only after its exact candidate passes the Standard Acceptance Gate, lands through an expected-head guard, and is post-merge verified may the exact test-only repair authority above be used.
