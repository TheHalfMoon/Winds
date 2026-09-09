# Spec 008 Tasks Amendment 001 — T101 Owning-Module Registration

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0, canonical Spec 008 Spec/Plan/Tasks, canonical Spec 007 T097 performance governance, and exact live PR #148 qualification evidence.

## Purpose

T101 is a pure workflow-domain slice. Its canonical Tasks authorize `src/main.rs` only as an optional test/module-registration surface, not because T101 requires production CLI or workbench behavior.

The first T101 candidate used that optional route only for two `#[cfg(test)]` module declarations. Because canonical T097 conservatively watches every `src/main.rs` change, that test-only registration caused the unrelated workbench performance workflow to run. The exact T097 run then correctly failed its pinned hosted-runner image preflight before any benchmark sample.

Exact historical evidence:

```text
T101_PR=148
T101_HISTORICAL_HEAD=4dc9449a9a54379fd1fa2d5fda8609cc9ad7fb2c
T097_RUN=34416980605
T097_RUN_ATTEMPT=1
T097_RESULT=FAILURE
T097_EXPECTED_IMAGE=20260831.293.1
T097_OBSERVED_IMAGE=20260907.300.1
T097_PERFORMANCE_SAMPLING_STARTED=NO
```

A separately governed repin candidate, PR #150, later demonstrated the GitHub-hosted `ubuntu-24.04` pool was concurrently serving both versions:

```text
REPIN_PR=150
REPIN_HEAD=589166cfd4ec4e33c178403b194fe8ca7ce2b694
REPIN_T097_RUN=34417704731
REPIN_T097_RUN_ATTEMPT=1
REPIN_T097_RESULT=FAILURE
REPIN_EXPECTED_IMAGE=20260907.300.1
REPIN_OBSERVED_IMAGE=20260831.293.1
REPIN_PERFORMANCE_SAMPLING_STARTED=NO
REPIN_PR_MERGED=NO
```

Neither failure is a flake. Neither may be rerun until green. This amendment does not weaken, bypass, or reinterpret T097.

Instead, this amendment removes the unnecessary T101 coupling to `src/main.rs`: register the new pure domain module and its focused test from the already-compiled owning `src/domain.rs` module. This keeps the final T101 candidate outside the workbench/CLI entry surface and therefore outside T097's canonical path trigger by construction.

## Exact T101 path authority adjustment

After this amendment is canonical, T101 final candidate authority becomes:

```text
src/workflow.rs
src/t101_workflow_domain_tests.rs
src/domain.rs    # module/test registration only
```

For T101, `src/main.rs` is no longer an authorized changed path.

The smallest permitted `src/domain.rs` change is:

- one module declaration exposing `src/workflow.rs` through the already-compiled domain ownership seam; and
- one `#[cfg(test)]` declaration registering `src/t101_workflow_domain_tests.rs`;
- a narrowly scoped `dead_code` allowance/reason only if required because later persistence/CLI callers remain dependency-blocked.

No existing domain behavior may be changed by this registration adjustment.

The T101 implementation remains exactly the canonical Tasks purpose:

- stable workflow/stage identity;
- distinct successor attempt identity and exact predecessor lineage;
- checked attempt ordinal progression;
- closed lifecycle state model and complete transition matrix;
- exact operation/precondition-bound idempotency;
- source truth separated from transition authority;
- fail-closed terminal cancellation/completion authority;
- procedural workflow-completion eligibility only;
- no persistence, migration, runtime, terminal child, provider, CLI command, dependency, or Git mutation.

## Qualification consequence for PR #148

After this amendment lands canonically and its post-merge checks succeed, PR #148 MUST:

1. forward-integrate then-current canonical `main` without rebase or history rewrite;
2. remove the T101-only `src/main.rs` declarations;
3. add only the owning-module registration described above in `src/domain.rs`;
4. adapt test module paths/imports only as required by the module hierarchy;
5. preserve the T101 domain semantics already authorized;
6. run all exact-candidate gates from scratch because HEAD/TREE moved;
7. preserve both historical T097 failures as material evidence;
8. require T097 only if the final T101 diff itself actually matches the canonical T097 trigger surface after this adjustment.

The candidate still requires repository `quality`, focused T101 tests, release-candidate and platform/regression workflows that actually apply/trigger, Author review, Ponytail/YAGNI review, fresh independent review, race reconciliation, guarded merge, and post-merge verification.

## Downstream registration discipline

T102-T108 already authorize changes to `src/store.rs` and/or `src/workflow.rs`. Their focused tests SHOULD be registered within an already-authorized owning module when Rust module structure permits, rather than editing `src/main.rs` solely for `#[cfg(test)]` registration. This is a scope-minimization rule, not new path authority.

`src/main.rs` remains available only where a later task has an actual product dispatch/entry requirement under canonical Tasks. In particular, T109's real CLI integration remains separately governed and is not authorized or modified by this amendment.

## Explicit non-authorization

This amendment does NOT authorize:

- changing `.github/workflows/t097-performance.yml`;
- changing T097 trigger paths, exact ImageVersion discipline, runner selection, thresholds, benchmark method, sample counts, evidence schema, or retry policy;
- dismissing or reclassifying runs 34416980605 or 34417704731;
- production CLI/workbench behavior;
- T102 or later implementation;
- dependency, lockfile, migration, schema, runtime/provider/browser, daemon/IPC, remote, learning, automatic Git mutation, or landing behavior;
- moving T101 domain logic into an unrelated existing module merely to avoid CI;
- skipping any workflow that still triggers on the final exact T101 candidate.

## Amendment acceptance gate

This governance-only amendment may land only if its exact final candidate satisfies:

- changed scope is exactly this amendment document;
- repository `quality` succeeds;
- Author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads;
- exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded normal expected-head merge;
- merge tree/ordered parents/signature verification;
- every actually-triggered post-merge push workflow succeeds.

Only then may repository truth state:

```text
SPEC_008_TASKS_AMENDMENT_001=CLOSED_CANONICAL
T101_OWNING_MODULE_REGISTRATION=AUTHORIZED
T101=OPEN_REQUALIFICATION_REQUIRED
T102..T113=BLOCKED_BY_DEPENDENCY
```
