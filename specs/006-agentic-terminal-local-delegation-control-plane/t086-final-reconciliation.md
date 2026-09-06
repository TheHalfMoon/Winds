# T086 — Spec 006 Final Reconciliation Candidate

Status: `IN_QUALIFICATION`

Canonical base at candidate creation:

```text
BASE=38809cad791e14784602f871b459ffb864f73b7a
BASE_TREE=d05c7bef14065aba414fbd667c288ff3feeeca21
T085=CLOSED_CANONICAL_IMPLEMENTATION
T086=AUTHORIZED_TO_START
```

This artifact is the focused Spec 006 acceptance/evidence reconciliation authorized by T086. It does not execute an Agent, send a prompt, install or alter a runtime, authenticate a provider, create a remote route, expand authority, or authorize a later Agentic phase.

The final immutable closeout SHA is intentionally not embedded in this file. As with the repository's Spec 003 T069 precedent, PR metadata, exact-head CI, review evidence, the guarded merge, and post-merge Git identity bind the final candidate without creating a self-referential follow-up commit.

## 1. Canonical implementation chain

The first implementation program reconciles to these canonical merged units:

| Task | Canonical PR | Canonical merge |
| --- | ---: | --- |
| T070 | #70 | `372e8d4de686f55f156c70665e7d022bff95a57c` |
| T071 | #72 | `3924dd9114952855d3437d29d326b88a60777f90` |
| T072 | #73 | `1404a580ff1168387e0ed61c2644b7508bb399aa` |
| T073 | #74 | `54fb578e483a0f40d2232667f2be5e5f945d0df8` |
| T074 | #75 | `ad9d5b223f55e78d6db73c69bb5ca076a1da3ea1` |
| T075 | #76 | `979d130a7d60bcc06069df8eb38626c0600ed170` |
| T076 | #77 | `0f071c85e401e7253991633906bbab7991155c9d` |
| T077 | #78 | `21ac0e4cc3985deb49d5ac3c5075238b6abcbfc3` |
| T078 | #79 | `06e515471cf91a0f1d5b257d6e9820096d9a0197` |
| T079 implementation lane | #80 | `9c81711dc84ca9b478c671875dea2add03bf90d1` |
| T080 implementation lane | #93 | `95c1a2b7342f5fbe970e4024889ff5254be141bd` |
| T081 | #94 | `ac0eec916d1f8c14b1aeac03a270d5ecfe1293eb` |
| T082 implementation lane | #95 | `b7d2ce1f33a1f2ae1d3a2b4fa186b0f89a083edf` |
| T083 | #96 | `b63fac2907cf9332078ceb75e0013957e5518de8` |
| T084 | #98 | `926cc454004c4fd1480b8b656d269f05cf0237f5` |
| T085 | #99 | `38809cad791e14784602f871b459ffb864f73b7a` |

The T079/T080/T082 rows deliberately distinguish the canonical implementation lane from deferred physical-runtime evidence. Landing deterministic implementation does not convert deferred live evidence into PASS.

## 2. Live-runtime truth

Current acceptance truth remains bounded to evidence actually obtained:

```text
T079_LIVE_EVIDENCE_LANE=OPEN_DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
CURRENT_HEAD_LIVE_ATTEMPT_AUTHORIZATION=ABSENT

T080_LIVE_EVIDENCE_LANE=OPEN_DEFERRED_EXTERNAL
T080_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO

T082_WORKER_LIVE_EVIDENCE_LANE=OPEN_DEFERRED_EXTERNAL
T082_WORKER_LIVE_PASS=NO
REAL_CODEX_WORKER_EXECUTION=NO
```

Historical Founder one-shot authorizations for T079 are not reusable after candidate movement. Historical bounded T079 attempts remain historical and do not establish current live PASS. This closeout candidate does not create or modify `CODEX_HOME`, credentials, runtime installation, system configuration, remote execution, or provider access solely to manufacture evidence.

Under canonical Tasks Amendment 001, deferred physical-runtime evidence does not block closure of the repository implementation program. It does block any claim that the corresponding live capability is accepted.

## 3. T070–T085 reconciliation

The dependency chain has landed in canonical order. Each successor began only after the predecessor's canonical landing/post-merge transition authorized it. Historical head-bound CI/review evidence remains historical and is not promoted across candidate movement.

T085's exact final head `c8d00e94b2ea0f56cc8c244b00e75db39870d6cf` passed repository `quality`, `windows-terminal`, and `release-candidate`; received exact-head author correctness/safety/evidence-integrity and Ponytail/YAGNI reviews; received independent CodeRabbit review covering the exact final candidate with no actionable comments and no current merge-blocking risk; and had zero unresolved material review threads before landing. Its canonical merge is `38809cad791e14784602f871b459ffb864f73b7a`, whose tree is exactly the accepted candidate tree `d05c7bef14065aba414fbd667c288ff3feeeca21`. Post-merge `quality` and `windows-terminal` push workflows succeeded on that canonical merge commit.

## 4. Functional-requirement reconciliation

Status vocabulary:

- `PROVEN_DETERMINISTIC`: accepted canonical implementation plus deterministic tests/evidence prove the first-program contract. This does not imply a separate physical-runtime claim.
- `PROVEN_GOVERNANCE_BOUNDARY`: canonical implementation/governance proves the requirement by preserving an explicit boundary or non-goal.
- `LIVE_ACCEPTANCE_DEFERRED`: the deterministic implementation contract exists, but the separately governed physical-runtime acceptance remains unproven and MUST NOT be represented as PASS.

| Requirement | Status | Canonical evidence / disposition |
| --- | --- | --- |
| FR-001 | `PROVEN_DETERMINISTIC` | T070 stable workspace identity is separate from display text. |
| FR-002 | `PROVEN_DETERMINISTIC` | T070 supports multiple stable Winds sessions per workspace. |
| FR-003 | `PROVEN_DETERMINISTIC` | T070/T071 preserve workstream identity separately from Winds/native/display identities. |
| FR-004 | `PROVEN_DETERMINISTIC` | T071 new-session semantics preserve the existing workstream. |
| FR-005 | `PROVEN_DETERMINISTIC` | T071/T081 preserve canonical work across runtime/session change. |
| FR-006 | `PROVEN_DETERMINISTIC` | T070/T071 rename fixtures preserve identity, links, and origin state. |
| FR-007 | `PROVEN_DETERMINISTIC` | T071 exposes explicit continue/fork/new-task semantics in the accepted seam. |
| FR-008 | `PROVEN_DETERMINISTIC` | T071/T084 return explicit deterministic candidates for ambiguity. |
| FR-009 | `PROVEN_DETERMINISTIC` | T073/T078 preserve explicit LIVE/RESUMED/RECONSTRUCTED/OWNERSHIP_LOST/STOPPED truth where applicable. |
| FR-010 | `PROVEN_DETERMINISTIC` | T073 refuses durable native/PID identity as proof of LIVE ownership. |
| FR-011 | `PROVEN_DETERMINISTIC`; `LIVE_ACCEPTANCE_DEFERRED` where vendor execution is required | T073/T078 require exact revalidated runtime/native binding before resume construction; no live vendor-resume PASS is inferred. |
| FR-012 | `PROVEN_DETERMINISTIC` | T078 labels newly constructed native sessions `RECONSTRUCTED`, not `RESUMED`. |
| FR-013 | `PROVEN_DETERMINISTIC` | T073 proves PID/native identifiers alone cannot recreate ownership/resume truth. |
| FR-014 | `PROVEN_DETERMINISTIC` | T071/T074 preserve canonical work/context independently of native session ID. |
| FR-015 | `PROVEN_DETERMINISTIC` | T074/T081 transfer reports distinguish transferred, omitted, derived, and unavailable state. |
| FR-016 | `PROVEN_DETERMINISTIC` | T072 models runtime/harness separately from model/provider identity. |
| FR-017 | `PROVEN_DETERMINISTIC` | T072/T077/T078 implement only the first concrete Codex/Claude targets, without a generic plugin platform. |
| FR-018 | `PROVEN_DETERMINISTIC` | T072 discovery fixtures prove no install/update/auth/terms/prompt/model call. |
| FR-019 | `PROVEN_DETERMINISTIC` | T072 preserves catalog/vendor/local-observation/unavailable capability provenance. |
| FR-020 | `PROVEN_DETERMINISTIC`; `LIVE_ACCEPTANCE_DEFERRED` for physical proof lanes | T072/T073/T080 require launch-significant executable/version revalidation before accepted use. |
| FR-021 | `PROVEN_DETERMINISTIC` | T072 keeps unsafe-to-observe auth readiness unknown/unavailable. |
| FR-022 | `PROVEN_DETERMINISTIC` within first-program vendor-native scope | T077/T078/T079/T080 use concrete structured vendor-native paths; ACP remains deliberately unlanded. |
| FR-023 | `PROVEN_GOVERNANCE_BOUNDARY` | ACP v1/schema-v1.20.0/SDK 2.0.0 planning pin remains canonical; no ACP crate was admitted. |
| FR-024 | `PROVEN_GOVERNANCE_BOUNDARY` | ACP draft v2 remains disabled/unstarted. |
| FR-025 | `PROVEN_GOVERNANCE_BOUNDARY` | MCP-over-ACP and MCP runtime remain unauthorized/unstarted. |
| FR-026 | `PROVEN_DETERMINISTIC` | T075/T084 preserve root/path visibility as non-authoritative and non-sandbox truth. |
| FR-027 | `PROVEN_DETERMINISTIC` | T075/T077 keep permission/approval requests subject to external Winds/human policy. |
| FR-028 | `PROVEN_GOVERNANCE_BOUNDARY` | No public runtime protocol, listener, HTTP/SSE/WebSocket control plane, or remote execution transport was added. |
| FR-029 | `PROVEN_DETERMINISTIC` | T074 builds bounded canonical context capsules with required canonical references/facts. |
| FR-030 | `PROVEN_DETERMINISTIC` | T074 retains fact authority/source classification. |
| FR-031 | `PROVEN_DETERMINISTIC` | T074 prevents imported history from overwriting protected Winds/human truth. |
| FR-032 | `PROVEN_DETERMINISTIC` | T074 compaction leaves canonical capsule truth unchanged. |
| FR-033 | `PROVEN_DETERMINISTIC` | T074 transfer report encodes transferred/derived/omitted/unavailable categories. |
| FR-034 | `PROVEN_DETERMINISTIC` | T074 explicitly refuses hidden/private-state transfer claims. |
| FR-035 | `PROVEN_DETERMINISTIC` | T074/T075 keep prompt-like imported/tool/repository text inert unless an authorized action path promotes it. |
| FR-036 | `PROVEN_DETERMINISTIC` | T075 separates Planner direct authority from delegation ceiling. |
| FR-037 | `PROVEN_DETERMINISTIC` | T075 computes Worker authority as the intersection of child/delegation/team/human ceilings. |
| FR-038 | `PROVEN_DETERMINISTIC` | T075 rejects self-expansion from Planner/Worker/model/tool/repository/imported text. |
| FR-039 | `PROVEN_DETERMINISTIC` | T075 proves explicit deny precedence. |
| FR-040 | `PROVEN_DETERMINISTIC` | T075/T081 enforce one Planner -> one Worker and reject recursive/multi-worker topology. |
| FR-041 | `PROVEN_DETERMINISTIC` | T076/T081 expose normalized inspectable delegation/approval content and authority planes. |
| FR-042 | `PROVEN_DETERMINISTIC` | T076/T081/T082 invalidate materially changed content-bound approval. |
| FR-043 | `PROVEN_DETERMINISTIC` | T076 persists protected approval/audit state outside ordinary governed worktree content. |
| FR-044 | `PROVEN_DETERMINISTIC` | T075 models truthful enforcement-quality values and preserves weaker labels. |
| FR-045 | `PROVEN_DETERMINISTIC` | T075 refuses `WINDS_ENFORCED` without complete Winds mediation. |
| FR-046 | `PROVEN_DETERMINISTIC`; `LIVE_ACCEPTANCE_DEFERRED` for real Worker | T077/T081/T082/T083 keep completion `AGENT_REPORTED` until independently observed; real T082 Worker PASS remains deferred. |
| FR-047 | `PROVEN_DETERMINISTIC` | T082 and canonical trust boundaries preserve `WORKTREE != SANDBOX`. |
| FR-048 | `PROVEN_DETERMINISTIC`; `LIVE_ACCEPTANCE_DEFERRED` for real Worker | T082 binds the deterministic Worker worktree to exact Git identity and retains dirty/failed/ambiguous state. |
| FR-049 | `PROVEN_DETERMINISTIC` | T082/T083 add no automatic winner/merge/rebase/cherry-pick/push/PR/primary-checkout mutation path. |
| FR-050 | `PROVEN_DETERMINISTIC` | T083 binds acceptance identity to exact commit OID + tree. |
| FR-051 | `PROVEN_DETERMINISTIC` | T083 review context binds exact candidate and acceptance criteria. |
| FR-052 | `PROVEN_DETERMINISTIC` | T083 excludes builder persuasion/confidence from review authority. |
| FR-053 | `PROVEN_DETERMINISTIC` | T083 bridges accepted evidence to repository-native `winds verify`; T085 re-runs Spec 003 authority regressions. |
| FR-054 | `PROVEN_DETERMINISTIC` | T083 marks prior candidate-bound evidence/review stale on candidate movement. |
| FR-055 | `PROVEN_DETERMINISTIC` | T083 retains historical stale evidence traceability. |
| FR-056 | `PROVEN_DETERMINISTIC` | T083 proves Agent done/success cannot imply VERIFIED/ACCEPTED. |
| FR-057 | `PROVEN_DETERMINISTIC` | T083 preserves explicit human landing; no automatic landing path was introduced. |
| FR-058 | `PROVEN_DETERMINISTIC` | T084 implements dependency-free deterministic searchable/fuzzy session selection. |
| FR-059 | `PROVEN_DETERMINISTIC` | T084 canonicalizes file/directory selections before use and rejects workspace escape. |
| FR-060 | `PROVEN_DETERMINISTIC` | T084 preserves changed/recent/test/symbol provenance and explicit semantic-unavailable state. |
| FR-061 | `PROVEN_DETERMINISTIC` | T084/T075 prove picker visibility does not grant read/send/modify/execution authority. |
| FR-062 | `PROVEN_DETERMINISTIC` | T070/T073/T076 canonical persistence is local-first; no cloud control plane is required. |
| FR-063 | `PROVEN_DETERMINISTIC` | T072/T074/T076 preserve secret/environment non-duplication boundaries. |
| FR-064 | `PROVEN_DETERMINISTIC` | T073/T082/T085 retain failed/dirty/interrupted/ownership-lost/stale/ambiguous states truthfully. |
| FR-065 | `PROVEN_DETERMINISTIC` | T085 and accepted platform workflows limit claims to directly exercised Linux/macOS/Windows/WSL domains. |
| FR-066 | `PROVEN_DETERMINISTIC` | Spec 003/T085 preserve native-Windows terminal support separately from unsupported native-Windows authoritative `winds verify`. |
| FR-067 | `PROVEN_GOVERNANCE_BOUNDARY` | No persistent daemon/session owner is required or claimed by the accepted first program. |

No FR marked `PROVEN_DETERMINISTIC` converts a separately governed live-runtime lane into PASS. T079, T080, and T082 physical-runtime acceptance remain governed by Section 2.

## 5. Success-criterion reconciliation

| Criterion | Status | Canonical evidence / disposition |
| --- | --- | --- |
| SC-001 | `PROVEN_DETERMINISTIC` | T070 rename fixtures preserve stable identities, task links, and evidence relationships. |
| SC-002 | `PROVEN_DETERMINISTIC` | T070 proves at least 20 sessions across at least 5 workstreams without identity collision. |
| SC-003 | `PROVEN_DETERMINISTIC` | T071 proves distinct continue/fork/new-task relationships without heuristic task creation. |
| SC-004 | `PROVEN_DETERMINISTIC`; `LIVE_ACCEPTANCE_DEFERRED` where physical resume is required | T073/T078 fixtures produce truthful resume/reconstruction/ownership-loss outcomes without false resume claims. |
| SC-005 | `PROVEN_DETERMINISTIC` | T074 and T085 prove byte-stable canonical JSON/SHA-256 and preserved provenance under repeated/reordered input. |
| SC-006 | `PROVEN_DETERMINISTIC`; physical cross-runtime execution `NOT_CLAIMED` | T081 fixture proves canonical cross-runtime handoff with explicit unavailable/private-state boundaries. |
| SC-007 | `PROVEN_DETERMINISTIC` | T072 discovery fixtures prove zero Agent prompt/model/install/update/terms/credential-duplication activity. |
| SC-008 | `PROVEN_DETERMINISTIC` | T075/T076/T085 adversarial cases reject over-ceiling, self-escalation, deny bypass, prompt injection, and changed approvals. |
| SC-009 | `PROVEN_DETERMINISTIC` | T075 proves explicit truthful enforcement quality and rejects false `WINDS_ENFORCED`. |
| SC-010 | `PROVEN_DETERMINISTIC` | T083/T085 invalidate all candidate-bound review/evidence applicability after candidate movement while retaining history. |
| SC-011 | `PROVEN_DETERMINISTIC` | T083 independently models exact review binding and proves builder confidence cannot satisfy acceptance. |
| SC-012 | `PROVEN_DETERMINISTIC`; `LIVE_ACCEPTANCE_DEFERRED` for real Worker execution | T081/T082 implement exactly one Planner -> one Worker walking skeleton without recursive fleet machinery; real Worker PASS remains deferred. |
| SC-013 | `PROVEN_GOVERNANCE_BOUNDARY` | No accepted first-program change requires daemon, public network control, MCP runtime, remote executor, generic plugin marketplace, custom renderer, SQL Studio, or LLM Observatory. |
| SC-014 | `PROVEN_DETERMINISTIC` | T083/T085 plus release-candidate T064 regression jobs preserve Spec 003 verification authority and keep Agent reports ineligible as verification evidence. |

## 6. Platform and deterministic hardening truth

T085 is the cumulative negative/fault/repetition campaign. On its accepted exact head it ran the repository's full deterministic graph, including focused T085 tests and the previously accepted T070–T084 surfaces. It added bounded fake-runtime/context repetitions rather than an invented live-model soak.

Direct platform claims are limited to the exact CI domains exercised by accepted workflows: Ubuntu/Linux, macOS, native Windows terminal/touched-surface checks, and real Windows+Ubuntu WSL2 where the repository workflow directly proves that domain. Native Windows terminal/ConPTY success does not imply native-Windows authoritative `winds verify` support.

Historical same-SHA runner retries and earlier-head failures remain historical evidence; they are not rewritten as if the first attempt succeeded, and they are not used to qualify a moved candidate.

## 7. Scope / non-claims

The first implementation program proves the deterministic local control-plane substrates, identity/continuity, runtime discovery and binding truth, deterministic context, bounded authority/approval semantics, fake structured runtime clients, implementation surfaces for bounded Codex/Claude proof lanes, deterministic cross-runtime handoff, Worker worktree/verification/evidence bindings, deterministic findability, and bounded cross-platform hardening covered by T070–T085.

It does **not** claim unavailable physical-runtime acceptance. In particular, no CI, mock, fixture, generic container, repository merge, or reviewer statement is treated as substitute evidence for T079 live Codex PASS, T080 live Claude PASS, or T082 real Worker PASS.

The following remain unstarted unless separately respecified and canonically authorized:

```text
ACP_DEPENDENCY=NOT_AUTHORIZED
MCP_RUNTIME=NOT_AUTHORIZED
DAEMON_IPC=NOT_AUTHORIZED
REMOTE_EXECUTION=NOT_AUTHORIZED
GENERIC_PLUGIN_FRAMEWORK=NOT_AUTHORIZED
RECURSIVE_FLEETS=NOT_AUTHORIZED
CUSTOM_RENDERER=NOT_AUTHORIZED
SQL_STUDIO=NOT_AUTHORIZED
LLM_OBSERVATORY=NOT_AUTHORIZED
LATER_AGENTIC_PHASE=NOT_AUTHORIZED_BY_T086
```

## 8. T086 acceptance gates

This artifact is not canonical acceptance merely because it exists. Before T086 may close, the exact final closeout candidate must satisfy the repository Standard Acceptance Gate and the T086-specific acceptance gate:

- T070–T085 canonical merge reconciliation remains correct;
- the FR-001..FR-067 and SC-001..SC-014 dispositions above remain truthful on the final head;
- canonical task-state truth is reconciled on the final closeout candidate when `tasks.md` is updated;
- no stale older-head evidence is represented as current;
- exact-head correctness/safety/evidence-integrity review passes;
- exact-head Ponytail/YAGNI review passes;
- fresh independent substantive review reaches the exact final candidate;
- zero unresolved material findings/threads;
- deterministic CI gates required for the candidate pass;
- real-runtime/platform claims remain limited to actual evidence;
- changed-file/base/head/tree/ruleset state is reconciled immediately before landing;
- landing uses an expected-head guard;
- post-merge `main` and tree are verified before declaring closure.

The candidate may mark task truth complete before merge, but that branch-local task truth is not canonical completion. Only after the final closeout candidate satisfies every gate above and lands canonically may repository truth state:

```text
T070..T086=CLOSED_CANONICAL
SPEC_006_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_006_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
SPEC_006_LIVE_RUNTIME_ACCEPTANCE=DEFERRED_EXTERNAL
T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
```

Closing T086 does not authorize a successor Agentic phase.