# Spec 007 Tasks Amendment 009 — T097 Reference Runner Image Repin

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0, canonical Spec 007, canonical Tasks and Tasks Amendment 003 T097 performance discipline, canonical Spec 008 Tasks, and the Founder directive to continue the dependency-ordered Winds program without weakening evidence gates.

## Purpose

The canonical T097 performance workflow deliberately fails closed when the GitHub-hosted reference image no longer matches the exact committed `ImageVersion`. That guard is now functioning as designed and blocks current downstream pull-request qualification before any performance sample is admitted.

Exact observed blocker:

```text
BLOCKED_PR=148
BLOCKED_CANDIDATE=4dc9449a9a54379fd1fa2d5fda8609cc9ad7fb2c
T097_RUN=34416980605
T097_RUN_ATTEMPT=1
T097_RESULT=FAILURE
FAILURE_STEP=Verify pinned reference environment before sampling
RUNNER_LABEL=ubuntu-24.04
RUNNER_OS=Linux
RUNNER_ARCH=X64
IMAGE_OS=ubuntu24
EXPECTED_IMAGE_VERSION=20260831.293.1
OBSERVED_IMAGE_VERSION=20260907.300.1
PERFORMANCE_SAMPLING_STARTED=NO
```

The failure is material evidence and MUST NOT be relabelled as a flake or bypassed. No rerun of that failed candidate can qualify merely by receiving a different hosted runner image.

This amendment authorizes the smallest governance-preserving maintenance action: a separately qualified forward-only change of the exact committed T097 expected runner-image version from `20260831.293.1` to `20260907.300.1`, followed by a complete fresh T097 qualification on the repin candidate.

## Mandatory predecessor truth

Before this amendment is used, live repository truth MUST continue to prove:

- Spec 007 first implementation program is canonically closed;
- `.github/workflows/t097-performance.yml` remains the accepted T097 measurement workflow from Amendment 003;
- its runner label remains `ubuntu-24.04`, `EXPECTED_IMAGE_OS=ubuntu24`, `EXPECTED_RUNNER_OS=Linux`, and `EXPECTED_RUNNER_ARCH=X64`;
- the only observed blocker requiring this amendment is the exact `ImageVersion` mismatch above;
- no T097 threshold relaxation or benchmark-method change is needed or authorized;
- Spec 008 T101 remains open and unmerged while this prerequisite is repaired.

If any of those facts moves materially before use, this amendment does not authorize guessing a replacement value or broadening scope.

## Exact additional authority after canonical landing

Only after this amendment itself is guarded-landed and post-merge verified, one maintenance candidate may change exactly:

```text
.github/workflows/t097-performance.yml
```

and only this semantic value:

```text
EXPECTED_IMAGE_VERSION: 20260831.293.1
->
EXPECTED_IMAGE_VERSION: 20260907.300.1
```

Whitespace-only formatting necessary to preserve YAML validity is permitted but should be avoided. No other workflow semantic is authorized.

The maintenance candidate MUST preserve unchanged:

```text
RUNNER_LABEL=ubuntu-24.04
EXPECTED_IMAGE_OS=ubuntu24
EXPECTED_RUNNER_OS=Linux
EXPECTED_RUNNER_ARCH=X64
RUST_TOOLCHAIN=1.97.1
BUILD_PROFILE=release
T097_THRESHOLDS=UNCHANGED
MEASUREMENT_METHOD=UNCHANGED
PERMISSIONS=contents:read
CHECKOUT_PERSIST_CREDENTIALS=false
```

## Repin qualification gate

The repin is not accepted merely because the newly observed image version is copied into the workflow. Its exact final candidate MUST:

1. run repository `quality` successfully;
2. trigger T097 from the changed workflow path;
3. pass the pinned reference-environment check with observed `ImageVersion=20260907.300.1` before sampling;
4. execute the complete existing T097 release-like benchmark campaign without threshold relaxation;
5. preserve exact candidate/tree/environment/method identity in generated evidence;
6. receive Author correctness/safety/governance/evidence-integrity review;
7. receive Ponytail/YAGNI review;
8. receive a fresh independent substantive review on the exact final candidate;
9. have zero unresolved material findings or review threads;
10. reconcile exact main/base/head/tree/scope/ruleset/mergeability immediately before landing;
11. use guarded normal merge with exact expected head;
12. verify merge tree, ordered parents, and GitHub signature;
13. verify every actually-triggered applicable post-merge push workflow before the repin is considered canonical.

If the hosted image changes again before the qualifying campaign starts, or the exact environment guard observes any other mismatch, the repin candidate fails closed. This amendment does not authorize chasing a runner value through repeated reruns. A new observed environment must be separately reconciled and, if needed, separately amended.

## Interaction with open Spec 008 T101

PR #148 and all evidence bound to its current head remain historical while this maintenance dependency is repaired. The current T097 failure remains material.

After the repin lands canonically and is post-merge verified, PR #148 MUST forward-integrate then-current canonical `main` without rebase or history rewrite. Its HEAD/TREE movement invalidates all previous candidate-bound CI and reviews. T101 must then qualify from scratch on the new exact candidate before it can land.

No T102 authority is created by this amendment or by the repin. T102 remains dependency-blocked until T101 itself closes canonically.

## Explicit non-authorization

This amendment does NOT authorize:

- changing any FR-045..FR-053 threshold;
- changing sample counts, benchmark commands, warmup policy, percentile calculation, evidence schema, fixture behavior, or release profile;
- changing runner label, operating system family, architecture, Rust toolchain, action versions, permissions, checkout credential behavior, or concurrency semantics;
- accepting `ubuntu-latest`, dynamically discovered image identity, post-hoc image acceptance, or an unpinned reference environment;
- adding retries, rerun-to-green policy, tolerated failures, `continue-on-error`, conditional bypasses, or platform skips;
- production source, tests, dependencies, lockfiles, migrations, schemas, runtime behavior, provider/browser behavior, daemon/IPC, remote execution, learning, Git authority, or landing automation;
- treating the historical T097 failure as a flake;
- T101 merge, T102 implementation, or any successor authority.

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
SPEC_007_TASKS_AMENDMENT_009=CLOSED_CANONICAL
T097_REFERENCE_IMAGE_REPIN_20260907_300_1=AUTHORIZED
T097_REFERENCE_IMAGE_REPIN_LANDED=NO
T101=OPEN_BLOCKED_BY_T097_REFERENCE_ENVIRONMENT
T102=BLOCKED_BY_DEPENDENCY
```
