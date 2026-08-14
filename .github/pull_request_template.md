## What changed

Describe the smallest implementation slice in this PR.

## Spec Kit traceability

- Active spec: `specs/.../spec.md`
- Plan/tasks updated if scope changed: [ ]
- Acceptance scenario(s) proven: [ ]

## Deterministic evidence

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] Slice-specific required checks

## Review stack

- [ ] Correctness/safety review completed
- [ ] Ponytail over-engineering review completed
- [ ] Independent reviewer pass completed
- [ ] External reviewer findings reconciled when available

## Winds safety invariants

- [ ] Primary checkout is not mutated by candidate flows
- [ ] No forced worktree cleanup/deletion
- [ ] Evidence binds to exact candidate state
- [ ] Agent-reported claims are not promoted to observed truth
- [ ] No automatic winner/merge/rebase/push behavior introduced

## Findings and exceptions

List any advisory findings, unsupported reviewer findings, or explicitly accepted exceptions.
