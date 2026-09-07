# Spec 007 Tasks Amendment 003 — Bounded T097 Performance Surface

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, the Founder directive in the active project session to continue the authorized Winds program, canonical Spec 007 FR-045..FR-053, canonical T097 performance authority, and the accepted Spec 007 amendment precedent used for T087/T091 bounded evidence-generation gaps.

## Purpose

Canonical T097 requires release-like, reproducible evidence for FR-045..FR-053. FR-045 specifically measures cold/input-ready startup from Winds process start to an input-ready workbench with the first pane established, while FR-053 requires exact candidate/tree/environment/method identity.

Live canonical `main` after T096 closes does not expose a production `workbench` CLI entry, and no current workflow is authorized to execute and retain the complete T097 release-profile benchmark record on the exact candidate. Measuring only a unit-test constructor would not truthfully satisfy the process-start boundary, and reusing debug `quality` timing would violate the release-like build-profile requirement.

This amendment authorizes only the minimum product entry and read-only CI evidence surface required to measure the already-canonical T097 budgets without relaxing them. It does not broaden Spec 007 authority, introduce a dependency, or authorize successor work.

## Mandatory predecessor gate

This amendment is inert unless live canonical repository truth proves `T096=CLOSED_CANONICAL`, including:

- canonical `main` at the guarded T096 merge or a forward descendant;
- verified T096 landed tree/parents;
- successful applicable post-merge `quality` and `windows-terminal` push verification on the landed T096 merge;
- no unresolved material T096 review finding represented as closed without evidence.

At amendment creation, the observed T096 canonical landing is `fc5d15c60115b302674de061f72119bc315f0320`, tree `8704772323670365782cab92297a54d1c796188b`, with post-merge `quality` run `34170292213` and `windows-terminal` run `34170292209` both successful. This recorded observation must be reverified before the amendment is used.

T098–T100 remain dependency-blocked until T097 closes canonically.

## Exact additional T097 authority

When this amendment is canonical and the mandatory predecessor gate remains satisfied, T097 additionally authorizes only these surfaces.

### 1. Exact production workbench entry

T097 may add the smallest production command entry necessary to start the already-implemented Spec 007 workbench from the Winds binary:

```text
winds workbench [--repo <path>]
```

The command may only compose already accepted Spec 007 seams: canonical/local repository path resolution, existing native shell-profile discovery, one Winds-owned initial pane, existing PTY/ConPTY ownership, existing shell editor, existing synchronous Crossterm/Ratatui host event loop, existing bounded terminal parser/transcript state, and existing read-only workbench context/presentation state.

It MUST NOT add a provider/model route, network call, browser action, daemon/service, socket/RPC/IPC, persistent live-child ownership, remote execution, semantic search, new Git mutation, automatic verification/acceptance, automatic landing, or a new dependency.

T097 may make the minimum existing-workbench-module repairs required for the command to become genuinely input-ready and to preserve already accepted lifecycle/safety semantics. Such repairs remain subject to the original T097 requirement that measured failures trigger repair and requalification rather than threshold relaxation.

### 2. Exact benchmark readiness probe

For reproducible FR-045 measurement only, the `workbench` entry may expose one bounded diagnostic switch:

```text
--t097-exit-after-ready
```

The switch is valid only when `WINDS_T097_BENCHMARK=1` is present. Otherwise the command MUST reject it. Under the authorized benchmark environment, the command must execute the same production workbench initialization path through first-pane ownership and first render/input-ready establishment, emit one machine-readable readiness record, close any owned terminal through the accepted bounded cleanup path, restore host-terminal state if entered, and exit.

The readiness probe MUST NOT bypass shell discovery, pane creation, terminal ownership, parser/editor initialization, or the first workbench render that defines the measured ready state. It MUST NOT contact a provider, model, browser, or network service.

The probe is evidence instrumentation, not a second workbench implementation and not an alternate authority path.

### 3. Read-only exact-head performance workflow

T097 may add one workflow:

```text
.github/workflows/t097-performance.yml
```

The workflow is authorized only to:

1. run on the T097 pull-request candidate and by explicit `workflow_dispatch` with an exact candidate SHA;
2. use `permissions: contents: read` and checkout with `persist-credentials: false`;
3. verify the exact checked-out commit before measurements;
4. install repository-pinned Rust `1.97.1` using the same pinned toolchain action already accepted by repository workflows;
5. build/run the T097 harness in an explicit release-like profile with `--locked`;
6. run only local deterministic fixtures with provider/model/network behavior prohibited during benchmark execution;
7. collect the canonical T097 evidence fields and raw samples or a machine-readable lossless/adequate summary required by the canonical Plan/Tasks;
8. write machine-readable evidence under `specs/007-native-agentic-terminal-ux-foundation/evidence/` in the job workspace and print the record to the job log;
9. upload the generated evidence using the already repository-qualified pinned `actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` action;
10. fail closed if any FR-045..FR-053 threshold or identity/environment prerequisite is not proven.

The workflow MUST NOT commit, push, create or mutate a PR, tag, release, merge, rebase, cherry-pick, change repository settings, use write-capable repository permissions, or perform any product Git mutation.

The workflow may use the GitHub-hosted Ubuntu reference environment for the generic frozen performance budgets. It may add native-platform jobs only where a T097 performance claim is actually made for that platform. It MUST NOT infer platform parity from one runner.

## Reference-environment and measurement discipline

This amendment does not alter the canonical T097 evidence schema or thresholds. Each qualifying record must include at minimum:

```text
candidate_commit
candidate_tree
os
os_version_or_image
arch
rust_version
build_profile
cpu_description_or_runner_class
logical_cpu_count
memory
fixture_shell
fixture_id
measurement_command
warmup_policy
sample_count
raw_samples_or_machine_readable_summary
p50
p95
max
```

FR-045 must use at least 20 process launches of the release-built `winds workbench --t097-exit-after-ready` path and measure from process launch to the readiness record produced only after the accepted first-pane/input-ready boundary.

FR-046..FR-052 must measure the already accepted product seams, not synthetic no-op substitutes that omit the relevant workbench operation. Inert panes remain permitted only where the canonical FR/Task explicitly allows them.

The T097 workflow/harness must identify the exact measured commit/tree at runtime. Any candidate movement invalidates prior benchmark evidence and requires fresh measurement on the new exact candidate.

## Authorized changed-path reconciliation for final T097

In addition to the original canonical T097 paths, a final T097 candidate may contain only the minimum of:

- `.github/workflows/t097-performance.yml`;
- `src/main.rs` for the exact `workbench` entry and focused-test registration permitted here;
- existing `src/workbench*.rs` modules only where required to compose the accepted production entry or repair a measured T097 defect;
- deterministic T097 harness modules under `src/` or a minimal dependency-free `benches/` surface;
- `specs/007-native-agentic-terminal-ux-foundation/evidence/` T097 evidence-format/fixture material that does not fabricate candidate-bound measured values.

No other path gains authority from this amendment.

## Explicit non-authorization

This amendment does not authorize:

- any FR-045..FR-053 threshold relaxation;
- a benchmark crate or any other new dependency;
- Tokio/async runtime;
- a second PTY, parser, editor, renderer, or search engine;
- provider/model/network benchmark fixtures;
- browser/CDP/clipboard automation;
- daemon/service/socket/RPC/IPC or durable background ownership;
- remote execution;
- semantic/vector/RAG memory;
- plugin/ACP/MCP/A2A/runtime framework;
- schema migration or a new persistence table;
- automatic Git mutation, winner selection, merge, or landing;
- treating workflow success, terminal text, or benchmark prose as canonical acceptance without the Standard Acceptance Gate;
- T098, T099, T100, Spec 008, or any later roadmap implementation.

## Acceptance of this amendment

This amendment is not canonical merely because this file exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review bound to the exact candidate, zero unresolved material findings/threads, exact one-file scope reconciliation, guarded `expected_head_sha` landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing, and only while the mandatory T096 predecessor gate remains satisfied by live repository truth, may the additional T097 performance surfaces above be used.