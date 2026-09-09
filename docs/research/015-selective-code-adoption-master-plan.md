# Winds Selective Code Adoption Master Plan

**Status:** Research-only architecture and donor-adoption plan. Non-authorizing.

**Prepared:** 2026-09-08

**Canonical Winds base inspected:** `9d6300ff5566a40ba9a90247e164e0761a91fe65`

**Planning branch:** `docs/014-loopforge-skillhone-roadmap-reconciliation`

**Companion research:** `docs/research/014-loopforge-skillhone-roadmap-reconciliation.md`

## Authority and provenance boundary

The Founder has explicitly authorized Winds to copy, adapt, and otherwise reuse code from the source portfolio recorded in the repository. That project-direction authorization removes the need to re-request ordinary Founder permission for each selective reuse decision.

It does **not** remove the repository's existing provenance, license, notice, safety, evidence, and review requirements. Every copied or adapted slice must still identify the exact upstream source, preserve applicable notices, remain compatible with Winds' licensing and architecture, and prove Winds semantics with deterministic tests.

This document does not itself admit a dependency, copy donor code, amend the Constitution, alter closed Spec 007, or authorize Spec 008 implementation. The former T095/T096 dependency ladder is closed canonical history and is not a current merge gate.

---

## 1. Executive decision

Winds should aggressively reuse proven open-source implementation where that reuse reduces risk or duplicate engineering, but it should not become a collage of donor architectures.

The selection rule is:

```text
REUSE_WHERE_THE_DONOR_HAS_ALREADY_SOLVED_A_NARROW_PROBLEM_WELL
+ PRESERVE_WINDS_CANONICAL_IDENTITY_AUTHORITY_AND_EVIDENCE
+ COPY_ONLY_THE_SMALLEST_COHERENT_SLICE
+ KEEP_UPSTREAM_PROVENANCE_AND_UPDATE_STRATEGY
+ REQUIRE_WINDS_NATIVE_TESTS
= SAFE_ACCELERATION
```

The preferred order for any future capability is:

1. use an accepted standard or official protocol/SDK directly;
2. reuse an already-adopted Winds dependency;
3. copy/adapt a small proven implementation from a permissive donor when it is clearly better than rebuilding it;
4. translate donor semantics/tests into Winds-native Rust when language/runtime mismatch makes direct copying unattractive;
5. write the smallest Winds-native implementation only when the above paths are insufficient.

Large runtime transplants remain disfavored even when permission and license allow them.

---

## 2. Reuse modes

Every future donor decision must use exactly one primary reuse mode.

### `INTEGRATE_STANDARD`

Use an official protocol, executable, or SDK instead of copying its internals.

Examples:
- ACP Rust SDK;
- system Git;
- external verifiers such as ast-grep or gitleaks where useful.

### `DIRECT_DEPENDENCY`

Use a pinned library as a normal dependency.

Examples:
- `portable-pty` already follows this model;
- a future Ratatui decision may use this model if formally authorized.

### `SMALL_CODE_COPY`

Copy a narrow implementation slice when direct reuse is simpler and safer than reimplementation.

Required conditions:
- exact upstream commit and path;
- license/notice verification;
- copied-slice manifest;
- modifications documented;
- upstream update policy;
- Winds-native deterministic tests.

### `SEMANTIC_TRANSLATION`

Translate an algorithm/state-machine pattern from another language into Winds-native Rust without treating the translation as byte-for-byte source reuse.

Typical use:
- LoopForge Python workflow-state logic;
- SkillHone optimization/redaction semantics.

The source remains provenance-relevant even when implementation is independently rewritten.

### `TEST_CORPUS_ADAPTATION`

Adapt adversarial/regression scenarios rather than runtime implementation.

This is especially valuable where the donor's implementation architecture does not fit Winds but its failure cases are excellent.

### `DESIGN_ONLY`

Use product or architecture ideas only.

### `REJECT_TRANSPLANT`

Do not import the donor runtime even if individual ideas remain useful.

---

## 3. Cross-source capability map

| Capability | Primary source family | Preferred reuse mode | Winds-specific rule |
|---|---|---|---|
| Workflow stage state / resume | LoopForge + existing Spec 006 context primitives | `SEMANTIC_TRANSLATION` + `TEST_CORPUS_ADAPTATION` | Canonical Winds IDs/evidence remain authoritative; donor stage state cannot replace them |
| Artifact freshness | LoopForge | `SEMANTIC_TRANSLATION` | Bind baselines to exact hashes/candidate identity and fail stale |
| Actor/run identity | LoopForge + delegate-skills | `SEMANTIC_TRANSLATION` + test adaptation | Same display name never implies same actor/run instance |
| Delegated result envelope | delegate-skills | `SMALL_CODE_COPY` or translation after exact pin/path audit | Agent result remains agent-reported, never verifier truth |
| Worktree lifecycle | Worktrunk + agent-worktree | `SMALL_CODE_COPY` candidate for narrow Rust helpers/tests | Preserve failed/dirty workspaces; no silent merge/cleanup |
| Task dependencies / ready work | Beads | `DESIGN_ONLY` initially | Reuse existing SQLite; do not import Dolt merely for a task graph |
| Scheduler backpressure / stuck worker ideas | Gas Town | `DESIGN_ONLY` | Reject daemon/role mythology transplant |
| PTY/ConPTY | WezTerm `portable-pty` + Microsoft Terminal references | `DIRECT_DEPENDENCY` + platform reference | Existing accepted PTY dependency remains the primitive |
| Shell lifecycle telemetry | Atuin + VS Code shell integration | `SMALL_CODE_COPY` candidate for narrow scripts / `SEMANTIC_TRANSLATION` | Shell markers remain source-labelled, not `WINDS_OBSERVED` process truth |
| Agent interoperability | ACP Rust SDK + Codex/Claude ACP adapters | `INTEGRATE_STANDARD` | Prefer structured protocol over TUI scraping |
| Long-tail CLI adapters | delegate-skills + AgentAPI | `SMALL_CODE_COPY` candidate / compatibility fallback | Never supersede structured protocol where available |
| Model/provider metadata | models.dev | `INTEGRATE_STANDARD` / versioned snapshot | Catalog metadata is advisory, not local runtime truth |
| Usage/quota/cost parsing | ccusage + CodexBar | `SMALL_CODE_COPY` candidate for provider-neutral parsers/tests after path audit | Every value keeps provenance and pricing/version identity |
| Human diff/review UX | Difftastic + DeepCode + reviewdog patterns | integration/design/test adaptation | Review evidence never becomes automatic promotion authority |
| Secret scanning | gitleaks CLI | external integration | Clean scan does not prove absence of secrets |
| Strong sandbox provider | OpenSandbox | future external integration | A worktree/session/policy label is never a sandbox claim |
| Durable runtime ownership | Herdr + Winds Continuum research | `DESIGN_ONLY` first; narrow copy only after exact path audit | Do not transplant daemon/server architecture before Spec 010 threat model |
| Terminal/workbench UX | Warp product research + Herdr + Ratatui candidate | product design + future dependency | Do not copy AGPL Warp application code into current licensing model |
| Canonical context / memory | Winds Spec 006 + TencentDB Agent Memory research + Pi history research | Winds-native first | Imported memory never overwrites canonical work/evidence truth |
| Learning proposal/evaluation | SkillHone + Autoresearch + SpecBench + PaperBench + RE-Bench | semantic translation / test adaptation | Optimizer and evaluator remain independent; protected surfaces must be mechanically isolated |
| Skill evolution | SkillHone + Hermes + ACE + Voyager | semantic translation | Whole bundle gets immutable identity; active skills cannot mutate in place |
| Alternative archive | DGM | design only | Preserve challengers/failed variants; no automatic winner |
| Higher-level orchestration learning | Faraday/Replica research | deferred design only | Learning consumes verification evidence; it never creates authority |
| Code graph / retrieval | Graphify and related source-registry references | deferred integration research | Deterministic filesystem/Git search remains baseline; no premature vector/RAG dependency |

All other sources in `docs/provenance/source-registry.md` remain part of the candidate/reference portfolio. They may enter this matrix only after an exact capability, exact source path, exact license, and concrete Winds need are established.

---

## 4. Immediate high-value copy/adaptation candidates

These are planning candidates, not admitted code.

### 4.1 LoopForge workflow-state validation

Exact inspected source:
- `Tencent/LoopForge@09c765286f549624dd95434e1e6ef2249657cbeb`
- `skills/devflow/scripts/workflow_state.py`
- `skills/devflow/tests/test_regressions.py`

High-value semantics to translate into Rust:
- explicit workflow/stage status validation;
- bounded `retry_count`;
- route/stage consistency;
- unique run-instance/team identity;
- stale executor rejection;
- explicit blocked state;
- artifact file/tree digest baselines;
- atomic state replacement;
- prepare/start/finish/approve separation;
- failure-preserving resume semantics.

Do **not** copy the Python orchestration runtime wholesale. Winds already has Rust, SQLite, canonical workstream/session identity, exact candidate evidence, and stronger authority semantics.

### 4.2 LoopForge regression scenarios

Adapt the negative-test ideas directly into future Winds fixtures:
- same task/display name with stale run identity;
- executor ID reused across incompatible stages;
- missing/invalid executor topology;
- stale artifact accepted after a new prepare event;
- blocked workflow still exposing a next executable stage;
- completed workflow with incomplete stage set;
- legacy-state upgrade without silently changing authority.

These are more valuable than copying its CLI surface.

### 4.3 SkillHone redacted trajectory discipline

Exact inspected source:
- `Tencent/SkillHone@7d565839fb4dc74f9c77f09ace660e1c0484e048`
- `skills/skillhone/scripts/core/redaction.py`
- `skills/skillhone/scripts/optim.py`

Translate the useful behavior:
- redact sensitive key names recursively;
- redact known secret-value patterns;
- redact secret-bearing environment values;
- persist trajectory/history only after redaction;
- distinguish raw execution evidence from safe durable observation summaries;
- keep explicit omission markers so redaction is never confused with evidence completeness.

Winds should improve this further by making secret classes typed where practical and by testing nested/encoded/control-character cases adversarially.

### 4.4 SkillHone bounded optimization controls

Reuse semantics, not the Python runner:
- explicit maximum iterations;
- patience/no-improvement stop;
- optional budget ceiling;
- durable run ID/state;
- attributable one-change-per-cycle history;
- diagnostic observation before modification;
- failed/reverted variants retained.

These belong in future Spec 013 Verified Learning, not current Spec 007/008 runtime code.

### 4.5 delegate-skills relay/test slices

Current donor ledger already classifies this as a strong copy candidate.

Before actual reuse:
- pin exact commit;
- pin exact result-envelope/process-tree/touched-file paths;
- audit Windows behavior;
- copy only protocol-neutral result/process/test helpers that reduce bespoke code;
- preserve the rule that delegated actors do not own landing authority.

### 4.6 Rust worktree-safety donors

Worktrunk and agent-worktree are stronger candidates for literal Rust reuse than LoopForge/SkillHone because the language and problem domain align.

Potential future copy targets after exact path audit:
- direct process execution helpers that avoid shell-string composition;
- failure-preserving worktree lifecycle helpers;
- first-run/changed-hook approval test patterns;
- cross-shell/platform path handling;
- bounded command-log structures.

Do not import donor merge/promotion semantics where Winds already has stricter verification and human-selection rules.

### 4.7 Shell-integration snippets

Atuin and VS Code shell integration may justify narrow copied/adapted script fragments when shell-specific escaping/lifecycle handling is already robust upstream.

Any such copy must preserve:
- exact upstream path/commit;
- license header/notice;
- shell-specific regression fixtures;
- source classification (`SHELL_REPORTED` or equivalent);
- spoof/confusion tests, including nonce misuse.

---

## 5. New gaps found by the code-reuse pass

The earlier roadmap identified workflow/evaluation gaps. Code reuse introduces additional gaps that must be solved before donor code becomes routine.

### G1 — No copied-slice manifest

Winds needs a standard record for every copied/adapted source slice.

Required fields:
- donor repository;
- exact commit/tag;
- exact source paths;
- license and notices;
- reuse mode;
- copied/adapted destination paths;
- material modifications;
- why dependency/integration or smaller reimplementation was insufficient;
- upstream update strategy;
- Winds tests that own the resulting semantics.

### G2 — No upstream-drift policy

A copied slice can become stale or insecure even while Winds tests remain green.

Future admission must specify one of:
- frozen forever unless a defect requires update;
- periodically compared against pinned upstream releases;
- updated only with the owning Winds capability;
- replaced by standard integration when upstream protocol matures.

### G3 — No donor-boundary test requirement

Tests must prove the **Winds contract**, not merely reproduce donor tests.

Each copied slice needs:
- happy-path equivalence where useful;
- Winds authority/evidence invariants;
- malformed/adversarial inputs;
- platform-specific behavior;
- failure/recovery behavior;
- proof that donor-specific side effects are disabled if Winds does not authorize them.

### G4 — License composition needs per-slice truth

Winds' `MIT OR Apache-2.0` project license does not erase upstream terms.

Examples:
- copied MIT code must retain applicable MIT notice obligations;
- Apache-2.0 sources may require NOTICE handling;
- GPL executables such as system Git remain process boundaries rather than copied library code;
- AGPL Warp application code remains outside the current copy plan unless an explicit compatible licensing decision is made.

### G5 — No code-level provenance marker convention

Copied/adapted files should carry a concise source comment where appropriate, while `docs/provenance/donors.md` carries the authoritative full record.

Generated/translated code needs a different marker from verbatim copied code.

### G6 — No import-size ceiling

Large donor imports increase review and maintenance risk.

Default rule for future planning:
- prefer one coherent helper/module/test fixture at a time;
- any transplant that introduces a subsystem boundary, runtime, server, database, scheduler, plugin framework, or broad dependency tree requires its own explicit architecture decision.

### G7 — No semantic ownership rule after copy

Once code lands, Winds owns the behavior and tests. Upstream behavior is not automatically authoritative.

A future upstream change must not silently redefine Winds semantics.

### G8 — Event replay/idempotency remains underspecified

Resumable workflows and durable runtimes need explicit event identity and replay semantics.

Future state machines should define:
- event/run IDs;
- duplicate detection;
- idempotent transitions;
- ambiguous external side-effect classification;
- at-most-once vs at-least-once semantics where applicable.

### G9 — Deterministic clocks/IDs for tests

Donor implementations often use wall clocks/UUIDs directly. Winds needs injectable/deterministic test seams so recovery and replay tests are reproducible.

### G10 — Raw evidence vs durable safe observation

Redaction introduces two records:
- raw ephemeral/controlled evidence;
- durable redacted observation.

The product must never imply that the redacted view is complete raw evidence.

### G11 — Protected evaluation needs capability isolation, not path convention

The SkillHone audit makes this explicit.

A future holdout system must prove that the optimizer cannot read/modify:
- protected tasks;
- expected answers;
- verifier policy;
- hidden composition tests;
- evaluator credentials/configuration.

Prompt instructions and directory naming are insufficient.

### G12 — No donor-adoption benchmark

Before copying a non-trivial implementation, record whether it actually reduces:
- code size;
- defect surface;
- platform risk;
- implementation time;
- maintenance complexity.

A donor that adds more adaptation code than a small Winds-native implementation should be rejected by Ponytail.

### G13 — No rollback plan for donor slices

Each copied/adapted slice needs a reversible removal/replacement path when practical.

### G14 — No security update ownership

For code that enters a security boundary, define who/what watches upstream advisories and how a vulnerable copied version is identified.

### G15 — No source trust score should exist

Do not create a magic donor score. Source admission is capability-specific and evidence-specific.

---

## 6. Donor admission manifest

Before any copied/adapted source is accepted, add an entry to `docs/provenance/donors.md` using this conceptual record:

```text
DonorSlice
- donor_repository
- donor_commit
- donor_tag_if_any
- source_paths[]
- source_license
- notice_files[]
- reuse_mode
- destination_paths[]
- copied_verbatim: yes/no
- translated_or_modified: yes/no
- modification_summary
- capability_owned
- architecture_decision_reference
- exact_winds_candidate
- deterministic_tests[]
- platform_tests[]
- security_or_authority_tests[]
- upstream_update_policy
- rollback_or_replacement_path
```

Acceptance is invalid if the manifest is incomplete for a copied slice.

---

## 7. Revised post-Spec-007 architecture sequence

The roadmap from the companion reconciliation remains the preferred order:

```text
SPEC_007  Native Agentic Terminal UX Foundation
    ↓
SPEC_008  Resumable Workflow & Decision Ledger
    ↓
SPEC_009  Model Mesh / explicit multi-provider continuity
    ↓
SPEC_010  Winds Continuum + durable local runtime owner
    ↓
SPEC_011  Verified Browser + browser evidence
    ↓
SPEC_012  Proof-Carrying Reality Branches / candidate comparison
    ↓
SPEC_013  Verified Learning + Experiment Plane
    ↓
SPEC_014  Remote/mobile/team continuation
```

The code-adoption pass strengthens that sequence:

- Spec 008 may reuse LoopForge test/state ideas and narrow Rust donor helpers, but remains single-process.
- Spec 009 should integrate official structured protocols/SDKs before copying provider adapters.
- Spec 010 may study/copy narrow Herdr lifecycle primitives only after the durable-owner threat model is accepted.
- Spec 011 should integrate existing browser automation standards rather than invent a browser engine.
- Spec 013 may reuse SkillHone/Autoresearch evaluation mechanics only after protected-evaluation isolation is real.

---

## 8. Proposed Spec 008 implementation waves — research only

These waves are sequencing guidance, not authorized tasks.

### W0 — Donor/provenance freeze

- re-read exact post-Spec-007 `main`;
- revalidate LoopForge/SkillHone pins and any selected Rust donor pins;
- decide each selected slice's reuse mode;
- create donor manifests before code acceptance.

### W1 — Workflow and stage state

- `WorkflowRun` / `StageRun` state machine;
- deterministic IDs and transition validation;
- versioned schema;
- SQLite transactional persistence;
- no daemon/IPC.

### W2 — Artifact freshness and actor identity

- artifact baseline/digest references;
- stage-attempt identity;
- stale actor/session rejection;
- exact candidate invalidation.

### W3 — Resume / reconstruct / block semantics

- explicit proof levels;
- stage-specific context capsule;
- no-progress/retry policy;
- duplicate/replay handling;
- ambiguous external side-effect truth.

### W4 — Decision ledger

- append-only decision records;
- supersede/revert lineage;
- source/authority/candidate/evidence binding;
- rejected alternatives remain inspectable.

### W5 — Adversarial recovery

- corrupt/partial state;
- stale artifacts;
- reused identities;
- duplicate completion;
- changed candidate;
- authority escalation;
- secret-bearing output;
- replayed external action.

### W6 — Readable UX projections

- status;
- resume preview;
- why blocked;
- decision history;
- reconstruction loss report.

### W7 — Cross-platform and closeout qualification

- deterministic Linux/macOS/Windows/WSL evidence where claimed;
- migration/recovery fixtures;
- performance/soak checks for state growth;
- correctness/safety/Ponytail/independent review;
- no automatic downstream authorization.

---

## 9. What should still not be copied

Founder authorization does not make every source architecturally desirable.

Do not transplant:
- LoopForge's full Python orchestration/runtime into the Rust core;
- SkillHone's Forgejo/LiteLLM/high-permission optimization runtime;
- Gas Town's daemon-heavy hierarchy;
- a generic plugin/marketplace framework before a concrete spec;
- OpenSandbox's control plane into the local core merely to claim isolation;
- Herdr's full server/socket architecture before Spec 010;
- Warp AGPL application implementation under the current Winds licensing model;
- provider/browser credential scraping defaults;
- automatic reviewer merge/promotion behavior;
- donor task completion semantics that collapse `DONE`, `VERIFIED`, and `HUMAN_ACCEPTED`.

---

## 10. Planning completion criteria

This selective-adoption plan is ready to inform future formal specs only if a future planner can answer:

1. Which exact source solves this exact capability?
2. Is direct integration, dependency, copy, translation, test adaptation, or design-only use best?
3. What exact upstream commit/path/license applies?
4. What notices must remain?
5. What Winds invariant is stronger than the donor's default behavior?
6. What donor behavior must be deleted/disabled?
7. What tests prove Winds semantics independently?
8. What platform claims are directly exercised?
9. How will upstream drift/security updates be handled?
10. How can the donor slice be removed or replaced?
11. Does the reuse materially reduce complexity compared with a small Winds-native implementation?
12. Is the source still non-authoritative for acceptance/authority where required?

If those questions cannot be answered, the source remains `UNADMITTED` regardless of permission or license.

---

## 11. Final recommendation

Winds should use its unusually rich donor portfolio as an acceleration advantage, but the product should remain architecturally coherent.

The correct strategy is:

> **Integrate standards. Depend on mature primitives. Copy narrow proven code. Translate strong semantics. Adapt adversarial tests. Reject donor architectures that weaken Winds truth.**

LoopForge and SkillHone are especially valuable because they expose missing workflow/evaluation discipline. The wider Winds source portfolio adds mature Rust lifecycle, shell, worktree, protocol, sandbox, observability, terminal, and learning references.

The opportunity is not to copy the most code. It is to copy **the highest-leverage code and tests while keeping Winds' canonical identity, authority, evidence, and human-decision model stronger than every donor.**
