# Spec 007 Tasks Amendment 011 — T097 Reference Image Cohort Refresh

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007, canonical T097 Tasks, canonical Amendments 003/009/010, and live exact-head T168 qualification evidence showing a newly assigned GitHub-hosted `ubuntu-24.04` image outside the canonical finite cohort.

## Purpose

Canonical Amendment 010 deliberately fails closed when GitHub assigns any `ImageVersion` outside the exact cohort:

```text
20260831.293.1
20260907.300.1
```

That fail-closed behavior is working as designed. GitHub now assigns a third exact hosted image under the same committed `ubuntu-24.04` label:

```text
20260920.314.1
```

This amendment authorizes the smallest reversible governance refresh needed to recognize that already-observed third image without weakening any performance threshold, sample count, benchmark command, fixture, warmup policy, percentile calculation, permissions boundary, or evidence schema.

## New material evidence

The following first-attempt exact-head T168 qualification runs remain material failures and MUST NOT be relabelled as flakes or passes:

```text
PR=262
T168_CANDIDATE=e5329578221ac77f062c7b7f07e86477498f9a5c
T097_RUN=36336673486
T097_JOB=108668732070
RUNNER_LABEL=ubuntu-24.04
OBSERVED_IMAGE_VERSION=20260920.314.1
ACCEPTED_COHORT=20260831.293.1,20260907.300.1
PERFORMANCE_SAMPLING_STARTED=NO
RESULT=FAIL_CLOSED_REFERENCE_ENVIRONMENT

PR=262
T168_CANDIDATE=1aeeaf7e1daa8c8c3e3dd1cc4e0b6cdf037c8abf
T097_RUN=36337399875
RUNNER_LABEL=ubuntu-24.04
OBSERVED_IMAGE_VERSION=20260920.314.1
ACCEPTED_COHORT=20260831.293.1,20260907.300.1
RESULT=FAIL_CLOSED_REFERENCE_ENVIRONMENT
```

The first run's job log proves the workflow stopped at environment preflight before benchmark build/sampling because the observed image was outside the canonical finite cohort. This evidence does not prove performance success or failure on the new image.

## Exact additional authority after canonical landing

After this amendment is guarded-landed and post-merge verified, one maintenance candidate may change exactly:

```text
.github/workflows/t097-performance.yml
```

The maintenance candidate may expand the canonical finite cohort from two members to exactly three members:

```text
20260831.293.1
20260907.300.1
20260920.314.1
```

The preflight MUST accept an observed `ImageVersion` only when it exactly equals one of those three values. Any other value, missing value, changed runner label, changed `ImageOS`, changed runner OS, or changed architecture MUST fail before sampling.

The maintenance candidate MUST preserve unchanged:

```text
RUNNER_LABEL=ubuntu-24.04
EXPECTED_IMAGE_OS=ubuntu24
EXPECTED_RUNNER_OS=Linux
EXPECTED_RUNNER_ARCH=X64
RUST_TOOLCHAIN=1.97.1
BUILD_PROFILE=release
FR_045_THROUGH_FR_053_THRESHOLDS=UNCHANGED
SAMPLE_COUNTS=UNCHANGED
BENCHMARK_COMMANDS=UNCHANGED
FIXTURES=UNCHANGED
WARMUP_POLICY=UNCHANGED
PERCENTILE_CALCULATION=UNCHANGED
EVIDENCE_SCHEMA=WINDS_SPEC_007_T097_PERFORMANCE_EVIDENCE_V1
PERMISSIONS=contents:read
CHECKOUT_PERSIST_CREDENTIALS=false
CONCURRENCY_POLICY=UNCHANGED
```

Every qualifying record MUST retain the exact observed `image_version`. The cohort is qualification-policy metadata only and MUST NOT normalize the observed member into a generic environment label.

No run may be retried merely to obtain a preferred cohort member. Attempt 1 on the exact candidate remains material. A benchmark threshold failure on any accepted cohort member is a real benchmark failure unless separately reconciled through accepted governance.

No cross-image performance-equivalence claim is created.

## Maintenance qualification gate

The authorized maintenance candidate is accepted only if its exact final candidate:

1. changes no path other than `.github/workflows/t097-performance.yml`;
2. changes only the finite image cohort/preflight text necessary to recognize `20260920.314.1`;
3. preserves every frozen benchmark semantic above;
4. passes repository deterministic quality/static analysis;
5. triggers T097 from the changed workflow path;
6. passes environment preflight on attempt 1 with one exact accepted cohort member;
7. executes the complete release-like T097 campaign with no threshold relaxation;
8. records exact candidate/tree/observed-image/method identity;
9. receives exact-head Jev review;
10. receives exact-head Alibaba Open Code Review;
11. receives fresh independent correctness/security/governance/evidence-integrity review;
12. has zero unresolved material findings/review threads;
13. reconciles live main/base/head/tree/scope/ruleset/mergeability immediately before landing;
14. uses a guarded normal expected-head merge, with no rebase or history rewrite;
15. verifies merge tree, ordered parents, GitHub signature, and all applicable post-merge workflows.

## Interaction with T168

PR #262 remains blocked by the canonical T097 reference-environment policy until this amendment and its authorized maintenance change are canonical. Existing T168 qualification failures caused solely by the third image remain material historical evidence and MUST NOT be rewritten.

After the maintenance change lands canonically, PR #262 MUST forward-integrate then-current canonical `main` using a normal merge commit. That movement invalidates all prior candidate-bound CI and review evidence. T168 must then qualify from scratch on the new exact candidate.

T169 receives no authority until T168 itself is genuinely `CLOSED_CANONICAL`.

## Explicit non-authorization

This amendment does NOT authorize:

- any FR-045..FR-053 threshold change;
- any sample-count, benchmark-command, fixture, warmup, percentile, or release-profile change;
- `ubuntu-latest`, wildcard/prefix/range image acceptance, dynamic discovery, or post-hoc cohort expansion;
- accepting a missing or fourth image version;
- rerun-to-green, automatic retry, tolerated failure, `continue-on-error`, or platform skip;
- treating any historical T097 failure as a flake;
- changing production source, tests, dependencies, lockfiles, schemas, migrations, runtime/provider/browser behavior, daemon/IPC, remote execution, learning, Git authority, or landing automation;
- claiming performance equivalence between cohort members;
- merging T168 or activating T169 before T168's own exact-head gates pass.

If a fourth image appears, Amendment 010's fail-closed rule continues to apply: it requires another separately accepted governance decision rather than automatic expansion.

## Amendment acceptance gate

This governance-only amendment may land only if its exact final candidate satisfies:

1. changed scope is exactly this one amendment document;
2. repository deterministic quality/static analysis succeeds on the exact head;
3. exact-head Jev review executes genuinely and passes or has zero unresolved material findings under the repository's accepted Jev evidence semantics;
4. exact-head Alibaba Open Code Review executes genuinely;
5. fresh independent substantive review finds no unresolved material correctness/security/governance/evidence-integrity issue;
6. zero unresolved review threads;
7. exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
8. guarded normal expected-head merge;
9. canonical merge tree/ordered parents/signature verification;
10. every actually-triggered applicable post-merge push workflow succeeds.

Only after those gates may repository truth state:

```text
SPEC_007_TASKS_AMENDMENT_011=CLOSED_CANONICAL
T097_REFERENCE_IMAGE_COHORT=20260831.293.1,20260907.300.1,20260920.314.1
T097_IMAGE_COHORT_REFRESH_MAINTENANCE=AUTHORIZED
T097_IMAGE_COHORT_REFRESH_MAINTENANCE_LANDED=NO
T168=OPEN_BLOCKED_BY_T097_REFERENCE_ENVIRONMENT
```
