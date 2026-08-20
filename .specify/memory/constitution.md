# Winds Constitution

## Core Principles

### I. Evidence Over Agent Claims
Winds MUST distinguish `AGENT_REPORTED`, `WINDS_OBSERVED`, and `HUMAN_DECIDED` information. Agent claims, model prose, imported history, and inferred state are never authoritative verification evidence merely because Winds can read or retain them. Promotion and acceptance policy may depend only on Winds-observed facts and explicit human decisions. Evidence must bind to an exact Git base and candidate snapshot.

### II. Non-Destructive Git Safety
Winds MUST treat system Git as the change authority. Winds-controlled operations MUST NOT modify the user's primary checkout, force-clean, force-remove worktrees, resolve conflicts, or perform consequential landing/remote Git actions without an explicitly specified and human-authorized policy. Dirty, mismatched, live, failed, or ambiguous state is retained for recovery rather than deleted. Automatic winner selection and silent landing remain prohibited.

### III. Spec-Driven, Testable Slices
Every implementation slice MUST follow the Spec Kit sequence: Constitution -> Spec -> Plan -> Tasks -> Implement. User scenarios and acceptance criteria precede architecture and implementation. The smallest runnable check that proves the slice is mandatory. A feature is incomplete when its required evidence cannot be reproduced.

### IV. Simplicity Is a Quality Gate
Ponytail review is mandatory for implementation diffs. Prefer, in order: not building the feature; standard library/platform behavior; an already-adopted dependency; the smallest correct implementation. Speculative abstractions, generic runtimes, public protocols, service boundaries, and dependencies require concrete current use cases. Simplicity MUST NOT remove validation, security boundaries, accessibility, data-safety checks, recovery behavior, provenance, or authority truth.

### V. Independent Review Before Acceptance
No implementation is accepted solely because its authoring agent says it works. Every PR MUST pass deterministic CI, correctness/safety review, Ponytail over-engineering review, and at least one independent reviewer pass. Security-sensitive or Git-destructive code requires an explicit safety review. External reviewers such as Qodo, Cubic, Greptile, CodeRabbit, or other available review systems may add evidence but never replace deterministic gates.

### VI. Canonical Continuity Across Runtimes
Winds MUST keep canonical workspace, task/workstream, session, authority, and evidence identity separate from any provider/runtime/model/native-session identity. `NEW_SESSION != NEW_TASK` and `NEW_AGENT != NEW_TASK`. Native runtime resume, live process reattachment, Winds reconstruction, and cross-runtime handoff are different proof levels and MUST NOT be collapsed into a generic "restored" claim. Model-context compaction or imported/vendor history MUST NOT silently rewrite canonical work/evidence truth.

### VII. Explicit Local Authority and Delegation Ceilings
Direct execution authority and delegation authority are distinct. A child actor's execution authority MUST remain within the explicit human-approved delegation/team ceiling, and no model, planner, worker, tool output, hook, project file, or imported history may self-expand that ceiling. Safety-relevant capability claims MUST identify their enforcement quality and fail truthfully when Winds cannot mediate the underlying access. Winds-managed policy/trust state MUST be protected from the actors governed by that same policy.

### VIII. Runtime and Capability Truth
Winds MUST model agent runtime/harness separately from model/provider and distinguish catalog-declared, vendor-declared, and Winds-locally-observed capability state. Discovery is not trust and MUST NOT auto-install, auto-execute, or imply authentication readiness. Prefer pinned structured ACP/vendor-native control over terminal scraping when available. Protocol workspace roots, Git worktrees, and agent-native permission UX are not OS sandboxes unless an actual enforcement boundary proves otherwise.

## Product and Technical Constraints

- Current public product baseline remains the independent verification runtime released as `v0.1.0`; accepted later specifications may extend local workspace/execution behavior without weakening verification authority.
- Post-0.1 product direction: a verified local runtime for agentic software development that preserves canonical continuity, explicit local authority, exact-candidate evidence, reviewer independence, and human landing decisions across changing agent runtimes.
- Primary verification primitive remains `winds verify`; orchestration, connected sessions, delegation, and teams are secondary to exact evidence and authority truth.
- Human selects the candidate. Winds MUST NOT produce a magic winner score or silently promote/merge a candidate.
- Core implementation language: Rust by default, not by ideology.
- The accepted local architecture remains one-process unless a later formal specification authorizes a persistent owner. Any long-lived owner or IPC/control surface requires an explicit threat model, versioned lifecycle/ownership design, authenticated local-control semantics, and the narrowest private surface that satisfies current product need before any public protocol is considered.
- First formal agentic integration targets remain Codex and Claude Code. Broader runtimes are capability-driven follow-ons, not justification for a generic plugin/runtime platform up front.
- ACP integration MUST pin an exact stable wire/schema/SDK decision before implementation. Draft/unstable protocol surfaces require separate explicit authorization and may not be enabled by convenience.
- MCP use, if introduced by a later task, MUST pin the exact specification/SDK revision and define its authority/enforcement boundary before execution is enabled. ACP support does not silently authorize MCP.
- Platform claims MUST match deterministic evidence for the exact accepted slice. Native-Windows workspace/ConPTY support does not imply native-Windows authoritative verification when that authority remains unsupported.
- Git worktrees are workspace isolation, not an OS security sandbox. ACP additional roots and agent-native workspace declarations are also scope declarations, not sandbox proof.
- Local-first operation MUST NOT acquire an implicit cloud control-plane dependency merely to support connected sessions or local delegation.
- Persistent/background execution, remote execution, public runtime protocols, broad plugin/provider runtimes, rich terminal rendering, SQL/DB surfaces, LLM observability surfaces, and large heterogeneous fleets require their own explicitly scoped specifications/tasks and evidence.

## Development Workflow and Review Gates

1. **Specify**: Write prioritized, independently testable user scenarios, security/non-goals, and measurable outcomes.
2. **Plan**: Choose the smallest architecture that satisfies the spec and constitution; pin external protocols/dependencies before relying on them.
3. **Tasks**: Break the plan into independently reviewable slices with explicit tests and authority boundaries.
4. **Implement**: Build only the current authorized task scope.
5. **Deterministic gate**: format, compile, lint, unit/integration tests, and required fixture/platform checks.
6. **Correctness/safety review**: look for wrong behavior, Git/data loss, incomplete evidence, recovery gaps, security/authority overclaims, stale identity, and cross-platform assumptions.
7. **Ponytail review**: identify code/dependencies/abstractions/protocol surfaces that can be deleted or replaced by simpler native behavior.
8. **Independent reviewer pass**: a reviewer other than the authoring agent challenges the exact diff/candidate and acceptance claims.
9. **Evidence reconciliation**: unresolved findings remain blocking unless explicitly classified advisory with rationale; candidate movement invalidates stale evidence/review.

Reviewers MUST label claims as fact, inference, or recommendation when the distinction matters. Review output is evidence; it is not itself product truth.

## Governance

This constitution supersedes convenience, agent preference, imported history, and speculative roadmap architecture. Deviations require an explicit documented decision stating the current evidence, why the simpler path is insufficient, the affected authority/security boundary, and how the decision remains reversible where possible.

Spec Kit is used as a pinned process reference at `github/spec-kit` v0.16.4, commit `d1f50fcbe684a4222059c4ba7f2d7eabcca87402`. Ponytail is used as a pinned simplicity/review reference at `DietrichGebert/ponytail` v4.9.0, commit `0a4dd63ad4541f4f655c4108a295916f3c1d8fda`.

**Version**: 1.1.0 | **Ratified**: 2026-08-14 | **Last Amended**: 2026-08-21
