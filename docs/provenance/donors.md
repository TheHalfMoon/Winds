# Donor and Process Provenance Ledger

This ledger records external projects that materially shape Winds. Before code is copied/adapted, exact paths, source commit, license, modifications, and update strategy must be added.

| Project | Pin | License | Winds use | Reuse mode |
|---|---|---|---|---|
| `github/spec-kit` | v0.16.4 / `d1f50fcbe684a4222059c4ba7f2d7eabcca87402` | MIT | Spec-driven workflow structure: Constitution -> Spec -> Plan -> Tasks -> Implement | Process/template reference; Winds-authored artifacts |
| `DietrichGebert/ponytail` | v4.9.0 / `0a4dd63ad4541f4f655c4108a295916f3c1d8fda` | MIT | Mandatory YAGNI/over-engineering review discipline | Review/process reference; no runtime dependency |
| `HKUDS/DeepCode` | `287510fbf6820147a48adf79f7fd86b0ed1afe92`; `core/skills/builtin/review-agent/SKILL.md` | MIT | Read-only defect-first review methodology: complete diff, surrounding code, actionable P0-P3 findings, no invented issues | Review/process reference; no runtime dependency or copied runtime code |
| System Git | feature-probed; minimum target >= 2.36 | GPL-2.0-only executable boundary | Git/ref/worktree authority | Invoke executable; parse machine-readable output |
| SQLite / `rusqlite` | rusqlite 0.40.2; committed `Cargo.lock` | SQLite public domain / rusqlite MIT | Transactional local metadata/events/projections | `rusqlite` dependency with bundled SQLite |
| Rust `libc` | 0.2.189; committed `Cargo.lock` | MIT OR Apache-2.0 | Unix platform constants/syscalls used by verification and the T050 live PTY ownership check | Direct dependency; narrow platform boundary only |
| WezTerm `portable-pty` | crate `0.9.0`; published-source VCS `f8921727a11b9f8b073e8c24821d72fd41283500`; upstream path `pty/`; locked crates.io checksum `b4a596a2b3d2752d94f51fac2d4a96737b8705dddd311a32b9af47211f08671e` | MIT | Spec 003 PTY allocation, spawn, resize, I/O, foreground-process-group observation, and owned child-lifecycle primitive | **Exact direct dependency landed by T050 as `portable-pty = "=0.9.0"`.** Cargo-generated lockfile and exact transitive package/license audit are recorded in `docs/provenance/portable-pty-0.9.0-lock-audit.md`. No WezTerm runtime source is copied/adapted into Winds. Native-Windows/ConPTY runtime proof remains T051. |
| `retep998/winapi-rs` | tag `0.3.9` / `796a8e6c2971dc2ff1bcff166e6671284f9b5b6b`; target crates `winapi-i686-pc-windows-gnu 0.4.0` and `winapi-x86_64-pc-windows-gnu 0.4.0` | MIT/Apache-2.0 | Exact upstream license-text fallback for target-specific lock entries whose crates.io payloads do not expose a distributable license file to the release collector | License/notice text only, copied byte-for-byte from the pinned upstream target-crate source; no runtime source copied. Overrides live under `third-party/licenses/winapi-rs-0.3.9/`. |
| `amElnagdy/delegate-skills` | pin required before agent adapter implementation | MIT | Future adapter semantics/adversarial test corpus | Study/reimplement selectively; not runtime dependency |

No copied donor runtime code is currently present. The implementation in Winds is Winds-authored; external projects above are dependencies, approved dependency candidates, process/design references, or third-party notice sources only.

Winds-authored source is licensed under `MIT OR Apache-2.0` through the repository's standard `LICENSE-MIT` and `LICENSE-APACHE` files. That source-license choice does not replace or relicense third-party dependency terms; release artifacts must continue to satisfy applicable upstream notice obligations recorded by the release licensing audit.
