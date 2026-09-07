# Donor Slice Provenance Template

**Status:** Documentation template only. This file does not admit any donor or authorize copying by itself.

Use this template whenever Winds copies, adapts, translates, or vendors a concrete source slice from an external repository.

## Identity

- Winds capability/task:
- Winds destination path(s):
- Reuse mode: `INTEGRATE_STANDARD | DIRECT_DEPENDENCY | SMALL_CODE_COPY | SEMANTIC_TRANSLATION | TEST_CORPUS_ADAPTATION | DESIGN_ONLY`
- Exact Winds candidate SHA/tree:

## Upstream source

- Repository:
- Exact commit/tag:
- Exact source path(s):
- Relevant source line/range or symbol where practical:
- Upstream project license:
- File-level license/header if different:
- Required notice/copyright files:
- Third-party/generated/vendor provenance checked: `YES | NO | NOT_APPLICABLE`

## Founder/project authorization

- Founder source-use authorization applies: `YES | NO`
- Additional explicit decision, if required:

Founder authorization does not replace upstream license/notice obligations or Winds governance gates.

## Why reuse is justified

- Concrete problem solved:
- Why existing Winds code/dependency is insufficient:
- Why standard integration is insufficient, if copying code:
- Why a smaller Winds-native implementation is not preferable:
- Expected reduction in implementation/platform/maintenance risk:

## Modification record

- Copied verbatim: `YES | NO`
- Translated to Rust/other language: `YES | NO`
- Material modifications:
- Donor behaviors intentionally removed/disabled:
- Winds invariants stronger than donor defaults:

## Authority and evidence boundary

Confirm all that apply:

- [ ] Donor output cannot become `WINDS_OBSERVED` merely because donor code produced it.
- [ ] Donor success/completion state cannot become verification or human acceptance automatically.
- [ ] Donor code cannot expand execution/delegation authority.
- [ ] Donor runtime/session IDs remain separate from canonical Winds identity.
- [ ] Donor worktree/session/policy labels are not treated as sandbox proof.
- [ ] Candidate-bound evidence becomes stale when exact candidate identity changes.

## Deterministic qualification

- Focused tests:
- Regression tests:
- Adversarial/security tests:
- Platform tests:
- Recovery/fault tests:
- Performance/soak tests if applicable:
- License/notice collection test if shipped in release artifacts:

## Upstream drift and security ownership

Choose one update policy:

- [ ] Frozen until a concrete defect/security issue requires update.
- [ ] Periodically compared with pinned upstream releases.
- [ ] Updated only with the owning Winds capability.
- [ ] Transitional copy; planned replacement by a standard integration/dependency.

- Upstream release/advisory source to watch:
- Security-update owner/process:
- How Winds can identify the copied version later:

## Rollback / replacement

- How this donor slice can be removed or replaced:
- Data/schema compatibility impact:
- Fallback behavior:

## Review record

- Correctness/safety review:
- Ponytail/YAGNI review:
- Independent review:
- Unresolved findings:
- Final provenance entry added to `docs/provenance/donors.md`: `YES | NO`

## Acceptance rule

A copied/adapted slice is not accepted merely because it compiles or retains an upstream license header. Acceptance requires complete provenance, exact-candidate deterministic evidence, Winds-specific correctness/safety tests, Ponytail review, independent review, and canonical landing under the owning specification/task authority.