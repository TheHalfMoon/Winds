# Winds Agent Instructions

## Product truth
Winds is an independent verification runtime for agent-generated software. The core job is not to spawn the most agents; it is to establish trustworthy evidence about exact candidate snapshots and promote only explicit human selections safely.

## Mandatory workflow
1. Read `.specify/memory/constitution.md`.
2. Read the active `specs/<feature>/spec.md`, `plan.md`, and `tasks.md`.
3. Implement only the current authorized task/slice.
4. Run deterministic checks before requesting review.
5. Run a correctness/safety review.
6. Run Ponytail simplicity review and remove unjustified complexity.
7. Obtain an independent reviewer pass before acceptance.

## Ponytail mode
Default to Ponytail `full` behavior: understand first; do not build what is not required; prefer stdlib/platform/existing dependency; avoid speculative abstractions; write the minimum correct implementation. Never simplify away validation, security, accessibility, evidence completeness, Git/data safety, or recovery requirements.

Pinned process reference: `DietrichGebert/ponytail` v4.9.0, commit `0a4dd63ad4541f4f655c4108a295916f3c1d8fda`.

## Non-negotiable Winds safety rules
- Never treat agent-reported success as authoritative check evidence.
- Never mutate the primary checkout during candidate verification/promotion.
- Never force-clean or force-remove a worktree.
- Never recursively delete a candidate path whose ownership is ambiguous.
- Never auto-select a winner.
- Never merge/rebase/cherry-pick/push/open PRs as product behavior in 0.1.
- Worktrees are not sandboxes; do not claim OS/network/secret isolation.

## Architecture guardrails
For 0.1 do not introduce a daemon, IPC/public runtime protocol, terminal emulator, generic runtime abstraction, Graphify/code graph, Jujutsu dependency, plugin system, MCP/A2A, port broker, service orchestration, broad sandbox framework, or AI reviewer in product code unless the active spec is explicitly amended first.
