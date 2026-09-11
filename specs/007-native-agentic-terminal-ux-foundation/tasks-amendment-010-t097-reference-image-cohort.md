# Spec 007 Tasks Amendment 010 — T097 Finite Reference Image Cohort

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007, canonical T097 Tasks and Amendment 003 performance discipline, canonical Amendment 009 evidence, canonical Spec 008 T109 authority, and the Founder directive in the active project session to continue the dependency-ordered Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

T097 currently requires the GitHub-hosted `ubuntu-24.04` runner to expose one exact committed `ImageVersion` before any benchmark sample may qualify. Live evidence now proves that exact image version is not selectable by the workflow while the hosted pool is rolling between two already-observed image versions.

The current fail-closed guard is functioning correctly. The problem is not a benchmark failure and not a reason to weaken any performance threshold. It is a reference-environment selection defect: the workflow declares one exact hosted image version but GitHub can assign another exact image version under the same committed runner label.

This amendment authorizes the smallest reversible governance change that preserves exact image identity and first-attempt discipline: replace the single expected image version with one finite, predeclared two-member reference image cohort containing only the two versions already proven by material historical evidence.

This amendment does not claim that the two images are identical or equivalent. Every qualifying evidence record must retain the exact image version actually assigned to that candidate.
## Material evidence

The following first-attempt failures remain material and MUST NOT be relabelled as flakes or passes:

```text
PR_148_T097_RUN=34416980605
EXPECTED_IMAGE_VERSION=20260831.293.1
OBSERVED_IMAGE_VERSION=20260907.300.1
PERFORMANCE_SAMPLING_STARTED=NO

PR_150_REPIN_RUN=34417704731
EXPECTED_IMAGE_VERSION=20260907.300.1
OBSERVED_IMAGE_VERSION=20260831.293.1
PERFORMANCE_SAMPLING_STARTED=NO

PR_159_PRIOR_HEAD_T097_RUN=34542151720
CANDIDATE=fa6641e83cfb8dd434c237d033feb7ecb0198725
EXPECTED_IMAGE_VERSION=20260831.293.1
OBSERVED_IMAGE_VERSION=20260907.300.1
PERFORMANCE_SAMPLING_STARTED=NO

PR_159_CURRENT_HEAD_T097_RUN=34543520483
CANDIDATE=07756c301abc873ce436751ea6fb38095c56d6fd
EXPECTED_IMAGE_VERSION=20260831.293.1
OBSERVED_IMAGE_VERSION=20260907.300.1
PERFORMANCE_SAMPLING_STARTED=NO
```

Together these runs prove both image versions can be assigned under the same `ubuntu-24.04` label and that a one-value repin cannot make the hosted allocation selectable.
## Why the simpler path is insufficient

Canonical Amendment 009 authorized one forward-only repin from `20260831.293.1` to `20260907.300.1`. PR #150 exercised that authority and failed on attempt 1 because GitHub assigned the older image. Amendment 009 explicitly forbids chasing hosted-runner allocation through reruns or repeated repins.

Re-running a failed candidate until the desired image appears is therefore prohibited. Repeating the one-value repin is also prohibited. Removing T097 from a changed `src/main.rs` candidate would weaken Spec 008's applicable-regression discipline and is not authorized.

The remaining minimal path is to predeclare the exact finite set that the hosted service is demonstrably assigning and fail closed outside that set.

## Exact additional authority after canonical landing

After this amendment is guarded-landed and post-merge verified, one maintenance candidate may change exactly:

```text
.github/workflows/t097-performance.yml
```

The maintenance candidate may replace the single expected image version with the exact closed cohort:

```text
20260831.293.1
20260907.300.1
```

The preflight MUST accept an observed `ImageVersion` only when it exactly equals one cohort member. Any other value, missing value, changed runner label, changed `ImageOS`, changed runner OS, or changed architecture MUST fail before sampling.
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
WARMUP_POLICY=UNCHANGED
PERCENTILE_CALCULATION=UNCHANGED
EVIDENCE_SCHEMA=WINDS_SPEC_007_T097_PERFORMANCE_EVIDENCE_V1
PERMISSIONS=contents:read
CHECKOUT_PERSIST_CREDENTIALS=false
CONCURRENCY_POLICY=UNCHANGED
```

The evidence assembly MUST continue to record the exact observed `image_version` for the qualifying run. The cohort is qualification policy metadata only; it MUST NOT erase or normalize the observed member into a generic label.

No run may be retried merely to obtain a preferred cohort member. Attempt 1 on the exact candidate remains material. A benchmark result that fails a frozen threshold on either accepted member is a benchmark failure and MUST NOT be retried away or blamed on cohort membership without a separately accepted governance decision.

No cross-image performance equivalence claim is created. A successful record proves only that the exact candidate met the frozen thresholds on the exact observed cohort member recorded in that evidence.
## Maintenance qualification gate

The image-cohort maintenance candidate is not accepted merely because the guard accepts one cohort member. Its exact final candidate MUST:

1. change no path other than `.github/workflows/t097-performance.yml`;
2. preserve the exact two-member cohort and all frozen benchmark semantics above;
3. run repository `quality` successfully;
4. trigger T097 from the changed workflow path;
5. pass the environment preflight on attempt 1 with an observed `ImageVersion` equal to one exact cohort member;
6. execute the complete release-like T097 campaign with no threshold relaxation;
7. retain exact candidate/tree/observed-image/method identity in evidence;
8. receive Author correctness/safety/governance/evidence-integrity review;
9. receive Ponytail/YAGNI review;
10. receive fresh independent substantive review on the exact final candidate;
11. have zero unresolved material findings or review threads;
12. reconcile current main/base/head/tree/scope/ruleset/mergeability immediately before landing;
13. use guarded normal merge with exact expected head;
14. verify merge tree, ordered parents, and GitHub signature;
15. verify every actually-triggered applicable post-merge push workflow before the maintenance change is canonical.

If the exact first attempt observes an image outside the two-member cohort, the maintenance candidate fails closed. This amendment does not authorize automatically adding the new value. A new material environment must be reconciled through a separately accepted amendment.
## Interaction with open Spec 008 T109

PR #159 remains open and blocked. Its T097 failures on `fa6641e83cfb8dd434c237d033feb7ecb0198725` and `07756c301abc873ce436751ea6fb38095c56d6fd` remain material historical evidence and are not converted to passes by this amendment.

After the image-cohort maintenance change lands canonically, PR #159 MUST forward-integrate then-current canonical `main` without rebase or history rewrite. That movement invalidates all prior candidate-bound CI and reviews. T109 must then qualify from scratch on the new exact candidate, including whatever workflows actually trigger.

T110 receives no authority until T109 itself closes canonically.

## Explicit non-authorization

This amendment does NOT authorize:

- any FR-045..FR-053 threshold change;
- any sample-count, benchmark-command, fixture, warmup, percentile, or release-profile change;
- `ubuntu-latest`, a wildcard image version, prefix/range matching, dynamically discovered cohort expansion, or post-hoc image acceptance;
- accepting a missing or third image version;
- rerun-to-green, automatic retry, tolerated failure, `continue-on-error`, or platform skip;
- treating any historical T097 failure as a flake;
- changing production source, tests, dependencies, lockfiles, migrations, schemas, runtime/provider/browser behavior, daemon/IPC, remote execution, learning, Git authority, or landing automation;
- claiming performance equivalence between cohort members;
- T109 merge or any successor Spec 008 authority.
## Amendment acceptance gate

This governance-only amendment may land only if its exact final candidate satisfies:

- changed scope is exactly this one amendment document;
- repository `quality` succeeds on the exact head;
- Author correctness/safety/governance/evidence-integrity review passes;
- Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads;
- exact main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded normal expected-head merge;
- canonical merge tree/ordered parents/signature verification;
- every actually-triggered applicable post-merge push workflow succeeds.

Only after those gates may repository truth state:

```text
SPEC_007_TASKS_AMENDMENT_010=CLOSED_CANONICAL
T097_REFERENCE_IMAGE_COHORT=20260831.293.1,20260907.300.1
T097_IMAGE_COHORT_MAINTENANCE=AUTHORIZED
T097_IMAGE_COHORT_MAINTENANCE_LANDED=NO
T109=OPEN_BLOCKED_BY_T097_REFERENCE_ENVIRONMENT
```

The amendment is intentionally reversible: if GitHub later provides a selectable immutable hosted image identity, a future accepted amendment may return T097 to one exact selectable reference image without changing the frozen product thresholds.