# Spec 011 T159 — Persistent Runtime Resource and Performance Qualification

## Purpose

T159 proves that the accepted persistent-runtime design remains bounded and responsive under idle residency, sustained terminal output, observer scale, runtime scale, and reconnect churn. This document is the retained qualification contract and evidence ledger for the exact candidate that closes T159.

T159 does not authorize threshold relaxation, correctness-gate removal, security-gate removal, provider execution, remote control, transcript persistence, or a new terminal backend.

## Frozen qualification thresholds

The candidate must preserve every frozen Spec 011 Plan/Tasks threshold:

- idle owner CPU with zero live runtimes: `<= 1% of one logical core p95`;
- idle owner RSS with zero live runtimes: `<= 64 MiB`;
- local reattach plus runtime snapshot: `<= 100 ms p95`;
- cached observer attach: `<= 100 ms p95`;
- controller input dispatch overhead: `<= 10 ms p95`, excluding child processing;
- resize dispatch overhead: `<= 25 ms p95`, excluding child processing;
- output campaign: `>= 10 MiB` and `>= 100,000` logical lines;
- concurrent observers to one runtime: `>= 8`;
- idle runtime namespaces: `>= 32`;
- deterministic attach/detach reconnect churn: `>= 500` cycles;
- replay and observer queue bounds remain the exact T153 values;
- no correctness or security check may be disabled for performance.

## Measurement boundaries

The exact-candidate `t159-performance` workflow runs on `ubuntu-24.04` with Rust `1.97.1` and a release-profile Winds binary.

The core Rust campaign measures the accepted persistent owner/runtime implementation directly. It records raw microsecond samples for reattach/snapshot, cached observer attach, controller input dispatch, and resize dispatch. The same campaign exercises a real `/bin/sh` PTY through the persistent owner, drains at least 10 MiB / 100,000 logical lines, keeps eight observer handles attached, deliberately leaves one observer slow while draining peers, creates 32 live idle runtime namespaces, executes 500 exact-generation reattach cycles, and checks process file-descriptor cleanup.

The idle-owner campaign measures a separate `target/release/winds __winds-internal-owner-v1` process with a private `XDG_RUNTIME_DIR`, zero clients, and zero live runtimes. It retains 250 ms raw samples for 60 seconds and gates CPU p95, RSS maximum, file-descriptor stability, and owner-home storage growth.

The workflow assembles `t159-evidence/combined.json` with exact commit/tree identity, release binary SHA-256, runner image identity, Rust identity, CPU/memory description, raw campaign measurements, and explicit non-relaxation flags. The raw logs and combined evidence are uploaded as an exact-head artifact.

## T152 predecessor qualification note

Canonical predecessor `main` was verified at merge commit `68fe2b1b2ccbc0a87e788ca14565c04fe9cb8cc1`, with T158 PR #242 merged from exact head `d6b03673647e389a63eec9f63d4cd710c184595c`.

Post-merge push workflows on that exact merge commit all completed successfully:

- `quality` — run `35487628367`;
- `windows-terminal` — run `35487628361`;
- `t141-desktop-security` — run `35487628377`;
- `t142-native-platform` — run `35487628432`.

A previously observed local T152 direct-output `OutputGap` was investigated before T159 work began. From a fresh exact-main clone, the focused detach/reattach test passed 20/20 serialized runs, a 12-worker campaign with isolated temporary namespaces passed 360/360 runs, and four independent full binary regressions each completed with `782 passed / 0 failed / 5 ignored`. A separate cross-process stress attempt reused colliding temporary runtime paths and therefore produced `LiveEndpointCollision` failures plus one `OutputGap`; rerunning with isolated temporary roots removed both failure classes. The direct-output overflow remains fail-closed by design and no queue bound or assertion was relaxed.

Those local runs are diagnostic predecessor evidence only. T159 closure authority comes from the exact-head GitHub workflow artifact and required reviews for the T159 candidate.

## Closure rule

T159 is not closed by this document, by a local run, or by a green unrelated workflow. Closure requires all of the following on one exact candidate:

1. exact-head `t159-performance` success with `all_checks_pass=true`;
2. exact-head repository quality and every other actually-triggered required workflow green;
3. exact changed-file and review accounting with zero unresolved material findings;
4. no threshold, sample floor, security boundary, or correctness gate weakened;
5. exact base/head/tree/ruleset/mergeability reconciliation immediately before landing;
6. guarded normal merge without force-push, rebase, or history rewrite;
7. merge tree/ordered-parent/signature verification;
8. every actually-triggered post-merge push workflow succeeds.

Only canonical T159 closure authorizes T160.
