# Winds

**Winds is the independent verification runtime for agent-generated software.**

Winds verifies one or more candidate changes from a pinned Git base, executes repository-owned checks independently of the authoring agent, records evidence with explicit provenance, and promotes only the human-selected verified snapshot without mutating the primary checkout.

## Current phase

Winds is in pre-alpha, spec-driven construction. The first implementation slice is intentionally narrow: Git-backed candidate verification, evidence capture, and safe promotion. Terminal emulation, a daemon, public runtime protocols, broad sandboxing, agent dashboards, and semantic "brain" features are out of scope until usage evidence earns them.

## Trust model

Winds distinguishes three authorities:

- **AGENT_REPORTED** — claims and events reported by an authoring agent.
- **WINDS_OBSERVED** — Git/process/check facts observed directly by Winds.
- **HUMAN_DECIDED** — explicit candidate selection, overrides, and promotion approval.

Only Winds-observed facts and human decisions may drive promotion policy.

## Development method

Construction is spec-driven using GitHub Spec Kit semantics, with a mandatory review stack that includes deterministic checks, correctness/safety review, Ponytail simplicity review, and independent reviewer passes before implementation is accepted.

See `.specify/memory/constitution.md` and `specs/001-verification-walking-skeleton/` once the governance baseline lands.
