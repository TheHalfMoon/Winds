# Donor and Process Provenance Ledger

This ledger records external projects that materially shape Winds. Before code is copied/adapted, exact paths, source commit, license, modifications, and update strategy must be added.

| Project | Pin | License | Winds use | Reuse mode |
|---|---|---|---|---|
| `github/spec-kit` | v0.16.4 / `d1f50fcbe684a4222059c4ba7f2d7eabcca87402` | MIT | Spec-driven workflow structure: Constitution -> Spec -> Plan -> Tasks -> Implement | Process/template reference; Winds-authored artifacts |
| `DietrichGebert/ponytail` | v4.9.0 / `0a4dd63ad4541f4f655c4108a295916f3c1d8fda` | MIT | Mandatory YAGNI/over-engineering review discipline | Review/process reference; no runtime dependency |
| `HKUDS/DeepCode` | `287510fbf6820147a48adf79f7fd86b0ed1afe92`; `core/skills/builtin/review-agent/SKILL.md` | MIT | Read-only defect-first review methodology: complete diff, surrounding code, actionable P0-P3 findings, no invented issues | Review/process reference; no runtime dependency or copied runtime code |
| System Git | feature-probed; minimum target >= 2.36 | GPL-2.0-only executable boundary | Git/ref/worktree authority | Invoke executable; parse machine-readable output |
| SQLite / `rusqlite` | rusqlite 0.40.2; committed `Cargo.lock` | SQLite public domain / rusqlite MIT | Transactional local metadata/events/projections | `rusqlite` dependency with bundled SQLite |
| Rust `libc` | 0.2.189; committed `Cargo.lock` | MIT OR Apache-2.0 | Unix `O_NOFOLLOW` / `O_NONBLOCK` constants for race-resistant validation of pre-existing evidence blobs | Direct dependency; platform constants only |
| `amElnagdy/delegate-skills` | pin required before agent adapter implementation | MIT | Future adapter semantics/adversarial test corpus | Study/reimplement selectively; not runtime dependency |

No copied donor runtime code is currently present. The implementation in Winds is Winds-authored; external projects above are dependencies or process/design references only.
