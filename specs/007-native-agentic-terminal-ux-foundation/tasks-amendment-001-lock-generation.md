# Spec 007 Tasks Amendment 001 — Bounded T087 Lock Generation

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Constitution 1.1.0 governance deviation/amendment process, the Founder directive in the active project session to continue the authorized Winds program, and the repository precedent recorded in `docs/provenance/portable-pty-0.9.0-lock-audit.md` for Cargo-generated dependency lock evidence.

## Purpose

T087 requires exact dependency qualification for `ratatui = "=0.30.2"` and `crossterm = "=0.29.0"`, including a Cargo-generated `Cargo.lock`, resolved graph, source identities, checksums, features, licenses, MSRVs, and platform evidence.

The execution environment performing repository work does not currently expose a usable local Cargo toolchain or outbound Git resolution. Hand-editing a dependency lockfile would weaken evidence integrity. Existing Winds precedent generated dependency lock state in an isolated GitHub Actions job under the pinned Rust toolchain and removed the temporary workflow before final candidate review.

This amendment authorizes only that bounded lock-generation mechanism for T087. It does not broaden T087 product scope or dependency authority.

## Precedence and Scope

When canonical, this amendment supplements only T087's authorized-path boundary for temporary dependency-lock generation.

All existing Spec 007 `tasks.md` requirements remain unchanged, including exact dependency authority, Standard Acceptance Gate requirements, no-daemon/no-IPC boundaries, platform truth, focused-test registration, exact-head review, guarded landing, and the prohibition on unauthorized dependencies.

T087 remains the only authorized implementation task. T088–T100 remain dependency-blocked.

## Temporary Workflow Authority

During T087 development only, a branch-local temporary workflow may be added at:

```text
.github/workflows/t087-lock-generation.yml
```

The temporary workflow is authorized only to:

1. run for the T087 development branch/PR candidate;
2. use `permissions: contents: read` and checkout with `persist-credentials: false`;
3. verify the exact checked-out candidate SHA before generating evidence;
4. install the repository-pinned Rust `1.97.1` toolchain using the same pinned toolchain action already accepted by repository workflows;
5. run `cargo generate-lockfile` from the repository root;
6. run read-only Cargo metadata/tree commands needed to expose the exact resolved dependency graph and enabled feature graph;
7. print deterministic SHA-256 and package/checksum evidence for the generated `Cargo.lock`, or upload only that generated lock/evidence as a GitHub Actions artifact using an already repository-qualified pinned action if available;
8. perform no Git commit, push, PR mutation, tag, release, merge, rebase, cherry-pick, branch mutation, credential operation, or product/runtime execution;
9. use network access only as required for the explicitly authorized checkout, retrieval of repository-qualified pinned GitHub Actions, installation of the pinned Rust `1.97.1` toolchain, and Cargo registry/index/package resolution for the already-authorized dependency graph. All other workflow-initiated network access remains prohibited.

The workflow MUST NOT contain a write-capable repository token or `contents: write` permission.

## Mandatory Removal Before Final T087 Candidate

The temporary workflow is a development instrument only. It MUST be deleted from the T087 branch after its generated lock/evidence has been incorporated and before final candidate qualification begins.

The final T087 base-to-head changed-file set MUST NOT contain `.github/workflows/t087-lock-generation.yml`.

No CI, author review, Ponytail review, independent review, benchmark, platform evidence, or merge-ready evidence produced before removal may qualify the final T087 candidate. After removal, all candidate-bound T087 gates start again on the exact final HEAD/TREE.

Historical lock-generation workflow runs remain historical provenance only.

## Dependency and Feature Boundary

This amendment does not change T087's exact direct dependency authority:

```text
ratatui = "=0.30.2"
crossterm = "=0.29.0"
```

No other direct dependency is authorized.

The generated graph must still prove:

- Ratatui defaults are disabled if they introduce unjustified features;
- the accepted minimal Ratatui feature set is explicit;
- Crossterm `osc52` is not enabled;
- Crossterm `event-stream` is not enabled;
- no Tokio/async executor, network/provider/browser client, clipboard automation dependency, daemon/service framework, or unrelated UI framework enters the resolved T087 graph;
- crate MSRVs are compatible with Winds Rust `1.97.1`;
- direct and transitive licenses are compatible with Winds `MIT OR Apache-2.0` distribution;
- exact crates.io source identities and checksums are recorded from the Cargo-generated lockfile.

If the generated graph violates any existing T087 boundary, T087 remains blocked until repaired within existing authority or a separately accepted canonical amendment changes scope.

## Explicit Non-Authorization

This amendment does not authorize:

- a permanent lock-generation workflow;
- weakening or deleting any existing workflow/check;
- GitHub Actions write permissions;
- dependency versions other than the exact T087 direct versions;
- `vt100`, `ratatui-textarea`, Tokio, async runtimes, `tui-term`, Alacritty terminal runtime, browser/provider/model libraries, clipboard automation, daemon/IPC, plugins, learning/vector memory, LSP/editor frameworks, or benchmark crates;
- a terminal child, PTY/ConPTY ownership, persistent event loop, parser, editor, history persistence, database migration, Git mutation, provider/model behavior, or any T088+ implementation;
- treating the temporary workflow run as final T087 qualification;
- bypassing exact-head CI, author review, Ponytail/YAGNI review, independent review, zero-thread reconciliation, guarded merge, or post-merge verification.

## Acceptance of This Amendment

This amendment is not canonical merely because this file exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review bound to the exact candidate, zero unresolved material findings/threads, exact one-file scope reconciliation, guarded expected-head landing, and post-merge canonical main/tree verification.

Only after successful canonical landing may the temporary T087 lock-generation workflow be created.