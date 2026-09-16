# Spec 010 T143 — Performance and Stress Qualification

Status: CANDIDATE

## Canonical authority

- Canonical predecessor: T142A guarded merge `8d75cab455f11afddeaa6fbe6244dbb1021d1467`.
- The T142A merge tree is `0b5a601b8103aa23e8b30a6169308afdde1a9241` and all 11 workflows triggered by that merge completed successfully.
- T143 is authorized by the accepted Spec 010 task order. T144 remains blocked until T143 is `CLOSED_CANONICAL`.

## Purpose

T143 qualifies the final pre-Founder-review desktop shell against the frozen launch, interaction, render, idle-resource, large-scale, output, and resize budgets. It measures before optimizing. No performance framework, WebGL path, virtualization layer, renderer package, or runtime dependency is added by this task.

## Measurement boundary

The reference performance environment is the exact GitHub `ubuntu-24.04` runner used by the `t143-performance` workflow. Evidence records the runner image identity, exact candidate/tree, toolchain versions, WebKitGTK package/runtime version, browser host, release binary hash, renderer-dist hash, fixture identity, raw samples, and threshold disposition.

Native launch and idle-resource measurements use the release Tauri host plus its descendant WebKitGTK processes. The benchmark-only build feature adds a page-load observation hook but no Tauri command, IPC authority, dependency, persistence path, network service, or production behavior. The renderer marks a populated two-Session fixture shell ready after two animation-frame boundaries, then performs a same-origin benchmark marker navigation observed by the host hook.

Renderer interaction measurements use WebKitGTK automation through `WebKitWebDriver` and GNOME Web/Epiphany in automation mode. The browser host is a measurement harness for the same WebKitGTK engine; it does not substitute for the native Tauri cold-launch/idle measurements.

## Required campaigns

The exact-head workflow retains raw/lossless-enough results for:

- at least 20 native cold launches with p95 `<= 1500 ms`;
- at least 1000 exact Session-selection samples with p95 `<= 50 ms`;
- at least 1000 composer keystroke-to-paint samples with p95 `<= 16 ms`;
- at least 1000 single/dual layout transitions with p95 `<= 100 ms`;
- at least 1000 local right-dock tab switches with p95 `<= 50 ms`;
- at least 1000 divider resize samples under two visible representative Session work streams;
- a `>=100 Project / >=1000 Session` fixture with measured search, scroll, and focus behavior;
- 60 seconds of native Tauri/WebKitGTK idle CPU/RSS sampling with CPU `<=2%` of one logical core and renderer+host RSS `<=300 MiB`;
- inherited exact-candidate live output stress processing at least 10 MiB and 100,000 logical lines while preserving terminal lifecycle/ownership plus 1000 backend resize operations;
- deterministic hidden-output batching across multiple simulated sessions proving bytes are coalesced before renderer writes rather than scheduling one full-frame action per byte.

## Non-authority and nonclaim boundaries

The composer timing harness temporarily enables the fixture textarea in the measurement DOM only. It never submits a prompt, calls a runtime/provider, or changes canonical T139 composer availability. This is a paint-overhead measurement, not runtime-input evidence.

The large fixture exists only in `VITE_WINDS_T143_BENCHMARK=1` builds. Production builds retain the canonical/fixture bridge choice established before T143. The production Tauri command count remains 22 and the standard security/authority workflows remain authoritative.

No T143 result may be used as cross-platform performance equivalence. T142 remains the direct platform-coverage authority; T143 performance ceilings are qualified on the declared Ubuntu/WebKitGTK reference environment.

## Qualification-discovered workflow regression repair

The first exact-head T143 candidate (`d40053bcf3ccd54e4ea20d761f27c7ce13a48447`) exposed a repository-owned CI defect before qualification could close: the already-closed `t142a-winds-identity` workflow compared every later desktop PR against the current PR base/head and re-applied the historical T142A implementation allowlist. It therefore rejected T143's separately authorized workflow, Tauri benchmark feature/hook, and T143 provenance paths before running the intended T142A regression checks.

T143 repairs that stale gate without weakening the accepted T142A boundary. The workflow now re-verifies the immutable canonical T142A base/head/merge parentage, tree identity, and original path allowlist, then runs the Current Spectrum deterministic regression gates against the current candidate. It no longer treats T142A's historical implementation scope as the authority model for later canonically authorized tasks. The failed first run remains historical evidence and is not rerun-to-green on the stale candidate.

## Independent-review repair

The first independent exact-head review of successor `6e4744afae775a63eea091f196d9f6d74b62b2d8` found one material harness gap: the 60-second idle CPU/RSS campaign recorded process counts and names but did not require a WebKit renderer to remain present, so a host-only remainder could theoretically satisfy the frozen `renderer+host` resource ceilings. That reviewed head is therefore not qualification authority.

The first live campaign after that repair, on successor `6fb4a1d75a2410e1a3ecf05ddf53866f5049ca48`, failed the unchanged `renderer+host idle RSS <=300 MiB` gate with an aggregate process-tree RSS maximum of `420.891 MiB`; launch, idle CPU, renderer-presence, and process-count checks passed. That failure is retained as real diagnostic evidence and is not rerun-to-green. The next successor adds per-process RSS attribution only; it does not change the RSS scope, threshold, sample duration, or pass/fail calculation.

The attribution successor `a31cf38dd8dcb5a8addf8ecfbcf74ae3ea481ac2` also failed the unchanged aggregate RSS gate (`425.402 MiB`) while proving the peak split was host `161.359 MiB`, WebKit web renderer `204.629 MiB`, and WebKit network process `59.414 MiB`; launch p95 (`1375.2275 ms`), idle CPU (`0.51548%`), renderer-presence, and process-count gates passed. The measured renderer pressure justifies a forward-only product optimization: keep the canonically accepted Terminal sub-surface selected in workbench mode, but defer loading/allocating xterm.js until the user explicitly starts a Rust-owned terminal. Terminal selection, Rust lifecycle/authority, command count, security behavior, and the frozen performance thresholds remain unchanged.

The forward-only successor requires every retained idle sample to include at least one descendant process whose Linux `comm` contains `WebKit`, requires at least two processes (host plus descendant) in every idle sample, records per-sample names and renderer-presence truth, and includes both predicates in the native checks propagated into combined `all_checks_pass`. This closes the review finding without changing the product command surface or performance ceilings.

## Closure discipline

T143 may close only after the complete exact-head candidate passes deterministic gates, the T143 performance workflow, all other applicable workflows, author correctness/safety review, Ponytail/YAGNI review, fresh independent substantive review with zero material findings, zero unresolved review threads, guarded expected-head merge, verified merge parentage/tree/signature, and every workflow actually triggered by the merge commit.

T143 closure authorizes T144 only. It does not create or substitute the human Founder visual acceptance required by T144.
