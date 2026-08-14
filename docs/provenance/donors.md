# Donor and Process Provenance Ledger

This ledger records external projects that materially shape Winds. Before code is copied/adapted, exact paths, source commit, license, modifications, and update strategy must be added.

| Project | Pin | License | Winds use | Reuse mode |
|---|---|---|---|---|
| `github/spec-kit` | v0.16.4 / `d1f50fcbe684a4222059c4ba7f2d7eabcca87402` | MIT | Spec-driven workflow structure: Constitution -> Spec -> Plan -> Tasks -> Implement | Process/template reference; Winds-authored artifacts |
| `DietrichGebert/ponytail` | v4.9.0 / `0a4dd63ad4541f4f655c4108a295916f3c1d8fda` | MIT | Mandatory YAGNI/over-engineering review discipline | Review/process reference; no runtime dependency |
| System Git | feature-probed; minimum target >= 2.36 | GPL-2.0-only executable boundary | Git/ref/worktree authority | Invoke executable; parse machine-readable output |
| SQLite / `rusqlite` | rusqlite 0.40.1; Cargo.lock pending CI generation | SQLite public domain / rusqlite MIT | Transactional local metadata/events/projections | `rusqlite` dependency with bundled SQLite |
| `amElnagdy/delegate-skills` | pin required before agent adapter implementation | MIT | Future adapter semantics/adversarial test corpus | Study/reimplement selectively; not runtime dependency |

No copied donor code is currently present. The implementation in Winds is Winds-authored; external projects above are dependencies or process/design references only.
