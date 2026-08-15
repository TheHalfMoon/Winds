# Winds 0.1 Licensing and Provenance Audit

**Audit task**: Spec 002 / T025  
**Baseline repository head**: `1516078d22ae64cd0a3b04a0d402351669846e70`  
**Canonical base**: `e4c94ed98743f39bbc4da9ad3e9b8fa3cfb4b7ef`  
**Audit date**: 2026-08-15  
**Status**: PASS for proceeding to source-license implementation; binary third-party notices remain a later release-artifact obligation

This is an engineering release-readiness audit, not a substitute for a legal opinion. Its purpose is to make the repository's current reuse/licensing facts explicit and to fail closed on any unresolved provenance before public publication.

## Conclusion

No current blocker was identified for licensing Winds-authored source as `MIT OR Apache-2.0`.

The current repository records no copied/adapted donor runtime code. Material external projects are either process/design references, ordinary package dependencies, bundled SQLite, or the system Git executable boundary. The locked Rust dependency graph contains 49 non-workspace packages; a deterministic `cargo metadata --locked --format-version 1` probe reported license metadata for all 49 packages and no package missing both a license expression and license file.

One additional distribution obligation must not be lost: `unicode-ident 1.0.24` is licensed as `(MIT OR Apache-2.0) AND Unicode-3.0`. Binary release artifacts that incorporate that dependency must carry the applicable Unicode license/notice together with the selected dependency notices. T035/release-artifact work must preserve this obligation.

## Deterministic Dependency Evidence

A temporary, read-only PR workflow checked out exact head `1516078d22ae64cd0a3b04a0d402351669846e70`, installed pinned Rust `1.97.1`, and ran:

```text
cargo metadata --locked --format-version 1
```

The probe used Python stdlib only to print package name/version/license metadata and failed if a locked non-workspace package had neither `license` nor `license_file` metadata.

Observed terminal markers:

```text
LOCKED_DEPENDENCY_COUNT=49
LICENSE_METADATA_COMPLETE=YES
```

The same exact head passed the ordinary quality workflow on Ubuntu and macOS: format, Clippy `-D warnings`, and tests.

The probe workflow was diagnostic only and is removed in the same T025 acceptance commit; Winds does not retain a one-off license-audit CI subsystem.

## Locked License Inventory by Expression

The inventory is grouped to keep this audit maintainable while retaining exact package identities.

### `MIT OR Apache-2.0` — 36 package entries

`bitflags 2.13.1`, `block-buffer 0.10.4`, `bumpalo 3.20.3`, `cc 1.4.3`, `cfg-if 1.0.4`, `cpufeatures 0.2.17`, `crypto-common 0.1.7`, `digest 0.10.7`, `find-msvc-tools 0.1.11`, `hashbrown 0.16.1`, `hashbrown 0.17.1`, `hashlink 0.12.1`, `itoa 1.0.18`, `js-sys 0.3.104`, `libc 0.2.189`, `once_cell 1.21.4`, `pkg-config 0.3.34`, `proc-macro2 1.0.107`, `quote 1.0.47`, `rustversion 1.0.23`, `serde 1.0.229`, `serde_core 1.0.229`, `serde_derive 1.0.229`, `serde_json 1.0.151`, `sha2 0.10.9`, `shlex 2.0.1`, `smallvec 1.15.2`, `syn 2.0.119`, `syn 3.0.3`, `thiserror 2.0.20`, `thiserror-impl 2.0.20`, `typenum 1.20.1`, `wasm-bindgen 0.2.127`, `wasm-bindgen-macro 0.2.127`, `wasm-bindgen-macro-support 0.2.127`, `wasm-bindgen-shared 0.2.127`.

### Legacy `MIT/Apache-2.0` dual-license notation — 4 package entries

`fallible-iterator 0.3.0`, `fallible-streaming-iterator 0.1.9`, `vcpkg 0.2.15`, `version_check 0.9.5`.

### `MIT` — 6 package entries

`generic-array 0.14.7`, `libsqlite3-sys 0.38.2`, `rsqlite-vfs 0.1.1`, `rusqlite 0.40.2`, `sqlite-wasm-rs 0.5.5`, `zmij 1.0.23`.

### Other permissive expressions — 3 package entries

- `foldhash 0.2.0` — `Zlib`.
- `memchr 2.8.3` — `Unlicense OR MIT`; Winds distribution can follow the MIT option.
- `unicode-ident 1.0.24` — `(MIT OR Apache-2.0) AND Unicode-3.0`; the Unicode license is additional, not optional, for the generated Unicode-derived material.

No copyleft Rust package license appeared in the locked Cargo metadata inventory.

## Direct Runtime Dependencies

Current direct dependencies in `Cargo.toml` are:

| Dependency | Locked/current license metadata | Reuse |
|---|---|---|
| `libc` | `MIT OR Apache-2.0` | linked Rust dependency for Unix platform constants |
| `rusqlite` | `MIT` | linked Rust dependency; bundled SQLite enabled |
| `serde` | `MIT OR Apache-2.0` | linked Rust dependency |
| `serde_json` | `MIT OR Apache-2.0` | linked Rust dependency |
| `sha2` | `MIT OR Apache-2.0` | linked Rust dependency |

Bundled SQLite deliverable code is recorded in the provenance ledger as public domain. Current SQLite upstream states that its deliverable code is dedicated to the public domain and may be copied, used, compiled, sold, or distributed. If Winds changes from bundled upstream SQLite to another source/build arrangement, this classification must be re-audited.

## Donor / Process Provenance Review

`docs/provenance/donors.md` is the canonical material-influence ledger.

| External project/boundary | Current release relevance | Audit disposition |
|---|---|---|
| `github/spec-kit` | process/template reference | MIT; Winds-authored artifacts; no runtime code copied |
| `DietrichGebert/ponytail` | review/process reference | MIT; no runtime dependency/code copy |
| `HKUDS/DeepCode` | read-only review methodology | MIT; no runtime dependency/code copy |
| System Git | executable Git/ref/worktree authority | GPL-2.0-only executable boundary; Winds invokes the user's system Git and does not link/copy/bundle Git code in 0.1 |
| SQLite / `rusqlite` | persistence dependency | SQLite public-domain deliverable code; rusqlite MIT |
| Rust `libc` | direct dependency | `MIT OR Apache-2.0` |
| `amElnagdy/delegate-skills` | future adapter study only | not an implemented/release dependency; exact pin remains required before future adapter reuse |

A repository search for the named process/donor projects found the donor ledger rather than runtime source references. The repository contains five current Rust source files under `src/`; the existing ledger states that the implementation is Winds-authored. This audit found no contrary evidence or recorded copied donor runtime code.

This is not a proof that unmarked copying is impossible. If later review identifies copied/adapted source, publication must stop until exact path/commit/license/modification provenance is added and compatibility is re-evaluated.

## Source-License Compatibility Decision

For the current Winds-authored source tree, the planned `MIT OR Apache-2.0` license is compatible with the current engineering reuse model:

- process-reference projects do not contribute copied runtime code;
- direct/locked Rust dependencies expose permissive license expressions;
- system Git remains an external executable boundary rather than bundled/link-time code;
- bundled SQLite deliverable code is public domain;
- no current Cargo dependency has missing license metadata.

This decision does not eliminate third-party notice obligations for binary artifacts. In particular, the final artifact workflow must account for MIT-family notices and the additional Unicode-3.0 material used by `unicode-ident`.

## Release Obligations Carried Forward

1. T026 may add the standard Winds `LICENSE-MIT` and `LICENSE-APACHE` texts and reconcile repository source-license truth.
2. T027 may set package metadata to `license = "MIT OR Apache-2.0"` while preserving `publish = false`.
3. If `Cargo.lock` dependency identities change before the final release candidate, rerun the locked license inventory and update this audit if the license set changes materially.
4. T035 binary artifact work must include/attach appropriate third-party license notices, including Unicode-3.0 for `unicode-ident`.
5. Any future decision to bundle Git, copy donor source, publish to crates.io, or add new distribution packaging reopens the relevant licensing audit.

## T025 Verdict

`PASS — NO CURRENT LICENSE/PROVENANCE BLOCKER IDENTIFIED FOR PROCEEDING TO T026.`
