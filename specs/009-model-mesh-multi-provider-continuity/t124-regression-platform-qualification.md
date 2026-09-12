# T124 — Spec 003/006/007/008 Regression and Platform Qualification Candidate

Status: `IN_QUALIFICATION`

Canonical base at candidate creation:

```text
BASE=20d1ea020c79c88ed883fb956c0b4dc401dd8ca6
BASE_TREE=b05c51dc21973f05ffc40749c9554bc5b5bdd421
T123=CLOSED_CANONICAL
T124=AUTHORIZED_TO_START
```

This task is evidence-only. No focused T124 glue test is required because T114-T123 introduced no unresolved cross-spec regression seam after the T123 adversarial campaign and exact-head platform/regression qualification. This candidate changes no production code, runtime behavior, Store behavior, migration, dependency, workflow, Workbench source, provider execution path, credential path, or Git authority.

The final immutable T124 candidate SHA is intentionally not embedded in this file. Pull-request metadata, exact-head CI, exact-head review, guarded landing, and post-merge Git identity bind the final candidate without creating a self-referential follow-up commit.

## 1. Immediate canonical baseline

T123 landed through PR #181 with:

```text
T123_HEAD=533f468b9a956acf8e5a73e65229f542a26d610b
T123_TREE=b05c51dc21973f05ffc40749c9554bc5b5bdd421
T123_MERGE=20d1ea020c79c88ed883fb956c0b4dc401dd8ca6
T123_MERGE_SIGNATURE=VERIFIED_VALID
T123_UNRESOLVED_REVIEW_THREADS=0
```

Exact-head T123 qualification completed successfully on the implementation-bearing tree:

| Workflow | Run | Result | Directly exercised domain |
| --- | ---: | --- | --- |
| `quality` | 34706322484 | SUCCESS | locked format, clippy, and full Rust test suite on Ubuntu and macOS |
| `release-candidate` | 34706322502 | SUCCESS | Ubuntu/macOS quality, T064 regression, Windows authority refusal, T063 terminal soak on Ubuntu/macOS/Windows, SC-001 soak, release builds |
| `t097-performance` | 34706322491 | SUCCESS | pinned Ubuntu release qualification and frozen performance/boundedness gates |
| `windows-terminal` | 34706322494 | SUCCESS | Ubuntu/macOS terminal integration, native Windows terminal, real Windows+Ubuntu WSL2 integration |

Post-merge canonical `main` then passed:

```text
POST_MERGE_QUALITY_34707095539=SUCCESS
POST_MERGE_WINDOWS_TERMINAL_34707095529=SUCCESS
```

These runs establish the inherited baseline. They do not substitute for the fresh exact-head workflows required on this moved documentation-only T124 candidate.

## 2. Spec 003 regression boundary

Spec 003 terminal lifecycle, recovery, workspace identity, Git observation, command lifecycle, transcript retention, and fail-closed cleanup remain exercised by the repository full suite and terminal workflows. Relevant canonical regression families include T059/T060/T063/T068 and their store/terminal integration coverage.

T124 makes no change to terminal ownership, PTY/ConPTY lifecycle, process-group cleanup, workspace registration, Git mutation authority, command execution, transcript persistence, or recovery semantics.

The historical local T123 full-suite cleanup-window failures remain historical evidence rather than success evidence. An unchanged canonical-main control reproduced the same bounded-cleanup-window class. No rerun-to-green was used to rewrite that history. Exact-head GitHub CI and post-merge platform checks succeeded independently.

## 3. Spec 006 regression boundary

Spec 006 runtime discovery, runtime/session binding, native-session provenance, context transfer, authority, approval, evidence, and worker boundaries remain exercised by the T072-T085 families plus the repository release/platform workflows.

Model Mesh does not relabel runtime prose or runtime kind into provider/model identity, does not manufacture native resume, and does not expand a prior approval beyond exact target/binding/session/role/current-authority scope.

The separately governed live-runtime nonclaims remain unchanged:

```text
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Persistence/domain fixture success, Windows/WSL2 terminal qualification, and Model Mesh target matching are not real provider/model worker execution proof.

## 4. Spec 007 regression boundary

Spec 009 introduced no Workbench renderer or second Model Mesh presentation authority. T122 canonically closed with `WORKBENCH_MODEL_MESH_NECESSARY=NO`.

No Spec 007 Workbench source is changed by T124. Existing full-suite qualification nevertheless continues to exercise Workbench topology, terminal ownership, input, navigation, host safety, accessibility, platform, and adversarial boundaries including T087-T099.

No Workbench result, terminal text, display label, search match, or source-agent prose becomes provider/model identity, verification, human acceptance, routing authority, native-resume proof, or landing authority.

## 5. Spec 008 regression boundary

Spec 008 workflow/stage/actor identity, exact baselines, reconstruction, retry/recovery, decision history, projections, CLI, reviewer handoff, and adversarial boundaries remain exercised by T101-T112 and repository quality/release workflows.

Model Mesh target descriptors remain bound to canonical workspace/workstream/workflow/stage/actor/session/role scope. Candidate/artifact/evidence movement remains explicit drift/staleness rather than latest-wins truth. Reviewer projections remain non-persuasive and do not select a winner.

No Model Mesh approval substitutes for a Spec 008 workflow decision, and no workflow decision substitutes for exact Model Mesh target-selection or continuity authority.

## 6. Platform claim boundary

Only directly exercised domains may be called platform-qualified:

- Ubuntu/Linux repository and terminal paths: exercised by exact-head GitHub workflows.
- macOS repository and terminal paths: exercised by exact-head GitHub workflows.
- native Windows terminal path: exercised by the native Windows job.
- Windows host plus Ubuntu WSL2 path: exercised by the real WSL2 integration job.

No platform result proves real Claude execution, real Codex worker execution, provider/model availability, provider authentication, provider billing, provider-native memory, or physical native-resume continuity.

## 7. Frozen schema and negative scope

`migrations/0011_model_mesh_continuity.sql` remains immutable after T116. Its accepted SHA-256 remains:

```text
9130e8efd70daaa71408189c46a6e61eca9fb68e3fdada990f3d90598bd27b9d
```

T124 authorizes and introduces none of the following:

- migration or schema mutation;
- new dependency or lockfile change;
- provider API, provider SDK, billing API, pricing database, or credential handling;
- gateway, daemon, socket, RPC, IPC, browser, or remote-control path;
- policy engine, silent fallback, automatic routing, automatic winner selection, or learning subsystem;
- second Model Mesh authority or second persistence store;
- automatic Git merge, rebase, cherry-pick, push, PR creation, or landing.

## 8. T124 acceptance gate

This candidate may close T124 only after all of the following are true on its exact final head:

```text
T124_PRODUCTION_SOURCE_CHANGE=NO
T124_FOCUSED_GLUE_TEST=NOT_REQUIRED
T124_MIGRATION_0011_CHANGE=NO
T124_NEW_DEPENDENCY=NO
T124_RUNTIME_OR_PROVIDER_EXECUTION=NO
T124_CREDENTIAL_OPERATION=NO
T124_PLATFORM_CLAIMS=DIRECTLY_EXERCISED_ONLY
SPEC_006_LIVE_NONCLAIMS=UNCHANGED
EXACT_HEAD_QUALITY=REQUIRED
APPLICABLE_EXACT_HEAD_PLATFORM_REGRESSION=REQUIRED
AUTHOR_REVIEW=REQUIRED
PONYTAIL_YAGNI_REVIEW=REQUIRED
FRESH_INDEPENDENT_SUBSTANTIVE_REVIEW=REQUIRED
UNRESOLVED_MATERIAL_FINDINGS=0_REQUIRED
GUARDED_NORMAL_LANDING=REQUIRED
POST_MERGE_PUSH_VERIFICATION=REQUIRED
```

Only after those gates are proven may repository truth state:

```text
T124=CLOSED_CANONICAL
T125=AUTHORIZED
```
