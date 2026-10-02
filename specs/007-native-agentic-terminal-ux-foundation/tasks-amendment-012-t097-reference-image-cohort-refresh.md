# Spec 007 Tasks Amendment 012 — T097 Reference Image Cohort Refresh

Status: CLOSED_CANONICAL

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007, canonical T097 Tasks, canonical Amendments 010/011, canonical T097 finite-cohort maintenance through PR #266, and live exact-head T174 qualification evidence showing a newly assigned GitHub-hosted `ubuntu-24.04` image outside the current canonical finite cohort.

## Purpose

The canonical T097 preflight intentionally accepts only this exact finite cohort:

```text
20260831.293.1
20260907.300.1
20260920.314.1
```

That fail-closed policy is working as designed. GitHub has now assigned a fourth exact hosted image under the same committed `ubuntu-24.04` runner label:

```text
20260927.320.1
```

This amendment authorizes the smallest reversible governance refresh needed to recognize that already-observed fourth image without weakening any performance threshold, sample count, benchmark command, fixture, warmup policy, percentile calculation, permissions boundary, checkout policy, concurrency policy, or evidence schema.

## New material evidence

The following first-attempt exact-head T174 qualification failure remains material and MUST NOT be relabelled as a flake or pass:

```text
PR=278
T174_CANDIDATE=7155fb38818600785926b5eeec3b4ab2c635e267
T174_TREE=e7552d622f3f83c5c0c9f0b9563e6b91e2e61aef
T097_RUN=36938177602
T097_JOB=110623260296
RUNNER_LABEL=ubuntu-24.04
RUNNER_OS=Linux
RUNNER_ARCH=X64
IMAGE_OS=ubuntu24
OBSERVED_IMAGE_VERSION=20260927.320.1
ACCEPTED_COHORT=20260831.293.1,20260907.300.1,20260920.314.1
PERFORMANCE_BUILD_STARTED=NO
PERFORMANCE_SAMPLING_STARTED=NO
RESULT=FAIL_CLOSED_REFERENCE_ENVIRONMENT
```

The job log proves exact candidate checkout and identity verification succeeded, then the workflow stopped at the reference-environment preflight before Rust installation, release build, or any benchmark sampling because the observed image was outside the canonical finite cohort. This evidence proves policy drift only; it does not prove performance success or performance failure on `20260927.320.1`.

Historical T097 failures and previous cohort changes remain material evidence. This amendment does not rewrite or supersede them.

## Exact additional authority after canonical landing

After this amendment is guarded-landed and post-merge verified, one maintenance candidate may change exactly:

```text
.github/workflows/t097-performance.yml
```

The maintenance candidate may expand the canonical finite cohort from three members to exactly four members:

```text
20260831.293.1
20260907.300.1
20260920.314.1
20260927.320.1
```

The preflight MUST accept an observed `ImageVersion` only when it exactly equals one of those four literal values. Any other value, missing value, changed runner label, changed `ImageOS`, changed runner OS, or changed architecture MUST fail before build or sampling.

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

Every qualifying record MUST retain the exact observed `image_version`. The cohort remains qualification-policy metadata only and MUST NOT normalize the observed member into a generic environment label.

No run may be retried merely to obtain a preferred cohort member. Attempt 1 on the exact candidate remains material. A benchmark threshold failure on any accepted cohort member remains a real benchmark failure unless separately reconciled through accepted governance.

No cross-image performance-equivalence claim is created.

## Maintenance qualification gate

The authorized maintenance candidate is accepted only if its exact final candidate:

1. changes no path other than `.github/workflows/t097-performance.yml`;
2. changes only the finite image cohort/preflight text necessary to recognize `20260927.320.1`;
3. preserves every frozen benchmark semantic above;
4. passes repository deterministic quality/static analysis;
5. triggers T097 from the changed workflow path;
6. passes environment preflight on attempt 1 with one exact accepted cohort member;
7. executes the complete release-like T097 campaign with no threshold relaxation;
8. records exact candidate/tree/observed-image/method identity;
9. receives genuine exact-head Jev review;
10. receives exact-head Alibaba Open Code Review;
11. receives fresh independent correctness/security/governance/evidence-integrity review;
12. has zero unresolved material findings/review threads;
13. reconciles live main/base/head/tree/scope/ruleset/mergeability immediately before landing;
14. uses a guarded normal expected-head merge, with no rebase or history rewrite;
15. verifies merge tree, ordered parents, GitHub signature, and all applicable post-merge workflows.

## Interaction with T174

PR #278 remains blocked by the canonical T097 reference-environment policy until this amendment and its authorized maintenance change are canonical, unless repository governance explicitly classifies T097 as non-applicable to the T174 acceptance gate. The observed failure is preserved either way and MUST NOT be relabelled.

If the maintenance change lands before T174, PR #278 MUST forward-integrate then-current canonical `main` using a normal merge commit. That movement invalidates all candidate-bound CI and review evidence, and T174 must qualify again from the new exact candidate.

T175 receives no authority until T174 itself is genuinely `CLOSED_CANONICAL`.

## Explicit non-authorization

This amendment does NOT authorize:

- any FR-045..FR-053 threshold change;
- any sample-count, benchmark-command, fixture, warmup, percentile, or release-profile change;
- `ubuntu-latest`, wildcard/prefix/range image acceptance, dynamic discovery, or post-hoc cohort expansion;
- accepting a missing or fifth image version;
- rerun-to-green, automatic retry, tolerated failure, or `continue-on-error`;
- treating any historical T097 failure as a flake;
- changing production source, tests, dependencies, lockfiles, schemas, migrations, runtime/provider/browser behavior, daemon/IPC, remote execution, learning, Git authority, or landing automation;
- claiming performance equivalence between cohort members;
- modifying the T174 candidate under this amendment;
- merging T174 or activating T175 merely because this governance candidate exists.

If a fifth image appears, the fail-closed rule continues to apply and requires another separately accepted governance decision rather than automatic expansion.

## Amendment acceptance gate

This governance-only amendment may land only if its exact final candidate satisfies:

1. changed scope is exactly this one amendment document;
2. repository deterministic quality/static analysis succeeds on the exact head;
3. genuine exact-head Jev review executes and passes or has zero unresolved material findings under the repository's accepted Jev evidence semantics;
4. exact-head Alibaba Open Code Review executes genuinely;
5. fresh independent substantive review finds no unresolved material correctness/security/governance/evidence-integrity issue;
6. zero unresolved review threads;
7. exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
8. guarded normal expected-head merge;
9. canonical merge tree/ordered parents/signature verification;
10. every actually-triggered applicable post-merge push workflow succeeds.

Only after those gates may repository truth state:

```text
SPEC_007_TASKS_AMENDMENT_012=CLOSED_CANONICAL
T097_REFERENCE_IMAGE_COHORT=20260831.293.1,20260907.300.1,20260920.314.1,20260927.320.1
T097_IMAGE_COHORT_REFRESH_20260927_MAINTENANCE=AUTHORIZED
T097_IMAGE_COHORT_REFRESH_20260927_MAINTENANCE_LANDED=NO
T174=OPEN_BLOCKED_BY_T097_REFERENCE_ENVIRONMENT_AND_ITS_OWN_ACCEPTANCE_GATES
```

## Canonical closeout

```text
AMENDMENT_PR=279
BASE_SHA=974ab64859e2468a558f9497f3d9bd10d7956e63
QUALIFIED_HEAD_SHA=bfc75732134cad061acdab5177bf75d1e02e9eae
MERGE_SHA=1bba46ab82fd5b6a682032897a2de776b3846915
MERGE_TREE=7c16eb1d8f7d472ef236592350ca2c5a9dc6bb81
MERGE_PARENT_1=974ab64859e2468a558f9497f3d9bd10d7956e63
MERGE_PARENT_2=bfc75732134cad061acdab5177bf75d1e02e9eae
MERGE_SIGNATURE=verified valid
PRE_MERGE_QUALITY_RUN=36938844791 SUCCESS
POST_MERGE_QUALITY_RUN=36991485912 SUCCESS

JEV=TypeSafe local exact-range review 974ab64859e2468a558f9497f3d9bd10d7956e63..bfc75732134cad061acdab5177bf75d1e02e9eae
JEV_STATUS=PASS
JEV_COVERAGE=COMPLETE
JEV_BLOCKING_FINDINGS=0
JEV_TOOL_ERRORS=0
JEV_RESULT_PATH=C:\Winds\.local\jev-amend012-bfc7573-result.json
JEV_EXECUTION_NOTE=GitHub Actions lacked TYPESAFE_API_KEY; genuine local credential-backed exact-range execution is the qualifying Jev evidence. The CI credential blocker is not treated as PASS.

ALIBABA_OCR=v1.12.9 exact-range accounting
ALIBABA_OCR_RUN=36938895298
ALIBABA_OCR_REVIEWABLE_COUNT=0
ALIBABA_OCR_EXCLUDED_COUNT=1
ALIBABA_OCR_EXCLUSION=unsupported_ext
ALIBABA_OCR_RULE=resolved
MANUAL_MARKDOWN_REVIEW=CLEAN
INDEPENDENT_EXACT_HEAD_REVIEW=CLEAN
UNRESOLVED_MATERIAL_FINDINGS=0
UNRESOLVED_REVIEW_THREADS=0
ZERO_COST_CONSTRAINT=SATISFIED
MERGE_METHOD=NORMAL_MERGE_COMMIT_WITH_EXPECTED_HEAD_GUARD
```

Amendment 012 is therefore closed canonical. It authorizes only the separately qualified maintenance candidate that may add the literal fourth T097 runner image `20260927.320.1`; it does not itself land that maintenance change, does not relax benchmark semantics, and does not authorize T174/T175 landing or any product/runtime authority expansion.
