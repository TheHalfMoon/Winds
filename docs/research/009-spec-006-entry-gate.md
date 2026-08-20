# Spec 006 formal entry gate

**Status:** ENTRY CANDIDATE — governance/specification only; no Agentic runtime implementation authorized by this document.

**Evaluated:** 2026-08-21

**Canonical base evaluated:** `6eb6390b0f7cb33ac5215a5990589c8018ed05d6`

**Source direction:** `docs/research/008-agentic-development-master-plan.md`

## Gate result

The Spec 003 prerequisite is now satisfied. Winds may proceed into **Phase A — formalize future product semantics**, but implementation remains blocked until the formal Spec 006 user scenarios, security/authority boundaries, measurable outcomes, and adversarial/deterministic test requirements are accepted through the repository process.

## Formal Spec 006 entry criteria

### 1. Spec 003 canonically closed

**PASS.**

Canonical `main` is `6eb6390b0f7cb33ac5215a5990589c8018ed05d6`, the guarded merge of PR #65. T068 and T069 are checked in canonical task truth and Spec 003 is closed.

### 2. No conflicting active implementation slice

**PASS.**

At evaluation time the remaining open PRs are not competing implementation slices:

- PR #62 is a historical exact-head T068 review-only PR whose body explicitly says **DO NOT MERGE**;
- PR #21 is a draft research archive and explicitly does not authorize Agent Fleet/ACP/MCP/runtime implementation.

Neither may be reused as the Spec 006 implementation branch or treated as current product authority.

### 3. Constitution/product wording amended for post-0.1 Agentic goals

**PENDING CANONICAL MERGE OF THIS GOVERNANCE PR.**

The accompanying Constitution 1.1.0 amendment adds explicit canonical-continuity, local-authority/delegation, runtime/model separation, capability-truth, protocol-pinning, and persistent-owner threat-model gates while preserving exact evidence, human selection, non-destructive Git recovery, and independent-review requirements.

No Spec 006 implementation may begin until that amendment is canonical.

### 4. Exact ACP protocol/SDK revision pinned and audited

**PASS AS SPECIFICATION INPUT.**

The accompanying provenance audit freezes:

```text
ACP_WIRE_PROTOCOL=1
ACP_SCHEMA=schema-v1.20.0
ACP_SCHEMA_COMMIT=5e89c71497fe07dd4ae633c181a17224f4a8956d
ACP_RUST_SDK=2.0.0
ACP_RUST_SDK_COMMIT=ce023279824149008659dd8f4b8b70266a7e8210
UNSTABLE_PROTOCOL_V2=DISABLED
UNSTABLE_MCP_OVER_ACP=DISABLED
```

The pin authorizes specification/dependency planning only. The first task that lands the crate must perform a fresh exact Cargo graph/checksum/license/platform/toolchain audit and may reopen this decision if the landing gates fail.

### 5. Exact MCP revision pinned if MCP is used

**NOT TRIGGERED / MCP EXCLUDED FROM THE FIRST SLICE.**

The first formal Spec 006 walking skeleton does not require MCP. No MCP runtime is authorized by the ACP pin. If a later task introduces MCP, the exact then-current MCP specification and SDK must be pinned before execution is enabled.

### 6. Persistent owner / IPC threat model before coding

**SATISFIED BY FIRST-SLICE NON-GOAL; FUTURE GATE REMAINS BLOCKING.**

The first formal Agentic slice will not add a daemon, persistent session owner, network listener, public runtime protocol, HTTP/SSE/WebSocket control plane, or remote execution. If persistent ownership or IPC is later proposed, an explicit threat model plus versioned lifecycle/ownership/authenticated-local-control design is required before code.

### 7. User scenarios and measurable acceptance outcomes before architecture

**REQUIRED IN FORMAL SPEC 006 BEFORE PLAN.**

The spec must freeze prioritized scenarios at minimum for:

1. named workspace/session identity with rename-safe stable IDs;
2. explicit `continue` / `fork` / `new` semantics where `NEW_SESSION != NEW_TASK`;
3. local Codex/Claude runtime discovery with runtime/model/capability truth separated;
4. structured continuation with explicit `LIVE` / `RESUMED` / `RECONSTRUCTED` / `OWNERSHIP_LOST` proof vocabulary as applicable;
5. one human-approved Planner -> Worker delegation with distinct Planner direct authority and delegation ceiling;
6. exact-candidate independent verification/review and stale-evidence invalidation.

### 8. Authority/trust boundaries and non-goals before adapters execute tools

**REQUIRED IN FORMAL SPEC 006 BEFORE PLAN/IMPLEMENTATION.**

The spec must define:

- `deny > ask > allow` policy semantics where applicable;
- Planner execution authority separately from delegation ceiling;
- child execution authority bounded by delegation/team/human ceilings;
- enforcement quality (`WINDS_ENFORCED`, `OS_SANDBOX_ENFORCED`, `AGENT_NATIVE_ENFORCED`, `BEST_EFFORT_TRIPWIRE`, `OBSERVATION_ONLY`, `UNAVAILABLE`);
- protected Winds policy/trust state;
- imported history and agent/tool output as non-authoritative until explicitly reconciled;
- worktree and ACP workspace roots as non-sandbox boundaries;
- no self-escalation and no automatic landing/winner behavior.

### 9. Deterministic continuity/security/recovery/review-staleness tests before implementation

**REQUIRED IN FORMAL SPEC 006 BEFORE TASK IMPLEMENTATION.**

The spec must include deterministic/adversarial cases for rename identity, ambiguous continuation, native-resume fallback, reconstructed handoff truth, imported-history provenance, child-authority ceilings, explicit deny precedence, candidate movement invalidating review, dirty/failed state retention, runtime replacement identity, and platform/domain claims.

### 10. Smallest Codex/Claude connected-session + single-delegation walking skeleton first

**FROZEN DIRECTION.**

Do not begin with a fleet, marketplace, plugin system, daemon, remote scheduler, renderer, MCP mesh, or every coding agent. The first implementation path must prove the differentiated loop with Codex/Claude structured integrations and one bounded delegation path, then demonstrate that a later runtime can be swapped without rewriting the Winds workspace/session/task/authority/evidence model.

## Authorized next action after this governance gate is canonical

Create formal:

```text
specs/006-agentic-terminal-local-delegation-control-plane/
  spec.md
```

The initial specification PR is **specification-only**. It must not add runtime source, dependencies, migrations, daemon/IPC, provider adapters, model calls, prompt execution, remote control, MCP, or workflow-semantic expansion.

Only after the spec is accepted may Winds proceed to `plan.md`, then `tasks.md`, then the first explicitly authorized implementation task.

## Non-authorization statement

This entry gate does not start an agent, send a prompt, call a model/provider API, inspect private model reasoning, install an agent, duplicate credentials, mutate a user repository checkout, create a persistent service, or authorize automatic merge/push/PR behavior.
