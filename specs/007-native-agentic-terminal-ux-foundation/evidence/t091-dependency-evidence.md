# T091 Dependency Qualification Evidence

Candidate lineage used for generation: `075b0042cced021771b98910cb668e2ab7453d1c`

Workflow run: `34131716135`

Artifact: `t091-lock-evidence-075b0042cced021771b98910cb668e2ab7453d1c` (`10022335426`)

Artifact ZIP SHA-256: `d62806eb225593cabd55a00fed76de107b22af47804cbb1aa1bf5aad1b7ebce8`

Cargo-generated `Cargo.lock` SHA-256: `20f9f2ce6517e53419a9df1a53acfdd06b658aba23927a9626c2ef333e4dce57`

## Direct dependency

- package: `ratatui-textarea`
- version: `0.9.2`
- source: `registry+https://github.com/rust-lang/crates.io-index`
- checksum: `3c78d5ba0f26f97baed69a4c479f268a31c7b5b89d68ab939842152e383d6e73`
- license: `MIT`
- MSRV: `1.86`
- enabled feature: `crossterm`

## Resolved boundary

The Cargo-generated graph enables only the required Crossterm backend for `ratatui-textarea`. The optional `search`/`regex`, `termion`, `termwiz`, `serde`, `arbitrary`, and `portable-atomic` feature paths are not activated. The workflow rejects Tokio, regex, `tui-term`, Termion, and Termwiz expansion.

The generated metadata contains 139 resolved packages. Every resolved package declares a license, every registry package has a Cargo-recorded checksum, and every declared MSRV is compatible with the repository Rust target `1.97.1`.

The graph remains on the accepted Ratatui/Crossterm family and introduces no second PTY/terminal runtime, async executor, network/provider/browser client, clipboard automation library, daemon/service framework, LSP/editor framework, or unrelated UI framework.

This record is dependency provenance only. It is not final T091 qualification. The temporary lock-generation workflow must be removed before final candidate-bound CI/review begins.