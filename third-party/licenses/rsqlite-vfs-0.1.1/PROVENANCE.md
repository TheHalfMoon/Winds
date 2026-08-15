# rsqlite-vfs 0.1.1 license override provenance

The crates.io archive for `rsqlite-vfs 0.1.1` declares `MIT` license metadata but does not contain a physical license file, so the release-candidate bundle cannot copy one directly from Cargo's unpacked crate directory.

Winds therefore carries the upstream MIT license text for this exact package/version only.

- Upstream repository: `Spxg/sqlite-wasm-rs`
- Upstream source commit inspected: `b2b7450bfdef4b63d769ca804892f49975f9aac7`
- Package manifest: `crates/rsqlite-vfs/Cargo.toml`
- Package version: `0.1.1`
- Workspace license expression: `MIT`
- License source path: repository-root `LICENSE`
- License source blob: `8839511c01e91a5d4ac8c9b753c165b52d86f530`
- Reuse mode: verbatim license/notice text only; no upstream runtime code is copied into Winds

`collect_licenses.py` recognizes only the exact `(rsqlite-vfs, 0.1.1)` tuple for this override. Any other locked package/version that lacks a distributable license/notice file continues to fail the release-candidate build closed.
