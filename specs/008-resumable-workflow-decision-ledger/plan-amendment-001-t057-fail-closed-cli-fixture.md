# Spec 008 Plan Amendment 001 — T057 Fail-Closed Terminal-Proof Fixture

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 003 terminal lifecycle/fault truth, canonical Spec 007 Amendments 005, 006, and 007 bounded-cleanup fixture precedents, canonical Spec 008 Plan exact-head acceptance requirements, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

Spec 008 Plan PR #142 is in qualification. Its current exact head `9e629c5513ebd193dd3027df360c1b5a649bfcd8` / tree `afc93a3c409f59ac0e826b036783b5d9c6d1fb18` changes only:

```text
specs/008-resumable-workflow-decision-ledger/plan.md
```

The first exact-head repository `quality` run on that candidate is run `34395694562`. Ubuntu job `102614732694` succeeded. macOS job `102614732764` passed exact checkout verification, Format, Clippy, 420 non-ignored unit tests, and all preceding integration tests, then failed only:

```text
tests/t057_cli.rs
minimal_cli_proves_workspace_profiles_execution_and_terminal_paths
```

The exact command failure was:

```text
winds: terminal terminate could not prove owned child exit inside bounded cleanup window
```

The failure occurred when the T057 CLI integration fixture called `terminal-proof` and unconditionally required command success. That command starts a native terminal and immediately requests controlled termination through the existing `TerminalExecution::terminate()` path.

This failed first attempt is material evidence. It MUST NOT be classified as a flake, retried into acceptance, or erased by later green evidence.

A single focused local macOS diagnostic execution on the same source code later followed the proven-cleanup branch and passed. That local result is diagnosis only; it does not replace or invalidate the failed CI attempt.

## Diagnosis

Production behavior already fails closed.

`TerminalExecution::controlled_cleanup(...)` uses the existing 500 ms production cleanup window and distinguishes these canonical outcomes:

1. `ExitedBeforeCleanup(exit)` -> durable `EXITED`, `WINDS_OBSERVED`, `PROCESS_EXITED` truth;
2. `Terminated(exit)` -> durable `INTERRUPTED`, `WINDS_OBSERVED`, controlled close reason `TERMINATED_BY_WINDS` for `terminate()`;
3. `Unproven` -> revoke session ownership, suppress later drop cleanup after ownership loss, persist durable `OWNERSHIP_LOST`, preserve absent `ended_unix_ms` / `duration_ms`, set `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`, emit `TerminalOwnershipLostAfterCleanupFailure`, and return the bounded-cleanup error above.

The wrapper Drop path explicitly performs no further cleanup when ownership has been revoked.

This is the same fail-closed production contract previously recognized by canonical Spec 007 Amendments 005 and 007. Amendment 006 additionally recognized that a child may exit naturally between a prior liveness observation and controlled cleanup and that `EXITED` must not be coerced into `INTERRUPTED`.

The T057 fixture currently assumes only outcome 2 is valid:

```text
terminal-proof command succeeds
execution.status == INTERRUPTED
terminal.close_reason == TERMINATED_BY_WINDS
```

That assertion is stronger than the existing production truth. The new macOS CI failure proves the T057 fixture can observe outcome 3. The production implementation preserved authority correctly; the fixture rejected that truthful outcome.

No evidence justifies changing the 500 ms production cleanup budget, kill/reap behavior, ownership proof, persistence semantics, or command retry behavior.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` is `5f25341c2f317f528e542d4d90a112c784ab24dc` or a governance-only forward descendant;
- Spec 008 entry and formal specification are `CLOSED_CANONICAL`;
- Spec 008 Plan is authorized for creation but is not `CLOSED_CANONICAL`;
- PR #142 remains the active Plan candidate or a forward-only successor;
- failed run `34395694562`, macOS job `102614732764`, remains preserved as the original material failure and is not replaced by rerun-to-green evidence;
- the failure remains attributable to the T057 success-only terminal-proof assertion with the exact bounded-cleanup error above;
- existing 500 ms production terminal cleanup, `TerminalExecution` ownership revocation, and `OWNERSHIP_LOST` persistence semantics remain unchanged;
- no later canonical repair has already removed the need for this amendment.

Landing this amendment does not close the Plan and does not authorize Spec 008 Tasks or implementation.

## Exact additional repair authority

Only after this amendment lands canonically may a repair modify:

```text
tests/t057_cli.rs
```

and only inside:

```rust
#[test]
fn minimal_cli_proves_workspace_profiles_execution_and_terminal_paths()
```

The authorized change is limited to the existing `terminal-proof` assertion block beginning with the invocation for `t057-terminal-proof` and ending with its terminal proof assertions. Workspace-open, profile discovery, shell-command execution, cross-workspace rejection, clone coverage, helpers, and every other T057 behavior remain unchanged.

The repaired fixture must distinguish exactly the existing truthful terminal outcomes below.

### Truthful outcome A — native shell exited before controlled cleanup

If `terminal-proof` succeeds and its persisted execution snapshot reports the natural-exit path, the fixture MUST prove:

- execution kind is `TERMINAL`;
- execution status is exactly `EXITED`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` are present;
- terminal profile identity equals the selected exact profile;
- terminal close reason is exactly `PROCESS_EXITED`;
- proof profile identity equals the selected exact profile;
- terminal Git observations remain empty.

The fixture MUST NOT coerce this observed natural exit to `INTERRUPTED`.

### Truthful outcome B — controlled termination is proven

If `terminal-proof` succeeds and its persisted execution snapshot reports controlled termination, the fixture MUST prove:

- execution kind is `TERMINAL`;
- execution status is exactly `INTERRUPTED`;
- status source is `WINDS_OBSERVED`;
- `ended_unix_ms` and `duration_ms` are present;
- terminal profile identity equals the selected exact profile;
- terminal close reason is exactly `TERMINATED_BY_WINDS`;
- proof profile identity equals the selected exact profile;
- terminal Git observations remain empty.

### Truthful outcome C — bounded cleanup is unproven

If `terminal-proof` fails, the fixture may accept only the exact existing bounded-cleanup failure:

```text
winds: terminal terminate could not prove owned child exit inside bounded cleanup window
```

The fixture MUST then:

- NOT retry `terminal-proof`, `terminate`, `close`, or any other cleanup operation;
- use the existing read-only `execution --repo ... --execution-id t057-terminal-proof` inspection command exactly once to inspect durable truth after the failed command process has returned;
- require the inspection command to succeed;
- require execution kind `TERMINAL`;
- require status exactly `OWNERSHIP_LOST`;
- require status source exactly `WINDS_OBSERVED`;
- require `ended_unix_ms` and `duration_ms` to remain null because exit was not proven;
- require terminal profile identity to equal the selected exact profile;
- require terminal close reason exactly `OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`;
- require at least one exact `TerminalOwnershipLostAfterCleanupFailure` event with source `WINDS_OBSERVED`;
- require terminal Git observations to remain empty.

The `execution` inspection is observation of already-final durable state. It must not trigger cleanup retry, lifecycle promotion, or ownership reacquisition.

Any other command failure, lifecycle state, source, close reason, event shape, end/duration shape, or profile mismatch MUST fail the fixture.

## Values and behavior that MUST remain unchanged

This amendment does NOT authorize changes to:

- `src/terminal.rs`;
- `src/execution.rs`;
- `src/cli_workspace.rs`;
- `src/store.rs`;
- any production source file;
- the 500 ms terminal cleanup window;
- kill/reap order or polling cadence;
- `TerminalDropCleanupOutcome` or `TerminalFinalization` semantics;
- ownership revocation or suppression of cleanup after ownership loss;
- `OWNERSHIP_LOST` persistence or fact-source rules;
- the `terminal-proof` CLI contract;
- T060/T096 fixture behavior already governed by canonical amendments;
- other T057 tests or helpers;
- workflows, runner images, dependencies, lockfiles, schemas, migrations, performance thresholds, platform claims, runtime authority, or Git authority;
- Spec 008 Plan contents except the normal later forward-integration/base reconciliation required after this repair lands;
- Spec 008 Tasks or implementation authority.

No timeout increase, sleep, retry loop, rerun-until-green strategy, ignored test, platform skip, assertion suppression, or conversion of unproven cleanup into success is authorized.

## Required evidence for the governance amendment

This amendment file itself is governance-only and is not canonical merely because it exists.

Its exact final candidate must satisfy the governance-only Standard Acceptance Gate:

- exact-head repository `quality` SUCCESS;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review bound to the exact candidate;
- zero unresolved material findings and review threads;
- exact one-file scope reconciliation;
- live main/ruleset/mergeability race reconciliation;
- guarded expected-head merge;
- post-merge canonical main/tree and every actually triggered applicable push check verified.

If the amendment candidate itself encounters the same T057 failure before repair authority exists, that failure remains material and execution must stop for another explicit governance decision; it MUST NOT be rerun away.

## Required evidence after amendment use

After this amendment is canonical, the test-only repair must start from then-current exact canonical `main` and be qualified from scratch.

Required repair evidence includes:

- exact-head repository `quality` SUCCESS on Ubuntu and macOS;
- the repaired `minimal_cli_proves_workspace_profiles_execution_and_terminal_paths` fixture demonstrably executes and passes while retaining strict assertions for all three truthful branches;
- no production source change;
- applicable terminal/platform workflows triggered by `tests/t057_cli.rs` remain green;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate;
- zero unresolved material findings/review threads;
- exact one-file / one-function scope reconciliation;
- live main/ruleset/mergeability race reconciliation;
- guarded expected-head landing;
- post-merge canonical main/tree plus every actually triggered applicable push check verified.

The original failed Plan run `34395694562` remains historical material evidence. A later green repair and later green Plan candidate MUST NOT relabel that attempt as a flake or erase it.

## Plan qualification after repair

After the test-only repair is canonically landed and post-merge verified:

- PR #142 must forward-integrate the repaired canonical main without rebase, force-push, or history rewrite;
- Plan exact base/head/tree/scope metadata must be reconciled to live truth;
- if Plan documentation contains a stale planning-base claim that is intended to describe the exact qualification base, it must be updated forward-only;
- all Plan candidate-bound CI/reviews from pre-integration heads become historical only;
- the Plan must be qualified from scratch under its exact final head and current canonical base;
- only a successfully guarded-merged and post-merge-verified Plan may authorize Spec 008 Tasks creation.

## Explicit non-authorization

This amendment does not authorize:

- rerunning failed Plan run `34395694562` as a substitute for repair;
- classifying the failure as a flake;
- widening production terminal cleanup timeouts;
- retrying cleanup after ownership loss;
- weakening proof of child exit;
- converting `OWNERSHIP_LOST` into `INTERRUPTED`/`EXITED` without proof;
- suppressing command failure in the unproven branch;
- modifying production terminal lifecycle code;
- dependencies, schemas, migrations, runtime/provider/browser/network changes;
- daemon/server/socket/RPC/IPC;
- remote execution;
- learning/vector memory/plugin/workflow-engine expansion;
- automatic candidate selection, Git mutation, PR creation, merge, or landing;
- Spec 008 Tasks or implementation before canonical Plan closure.

## Acceptance of this amendment

Only after exact-candidate governance qualification, guarded expected-head landing, and post-merge verification may the exact T057 test-only repair authority above be used.
