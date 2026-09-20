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

## T152 predecessor repair qualification

The predecessor state after T158 was canonical `main` merge `68fe2b1b2ccbc0a87e788ca14565c04fe9cb8cc1`. A local full-binary regression had exposed `PersistentTerminalRuntimeError::OutputGap` in the T152 detach/reattach fixture.

Focused and isolated local reruns did not reproduce that failure consistently, but the initial T159 Ubuntu campaign independently reproduced the same defect on exact candidate `5eb1fd205f31ec5f86cd0dca035ce98ae01ca45a`:

- workflow: `t159-performance`;
- run: `35489854065`;
- job: `106022819284`;
- correctness/security gate: `PASS`;
- core campaign: `FAIL`;
- error: `persistent terminal output exceeded the bounded T152 direct-output queue; the direct stream is incomplete`.

That GitHub reproduction superseded the earlier environmental/temp-path-collision hypothesis. The defect was product-real: the T152 direct-output bound was implemented as 64 queued read fragments, so sub-256-KiB output could fail solely because the PTY reader fragmented it into many small reads.

PR #244 repaired the implementation without increasing the accepted 256 KiB payload ceiling or weakening fail-closed overflow behavior. Exact repair head `90089e01003b5843c397e45aaac9585798f90b77` replaced fragment-count capacity with byte-count capacity and added a one-byte-reader regression proving that sub-ceiling output cannot become a gap solely due to read fragmentation.

PR #244 exact-head workflows all succeeded:

- `quality` — run `35490510910`;
- `release-candidate` — run `35490510900`;
- `t141-desktop-security` — run `35490510908`;
- `t142-native-platform` — run `35490510913`;
- `windows-terminal` — run `35490510911`.

PR #244 merged normally as canonical `main` `2e1e81e96046ce6027702c4b92dfe22a7a3b653e`. All actually-triggered post-merge push workflows on that exact merge also succeeded:

- `quality` — run `35490969665`;
- `windows-terminal` — run `35490969654`;
- `t141-desktop-security` — run `35490969695`;
- `t142-native-platform` — run `35490969686`.

T159 must therefore qualify only on a candidate containing canonical repair `2e1e81e96046ce6027702c4b92dfe22a7a3b653e` or a proven successor. No evidence from the failed pre-repair T159 head qualifies a repaired successor.

## Superseded T159 campaign failures and retained semantics

The repaired T159 program preserved two later failed candidates as negative evidence rather than relabelling them as qualification.

Exact candidate `a63847633d946fb3e5ecff8046958ac2f708db96` passed its persistent-runtime correctness/security gate and repository quality, then failed the Ubuntu T159 core campaign in run `35492553589`, job `106029868895`. The campaign emitted the entire >10 MiB workload as one burst before the accepted direct consumer could continuously drain the fixed 256 KiB transient direct-output queue. The intentional T152 `OutputGap` contract therefore fired. The failure did not authorize a larger queue. The successor retained the exact 256 KiB ceiling and the same >=10 MiB / >=100,000-line requirement while switching the qualification workload to 100 sustained 1,000-line batches, each drained before the next batch.

Exact candidate `8a3e3af527cc8fa0d8402cfa2c58eb3feac6adb1` passed its persistent-runtime correctness/security gate and its exact-head `quality`, `release-candidate`, `t141-desktop-security`, `t142-native-platform`, and `windows-terminal` workflows. Its T159 core campaign failed in run `35492782420`, job `106030468177`, when the deliberately undrained observer reached the already-accepted T153 slow-client boundary:

```text
persistent terminal replay failed: replay observer disconnected for slow-client backpressure
```

That result exposed a campaign-expectation defect, not a runtime defect. Spec 011 Plan AD-011-13 explicitly requires bounded per-client queues and permits deterministic slow-observer disconnect after the limit is exceeded, while requiring that a slow observer cannot stall the controller, another observer, PTY reader, or owner authority loop.

The successor campaign therefore requires exactly one accepted slow-observer disconnect, records the batch where it occurs, continues draining all seven fast observers, continues controller resize during sustained output pressure, and requires cleanup of the disconnected observer handle. The evidence assembler independently binds the final artifact to one disconnect, a recorded disconnect batch, positive fast-observer output delivery, and the unchanged 4 MiB observer queue ceiling.

These campaign corrections change no runtime implementation, queue ceiling, performance threshold, sample floor, authority boundary, correctness gate, or security gate.

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
