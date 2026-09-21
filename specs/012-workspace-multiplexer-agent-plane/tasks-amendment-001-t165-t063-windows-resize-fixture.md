# Spec 012 Tasks Amendment 001 — T165 Post-Merge T063 Windows Resize Fixture Reconciliation

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation process, canonical Spec 012 Tasks, canonical T165 implementation authority, canonical Spec 003 T063 terminal lifecycle truth, established Winds forward-only fixture-repair precedent, and the active Founder directive to continue the authorized Winds program without fabricating, suppressing, bypassing, or rerunning away material evidence.

## Purpose

T165 post-merge qualification exposed one inherited Windows-only T063 timing assumption outside the T165 authorized source paths.

This amendment authorizes only the minimum test-only repair needed to preserve the existing T063 Windows child-observed ConPTY resize proof while giving that single child-side PowerShell observation a bounded hosted-runner response window that reflects the failure evidence.

It does **not** repair the fixture itself, close T165, authorize T166, change production terminal behavior, weaken the T063 resize assertion, or authorize any workflow rerun.

## Preserved material evidence

T165 qualified and guarded-landed from exact head:

```text
T165_QUALIFIED_HEAD=93d41a77f4ec3d41662389eeda7e76a04faa52e7
T165_QUALIFIED_TREE=bae001f38f8d6fc4b78338a99da23d83c162956b
T165_MERGE=84c10ba7493e5ce797699efca607bb7aa414cd19
T165_MERGE_TREE=bae001f38f8d6fc4b78338a99da23d83c162956b
```

The merge tree exactly matches the qualified head tree. The merge parents are ordered as the former canonical main and the qualified T165 head.

Exactly six push workflows were triggered by the T165 merge. Five completed successfully:

```text
quality #1558                run 35559526799 SUCCESS
windows-terminal #983       run 35559526790 SUCCESS
t141-desktop-security #150  run 35559526779 SUCCESS
t142-native-platform #148   run 35559526847 SUCCESS
t159-performance #18        run 35559526809 SUCCESS
```

The sixth workflow remains material failure evidence and MUST NOT be rerun into acceptance:

```text
t160-native-platform #10
run=35559526761
head=84c10ba7493e5ce797699efca607bb7aa414cd19
persistent-runtime (macos)=SUCCESS
persistent-runtime (linux)=SUCCESS
persistent-runtime (windows)=FAILURE
failed_step=Run 100-cycle terminal lifecycle resource cleanup
failed_test=git::t063_soak_tests::active_close_and_windows_child_resize_guard
job=106209589842
```

The same Windows job proved all of the following before the failure:

- direct persistent-runtime qualification passed;
- direct domain evidence passed;
- persistent-owner reconnect churn passed;
- native Windows ConPTY qualification passed;
- `git::t063_soak_tests::controlled_terminal_lifecycle_soak_100_cycles` passed all 100 lifecycle cycles in 28.88 seconds.

The supplemental resize guard then failed only while waiting for the exact child-observed resize marker:

```text
T063 cycle 100 timed out waiting for output marker "WINDS_T063_CHILD_SIZE_33 101"
```

The captured terminal output showed:

- the Windows shell remained alive;
- the exact PowerShell child command had been accepted and echoed;
- the expected marker had not arrived within the fixture's 10-second observation deadline;
- no alternate marker, malformed marker, wrong resize value, process-exit claim, or production ownership failure was observed before the timeout.

The exact same T165 source tree had already passed this same supplemental test on the first exact-head pre-merge T160 run:

```text
premerge_t160_run=35558117843
windows_job=106205631879
head=93d41a77f4ec3d41662389eeda7e76a04faa52e7
tree=bae001f38f8d6fc4b78338a99da23d83c162956b
active_close_and_windows_child_resize_guard=PASS
test_duration=2.72s
```

No source difference exists between the qualified T165 tree and the T165 merge tree. The post-merge `windows-terminal #983` workflow also succeeded on the merge commit across native Windows, real Windows + Ubuntu WSL2, Ubuntu terminal integration, and macOS terminal integration.

This evidence does not permit calling the failure a flake or discarding it. It establishes only that the existing 10-second test-only child-observation deadline is not a deterministic hosted-Windows proof boundary for this supplemental marker.

## Existing product truth that remains unchanged

The T063 product and evidence requirements remain unchanged:

- native Windows ConPTY resize must still be accepted by the owned terminal;
- the child process must still independently observe and report the exact requested rows and columns;
- an incorrect child-observed size remains a hard test failure;
- missing child-observed proof remains a hard test failure after the authorized bounded observation window;
- active close must still prove the canonical owned-child cleanup truth;
- no terminal ownership, lifecycle, resize, cleanup, process, persistence, or evidence semantics may change.

The existing T063 100-cycle lifecycle soak remains unchanged.

## Exact repair authority after canonical amendment landing

Only after this amendment is `CLOSED_CANONICAL` may one forward-only repair modify:

```text
src/t063_soak_tests.rs
```

The repair authority is limited to the existing `OutputPump` wait fixture and the Windows-only block inside:

```text
active_close_and_windows_child_resize_guard
```

The repair MAY:

1. preserve the existing general `OutputPump::wait_for(...)` 10-second deadline for all existing callers;
2. add one private bounded wait helper that accepts an explicit `Duration`, with the existing read/channel/error/EOF/marker semantics unchanged;
3. make the existing `wait_for(...)` delegate to that helper with exactly 10 seconds;
4. use exactly 30 seconds only for the Windows child-observed `WINDS_T063_CHILD_SIZE_<rows> <cols>` marker in `active_close_and_windows_child_resize_guard`.

The repair MUST NOT:

- change `CYCLES=100`;
- change the primary T063 lifecycle soak;
- change the expected Windows child-observed rows or columns;
- accept command echo, prompt text, terminal title text, or any other output as resize proof;
- add retries, sleeps, polling side effects, test reruns, ignored tests, platform skips, or success-on-timeout behavior;
- change the PowerShell resize-observation command;
- change production Rust source, terminal/ConPTY code, cleanup windows, ownership semantics, lifecycle truth, persistence, protocol, workflow files, runner images, dependencies, lockfiles, schemas, migrations, or release behavior;
- authorize any T166 source or protocol-v2 implementation;
- reinterpret the original failed run as success.

Any timeout after the authorized 30-second observation budget remains a material failure and requires a new governance decision. Any wrong child-observed dimensions remain a material failure immediately.

## Why 30 seconds is bounded and non-product

The observed post-merge failure exhausted 10 seconds after the child command had already reached the owned shell. The exact same tree passed the same child proof in 2.72 seconds on the preceding hosted Windows qualification.

The repair therefore does not relax a product latency requirement. T063 defines a correctness/ownership/cleanup proof here, not a 10-second user-facing resize SLO. Thirty seconds is a test-only hosted child-process observation ceiling, remains finite, applies to exactly one supplemental Windows proof, and preserves hard failure when evidence never arrives.

No performance claim may be derived from this widened test-only observation ceiling.

## Required repair evidence

The repair must start from then-current exact canonical `main` after this amendment closes canonically and must qualify from scratch.

Required evidence:

- changed source scope exactly `src/t063_soak_tests.rs`;
- source diff proves only the authorized helper/delegation and single Windows child-marker call changed;
- focused `active_close_and_windows_child_resize_guard` PASS on native Windows;
- focused `controlled_terminal_lifecycle_soak_100_cycles` remains PASS where the canonical workflow runs it;
- `git diff --check` PASS;
- `cargo fmt --all -- --check` PASS;
- `cargo clippy --locked --all-targets --all-features -- -D warnings` PASS;
- repository `quality` SUCCESS on every actually-triggered required job;
- `windows-terminal` SUCCESS on every actually-triggered job;
- `t160-native-platform` SUCCESS on macOS, Linux, and Windows;
- any other actually-triggered workflow SUCCESS;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved material review threads;
- guarded normal merge using the exact expected repair head;
- merge tree, ordered parents, GitHub verification metadata, and every actually-triggered post-merge push workflow reconciled successfully.

A repair failure is material and MUST NOT be rerun on the same head to obtain green evidence.

## Amendment acceptance gate

This governance-only amendment is not canonical merely because it exists.

Its exact final candidate must satisfy:

- changed scope exactly this one amendment document;
- exact current base/head/tree and ahead/behind reconciliation;
- repository `quality` first-attempt execution on the exact amendment head;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved material review threads;
- visible ruleset/protection and mergeability reconciliation;
- current Herdr head/tree recheck with no source reuse;
- guarded normal merge with exact expected head;
- merge tree, ordered parents, and GitHub signature metadata reconciliation;
- every actually-triggered post-merge push workflow reconciled before this amendment is considered canonical.

Because this documentation-only amendment cannot itself repair the inherited T063 fixture, one narrow inherited-failure exception applies to the amendment candidate and amendment merge only:

- a triggered `t160-native-platform` may remain eligible only if macOS and Linux succeed;
- the Windows job must have zero failures before the T063 resource-cleanup step;
- the only failing test must be exactly `git::t063_soak_tests::active_close_and_windows_child_resize_guard`;
- the failure must be the same timeout waiting for the exact `WINDS_T063_CHILD_SIZE_33 101` marker under the unchanged 10-second fixture deadline;
- the primary `controlled_terminal_lifecycle_soak_100_cycles` must pass;
- no same-head rerun may replace, hide, or dilute the first-attempt failure;
- all other actually-triggered workflows must succeed;
- author and independent reviews must explicitly preserve this failure as inherited material evidence and report zero unrelated material findings.

If any of those conditions is false, this amendment is blocked and no repair is authorized.

The exception expires immediately after the authorized fixture repair lands. It does not generalize to T165 product code, T166, another T063 test, another platform, another marker, another workflow failure, or any future candidate.

Only after this amendment lands and its own post-merge evidence is reconciled may repository truth state:

```text
SPEC_012_TASKS_AMENDMENT_001=CLOSED_CANONICAL
T165_T063_WINDOWS_RESIZE_FIXTURE_REPAIR=AUTHORIZED_ONE_PATH_ONLY
T165=CANDIDATE_CLOSEOUT_BLOCKED_BY_REPAIR
T166=BLOCKED_BY_T165
T167..T185=BLOCKED_BY_PREDECESSOR
```
