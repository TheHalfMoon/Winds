# Spec 010 Tasks Amendment 003 — T143 Evidence-Calibrated Linux RSS Budget

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 010 FR-096 and FR-099, canonical Plan §19 performance-budget discipline, canonical T143 authority, the accepted Spec 007 performance-governance amendment precedents, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

T143 freezes a `renderer+host idle RSS <= 300 MiB` budget on the pinned Ubuntu/WebKitGTK reference environment. Repeated exact-head campaigns have preserved that metric and failed it while launch, CPU, renderer-presence, process-count, authority, security, and deterministic gates remained intact. The latest diagnostic-only candidate `e699df609dc98ec6c058d611efc9a0fee810d7d1` adds same-host static and minimal React baselines without changing production behavior.

Its first-attempt evidence shows that the selected Tauri + WebKitGTK + React 19 runtime floor consumes effectively the entire 300 MiB ceiling before the required Winds application anatomy is rendered. Continuing random allocator or dormant-code experiments would no longer be evidence-led. This amendment changes one engineering requirement only: the Linux reference `renderer+host idle RSS` ceiling becomes `<= 320 MiB` under the exact existing measurement boundary.

This is an explicit governance change, not a reinterpretation of the old threshold. Every pre-amendment result above 300 MiB remains a historical failure and MUST NOT be relabelled as a pass.

## Material evidence

The exact `e699df609dc98ec6c058d611efc9a0fee810d7d1` first-attempt workflow run `35091146899` produced artifact `t143-performance-e699df609dc98ec6c058d611efc9a0fee810d7d1` (artifact id `10443903924`, archive digest `sha256:956a761bf34ecb11f7b25a2febb57009e005d9eaefbbc71471a7c5f76777d1`). That artifact records 239 retained idle samples for each same-host campaign using the same Tauri host, WebKitGTK/Xvfb reference environment, descendant enumeration, two-second settle period, and 60-second measurement window:

```text
STATIC_PLATFORM_BASELINE_MAX=280.688 MiB
STATIC_PLATFORM_BASELINE_P95=280.688 MiB

MINIMAL_REACT_TAURI_BASELINE_MAX=298.543 MiB
MINIMAL_REACT_TAURI_BASELINE_P95=298.520 MiB
REACT_RUNTIME_DELTA_OVER_STATIC=17.855 MiB
HEADROOM_UNDER_300_AFTER_MINIMAL_REACT=1.457 MiB

WINDS_PRODUCT_MAX=316.363 MiB
WINDS_PRODUCT_P95=316.348 MiB
WINDS_PRODUCT_DELTA_OVER_MINIMAL_REACT=17.820 MiB
WINDS_PRODUCT_OVER_300=16.363 MiB
```

The same product run passed cold launch at `1055.5199 ms p95`, idle CPU at `0.54797%` of one logical core, renderer presence in every retained sample, and the minimum host+renderer process-count integrity check. Its sole native frozen-budget failure was RSS.

The retained product-memory profile has also repeatedly measured in the same narrow band after the material `Malloc=1` improvement: `316.523`, `315.191`, `314.137`, `315.723`, `315.414`, `315.590`, `315.930`, and `316.363 MiB` across forward-only candidates. Rejected configurations that intentionally changed renderer/allocator behavior and materially regressed CPU/RSS are not used to calibrate the replacement ceiling.

## Why the existing 300 MiB ceiling is no longer evidence-calibrated

The minimal React/Tauri baseline is not a Winds application optimization target: it contains only the already-selected React 19 / ReactDOM runtime, the existing Tauri window API, one minimal element, and the qualification readiness signal. At `298.543 MiB` maximum it leaves only `1.457 MiB` beneath the frozen ceiling for all required Winds Project/Session, dual-workbench, terminal, dock, accessibility, identity, and trust presentation code.

T143 already exercised multiple bounded forward-only hypotheses before this amendment: xterm allocation deferral, JavaScriptCore JIT disablement, compositing mode, WebKit cache/feature toggles, libpas scavenging, glibc arena/trim/tcache variants, and dormant renderer/code splitting. Materially beneficial settings were retained; falsified or unsafe settings were removed. The latest code-splitting candidate improved the product maximum by only about `0.56 MiB` and introduced a review finding, so it was reverted rather than accumulated.

The evidence therefore supports changing the engineering ceiling rather than weakening correctness, security, accessibility, authority boundaries, measurement scope, or the selected first-program stack merely to preserve an uncalibrated round number.

## Exact amended T143 budget

After this amendment is `CLOSED_CANONICAL`, T143 may replace exactly:

```text
renderer+host idle RSS <= 300 MiB excluding child agents/terminals
```

with:

```text
renderer+host idle RSS <= 320 MiB excluding child agents/terminals
```

`320 MiB` is the smallest five-MiB-rounded ceiling above every retained-profile same-boundary product result listed above. It provides `3.477 MiB` margin above the highest retained-profile observation (`316.523 MiB`) and `21.457 MiB` above the measured minimal React/Tauri baseline. It is a bounded first-program ceiling, not a target for future growth.

## Measurement semantics that MUST remain unchanged

The amendment does not authorize changing any of the following:

- reference runner label `ubuntu-24.04` or the exact observed image/WebKitGTK provenance captured by the workflow;
- release build profile or product binary identity binding;
- literal RSS accounting for the Tauri host plus WebKit rendering descendants;
- exclusion of the WebKit network process from the frozen renderer+host gate while retaining it as diagnostic evidence;
- exclusion of child agents/terminals only as already specified by T143;
- 60-second idle measurement duration or two-second settle period;
- all-thread `/proc/<pid>/task/*/children` descendant enumeration;
- renderer-presence and minimum-process-count integrity predicates;
- raw/lossless-enough sample retention;
- cold launch `<=1500 ms p95` and `>=20` launch samples;
- Project/Session selection `<=50 ms p95`;
- single/dual layout `<=100 ms p95`;
- composer keystroke-to-paint `<=16 ms p95`;
- cached right-dock switch `<=50 ms p95`;
- idle CPU `<=2%` of one logical core;
- large Project/Session fixture, visible/hidden output, huge-output, or resize stress floors;
- correctness, stale-binding, provenance, accessibility, security, runtime-authority, or platform requirements.

No PSS/USS substitution, reduced sample window, post-hoc process exclusion, tolerated failure, retry-to-green, percentile change, or dynamic threshold is authorized.

## Exact additional authority after canonical landing

Once this amendment has guarded-landed and all applicable post-merge verification is green, the active T143 branch may make the minimum forward-only changes required to enforce the amended ceiling in the existing T143 qualification surface:

```text
.github/workflows/t143-performance.yml
 desktop/tests/performance/t143_native.py
 desktop/tests/performance/t143_assemble.py
 desktop/tests/t143-performance.test.mjs
 docs/provenance/010-t143-performance.md
```

Only the threshold contract, its exact assertions/labels, and provenance reconciliation are newly authorized by this amendment. Existing diagnostic static/React baselines may remain because they preserve the evidence that justified the governance change, but they MUST NOT substitute for or enter product `all_checks_pass` except as non-gating diagnostics.

The T143 candidate must forward-integrate then-current canonical `main` without rebase or history rewrite. Candidate movement invalidates every prior CI, performance, and review result.

## Required evidence after use

The first post-amendment T143 exact head must qualify from scratch and must:

1. record the canonical amendment merge as an ancestor of the exact candidate;
2. run the complete T143 campaign on attempt 1;
3. satisfy `renderer+host idle RSS <=320 MiB` under the unchanged boundary;
4. satisfy every other frozen T143 latency, CPU, scale, output, resize, process-integrity, and provenance gate unchanged;
5. complete inherited huge-output and resize stress rather than stopping after native measurement;
6. pass every applicable repository, desktop, platform, security, and accessibility workflow;
7. receive author correctness/safety/evidence-integrity review and Ponytail/YAGNI review;
8. receive fresh independent substantive exact-head review with zero material findings;
9. have zero unresolved review threads;
10. use guarded expected-head normal merge and verify merge tree, ordered parents, GitHub signature, and every actually-triggered post-merge workflow.

A first-attempt post-amendment RSS result above 320 MiB is a real failure. This amendment does not authorize another threshold increase, automatic retry, or attribution to hosted-runner noise; a new material failure would require a new explicit governance decision.

## Explicit non-authorization

This amendment does NOT authorize:

- relabelling any pre-amendment `>300 MiB` result as PASS;
- changing the selected Tauri/React stack merely to manufacture a lower baseline;
- weakening any non-RSS T143 threshold or sample floor;
- changing security, accessibility, correctness, authority, persistence, runtime/provider, terminal ownership, Git, or landing semantics;
- adding a dependency, framework, daemon, public IPC/RPC/server, browser runtime, generic plugin system, remote control, automatic routing, or automatic Git landing;
- skipping T144 human Founder visual acceptance;
- beginning T145 before T144 closes canonically;
- claiming cross-platform RSS equivalence from the Linux reference result.

## Amendment acceptance gate

This governance-only amendment is not canonical merely because it exists. Its exact final candidate must satisfy:

- changed scope exactly this one amendment document;
- repository `quality` SUCCESS on the exact head;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive exact-head review with zero material findings;
- zero unresolved review threads;
- exact current main/base/head/tree/scope/ruleset/mergeability reconciliation;
- guarded normal merge with exact expected head;
- merge tree/ordered parents/GitHub signature verification;
- every actually-triggered applicable post-merge push workflow SUCCESS.

Only after those gates may repository truth state:

```text
SPEC_010_TASKS_AMENDMENT_003=CLOSED_CANONICAL
T143_RENDERER_HOST_IDLE_RSS_BUDGET_MIB=320
T143_RSS_MEASUREMENT_SEMANTICS=UNCHANGED
T143_POST_AMENDMENT_REQUALIFICATION=AUTHORIZED
T144=BLOCKED_UNTIL_T143_CLOSED_CANONICAL
```
