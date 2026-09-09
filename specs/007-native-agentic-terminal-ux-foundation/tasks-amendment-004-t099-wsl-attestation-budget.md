# Spec 007 Tasks Amendment 004 — Bounded T099 WSL Attestation Host Budget

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007 T099 authority, canonical Spec 003/T062 real Windows+WSL2 evidence requirements, the accepted T068 WSL cleanup-attestation safety boundary, and the Founder directive in the active project session to continue the authorized Winds program without fabricating evidence.

## Purpose

T099 requires the full repository regression and all applicable platform workflows to be green on one exact final candidate. The current T099 candidate exposed a repeated real Windows+WSL2 failure in the existing T062 attestation helper before the mapped/fallback production launch proof was reached.

The failure occurred twice on the same exact T099 head. In both failures, the Windows launcher process-scope cleanup was proven, but the Linux cleanup marker was not observed before the host execution phase expired. This is therefore not eligible for classification as a one-off infrastructure flake and MUST NOT be bypassed through rerun-to-green.

The existing helper is test-only and uses a 2-second Linux owned-scope watchdog with an 8-second total host timeout. The shared bounded process helper reserves up to 2 seconds of that total for cleanup, leaving only about 6 seconds for host-side observation of the WSL command lifecycle. Canonical T062 requires real distribution discovery, selected-distro launch, cwd mapping, Git identity verification, and visible mismatch behavior; it does not make the 8-second host transport budget a product requirement. The accepted safety property is bounded execution with truthful Linux and Windows cleanup evidence.

This amendment authorizes only the minimum test-only host-budget adjustment needed to make that existing attestation proof reproducible on current real Windows+WSL2 runners without weakening the Linux safety watchdog, production WSL command limits, cleanup requirements, or Spec 007 acceptance criteria.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- T098 is `CLOSED_CANONICAL` on canonical `main` or a forward descendant;
- T099 is the currently authorized Spec 007 task;
- the active T099 candidate still requires the repository's real Windows+WSL2 platform gate;
- the repeated T062 failure is still attributable to `prove_wsl_exec_scope_cleanup_for_test` failing before production mapped/fallback launch proof, with Windows process-scope cleanup proven and Linux cleanup-marker observation missing before the host phase deadline;
- no later canonical repair has already removed the need for this amendment.

At amendment creation, canonical `main` is `df0d48992f37d430fa73271fee7936e118be612a`, the guarded T098 landing. The active T099 PR is #121 at head `a9b9c7bc6ae52daa5e562bf4ff311facdd7974d0`.

T100 remains dependency-blocked until T099 closes canonically.

## Exact additional T099 authority

When this amendment is canonical and the mandatory gate above remains true, T099 may additionally modify only:

```text
src/wsl_launch.rs
```

and only inside:

```rust
#[cfg(all(windows, test))]
pub(crate) fn prove_wsl_exec_scope_cleanup_for_test(...)
```

The authorized repair is exactly:

1. for the descendant-cleanup proof call, change only the `total_timeout` argument passed to `run_wsl_exec_with_limits` from `Duration::from_secs(8)` to `Duration::from_secs(15)`;
2. for the descendant-absence verification call, change only the `total_timeout` argument from `Duration::from_secs(8)` to `Duration::from_secs(15)`.

No other behavior is authorized by this amendment unless a new material failure proves this exact repair insufficient, in which case execution MUST stop for a new explicit governance decision rather than silently widening scope.

## Values that MUST remain unchanged

This amendment does **not** authorize changing any of the following:

- the 2-second Linux owned-scope watchdog used by the two successful attestation-helper calls;
- the timeout-injection regression's 1-second Linux watchdog;
- the timeout-injection regression's 6-second total host timeout;
- production `run_wsl_exec` limits, including its 20-second Linux scope timeout and 30-second total timeout;
- `operation_deadlines`, cleanup-reserve calculation, Windows Job Object semantics, Linux cleanup-marker requirements, process ownership, or fail-closed cleanup truth;
- T062 production mapped/fallback WSL launch behavior;
- any Git, verification, evidence, acceptance, or landing authority;
- any Spec 007 FR/SC requirement or T097 performance threshold.

The widened test-only host budget MUST NOT convert an absent Linux cleanup marker into success. It only gives the existing bounded real-WSL transport/observation path more time to reach the same required marker and quiescence proof.

## Required evidence after use

If this amendment is used, candidate movement invalidates all prior T099 candidate-bound CI and review evidence. The repaired T099 candidate must be requalified from scratch under the Standard Acceptance Gate, including:

- repository `quality` SUCCESS on the exact repaired head;
- all focused T099 tests registered, executed, and green;
- `windows-terminal` SUCCESS on the same exact head, including native Windows/ConPTY and the real Windows Server + Ubuntu WSL2 T062 production proof;
- `release-candidate` SUCCESS on the same exact head;
- frozen T097 performance evidence SUCCESS where the current repository workflow applies it;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review bound to the exact repaired candidate;
- zero unresolved material findings and review threads;
- exact changed-path reconciliation proving that `src/wsl_launch.rs` changed only within the test-only helper authority granted here;
- guarded expected-head landing and applicable post-merge verification before T100 is authorized.

A successful rerun after this repair is not evidence that the earlier failures were flakes; the earlier failures remain historical evidence that the previous host observation budget was insufficient on those attempts.

## Explicit non-authorization

This amendment does not authorize:

- weakening or removing any WSL cleanup proof;
- rerun-until-green classification of the repeated pre-amendment failures;
- production WSL timeout changes;
- workflow changes;
- dependency or lockfile changes;
- schema or persistence changes;
- daemon/service/socket/RPC/IPC behavior;
- provider/model/browser/network runtime behavior;
- remote execution;
- automatic Git mutation, winner selection, merge, or landing;
- T100 implementation or closeout before T099 is canonically landed and post-merge verified.

## Acceptance of this amendment

This file is governance-only and is not canonical merely because it exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review bound to the exact candidate, zero unresolved material findings/threads, exact one-file scope reconciliation, guarded `expected_head_sha` landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing may the additional T099 test-only authority above be used.
