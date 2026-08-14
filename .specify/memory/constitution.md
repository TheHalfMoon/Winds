# Winds Constitution

## Core Principles

### I. Evidence Over Agent Claims
Winds MUST distinguish `AGENT_REPORTED`, `WINDS_OBSERVED`, and `HUMAN_DECIDED` information. Agent claims, model prose, and inferred state are never authoritative verification evidence. Promotion policy may depend only on Winds-observed facts and explicit human decisions. Evidence must bind to an exact Git base and candidate snapshot.

### II. Non-Destructive Git Safety
Winds MUST treat system Git as the change authority. Winds-controlled operations MUST NOT modify the user's primary checkout, force-clean, force-remove worktrees, merge, rebase, cherry-pick, push, open a PR, or resolve conflicts automatically in 0.1. Dirty, mismatched, live, failed, or ambiguous state is retained for recovery rather than deleted.

### III. Spec-Driven, Testable Slices
Every implementation slice MUST follow the Spec Kit sequence: Constitution -> Spec -> Plan -> Tasks -> Implement. User scenarios and acceptance criteria precede implementation. The smallest runnable check that proves the slice is mandatory. A feature is incomplete when its required evidence cannot be reproduced.

### IV. Simplicity Is a Quality Gate
Ponytail review is mandatory for implementation diffs. Prefer, in order: not building the feature; standard library/platform behavior; an already-adopted dependency; the smallest correct implementation. Speculative abstractions, generic runtimes, public protocols, service boundaries, and dependencies require concrete current use cases. Simplicity MUST NOT remove validation, security boundaries, accessibility, data-safety checks, or recovery behavior.

### V. Independent Review Before Acceptance
No implementation is accepted solely because its authoring agent says it works. Every PR MUST pass deterministic CI, correctness/safety review, Ponytail over-engineering review, and at least one independent reviewer pass. Security-sensitive or Git-destructive code requires an explicit safety review. External reviewers such as Qodo, Cubic, Greptile, CodeRabbit, or other available review systems may add evidence but never replace deterministic gates.

## Product and Technical Constraints

- Product wedge: independent verification runtime for agent-generated software.
- Primary primitive: `winds verify`; orchestration such as `winds race` is secondary.
- Human selects the candidate; Winds MUST NOT produce a magic winner score in 0.1.
- Core implementation language: Rust by default, not by ideology.
- 0.1 architecture: one process; private in-process seams; system Git; SQLite WAL plus bounded filesystem blobs.
- First supported authoring agents: Codex and Claude Code only, after the fake/existing-candidate walking skeleton proves safety.
- Initial supported environments: Linux x86-64, macOS arm64, and WSL2 x86-64 when repo/Git/Winds/agents all run inside the Linux filesystem.
- Native Windows, terminal emulation, `windsd`, public runtime protocol, Graphify, Jujutsu dependency, port/service orchestration, broad OS sandboxing, MCP/A2A, remote execution, plugin systems, and signed attestations are not 0.1 requirements.
- Git worktrees are workspace isolation, not an OS security sandbox. Winds MUST state that boundary truthfully.

## Development Workflow and Review Gates

1. **Specify**: Write prioritized, independently testable user scenarios and measurable outcomes.
2. **Plan**: Choose the smallest architecture that satisfies the spec and constitution.
3. **Tasks**: Break the plan into independently reviewable slices with explicit tests.
4. **Implement**: Build only the current authorized task scope.
5. **Deterministic gate**: format, compile, lint, unit/integration tests, and required fixture checks.
6. **Correctness/safety review**: look for wrong behavior, Git/data loss, incomplete evidence, recovery gaps, security overclaims, and cross-platform assumptions.
7. **Ponytail review**: identify code/dependencies/abstractions that can be deleted or replaced by simpler native behavior.
8. **Independent reviewer pass**: a reviewer other than the authoring agent challenges the diff and acceptance claims.
9. **Evidence reconciliation**: unresolved findings remain blocking unless explicitly classified advisory with rationale.

Reviewers MUST label claims as fact, inference, or recommendation when the distinction matters. Review output is evidence; it is not itself product truth.

## Governance

This constitution supersedes convenience, agent preference, and speculative roadmap architecture. Deviations require an explicit documented decision stating the current evidence, why the simpler path is insufficient, and how the decision remains reversible where possible.

Spec Kit is used as a pinned process reference at `github/spec-kit` v0.16.4, commit `d1f50fcbe684a4222059c4ba7f2d7eabcca87402`. Ponytail is used as a pinned simplicity/review reference at `DietrichGebert/ponytail` v4.9.0, commit `0a4dd63ad4541f4f655c4108a295916f3c1d8fda`.

**Version**: 1.0.0 | **Ratified**: 2026-08-14 | **Last Amended**: 2026-08-14
