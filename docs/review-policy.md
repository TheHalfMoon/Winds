# Winds Review Policy

Winds is built under an evidence-first review policy. Reviewers are intentionally redundant because authoring agents, static analyzers, LLM reviewers, and deterministic tests fail in different ways.

## Required review stack

### 1. Deterministic gate — blocking
- Format check.
- Compiler/type correctness.
- Clippy/lint with warnings denied for Winds code.
- Unit/integration/fixture tests.
- Slice-specific safety invariants from the active Spec Kit specification.

### 2. Correctness and safety review — blocking
Review the diff against the spec, not against author intent. Hunt for wrong behavior, stale evidence, Git/data-loss risks, unsafe cleanup, process/recovery bugs, security overclaims, unchecked error paths, and unsupported platform assumptions.

### 3. Ponytail review — blocking for unjustified complexity
Use Ponytail v4.9.0 semantics as an independent simplicity pass. The reviewer should return a delete/simplify list: reinvented standard-library/platform behavior, unneeded dependencies, speculative abstractions, dead flexibility, duplicate state, or code added only for hypothetical future features. Safety/validation/recovery requirements are not simplification targets.

### 4. Independent model/reviewer pass — blocking when a correctness finding is substantiated
The implementation author must not be the sole reviewer. Use a different agent/model or human reviewer to challenge acceptance claims and inspect the actual diff/tests.

### 5. External reviewer services — additive
When connected, Qodo, Cubic, Greptile, CodeRabbit, GitHub review tooling, security scanners, or equivalent systems should be allowed to report findings. Their comments are triaged into supported/unsupported findings; no bot is authoritative merely because it commented.

## Finding reconciliation

Each finding is classified:
- `BLOCKING_VALID`
- `ADVISORY_VALID`
- `DUPLICATE`
- `UNSUPPORTED`
- `OUT_OF_SCOPE`

A blocking valid finding must be fixed or explicitly re-authorized by the founder with evidence. Unsupported findings are documented rather than silently ignored.

## Review order

Run deterministic checks first, then correctness/safety, then Ponytail, then independent review. This order prevents reviewers from spending tokens on code that does not compile or on complexity that correctness fixes will replace.

## Anti-patterns

- Never ask multiple reviewers the same generic "review this" prompt and count agreement as proof.
- Never convert reviewer confidence into product truth.
- Never let an AI reviewer override a failing deterministic required check.
- Never merge because all bots are green while the exact spec acceptance evidence is missing.
