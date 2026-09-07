# Spec 007 Tasks Amendment 002 — Bounded T091 Lock Generation

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 Governance deviation/amendment process, the Founder directive in the active project session to continue the authorized Winds program, canonical T091 dependency authority for `ratatui-textarea = "=0.9.2"`, and the accepted T087 dependency-lock generation precedent recorded by Spec 007 Amendment 001.

## Purpose

T091 requires exact dependency qualification for `ratatui-textarea = "=0.9.2"`, including a Cargo-generated `Cargo.lock`, resolved graph, source identities, checksums, enabled features, licenses, MSRVs, and compatibility with the already accepted Ratatui/Crossterm graph.

The execution environment performing repository work does not expose a usable local Cargo toolchain or outbound Cargo registry resolution. Hand-editing `Cargo.lock` would weaken evidence integrity. Amendment 001 intentionally authorizes its temporary workflow only for T087 and explicitly does not authorize `ratatui-textarea`, so T091 requires a separate bounded governance decision rather than reuse of stale authority.

This amendment authorizes only the smallest temporary lock-generation mechanism needed for T091. It does not broaden T091 product scope, direct dependency authority, runtime behavior, or successor-task authority.

## Precedence and Scope

When canonical, this amendment supplements only T091's authorized-path boundary for temporary dependency-lock generation.

All existing Spec 007 `tasks.md` requirements remain unchanged, including exact dependency authority, the Standard Acceptance Gate, no-Tokio/no-daemon/no-IPC boundaries, exact submitted-byte semantics, safe multiline/paste fallback, focused-test registration, exact-head review, guarded landing, and the prohibition on unauthorized dependencies.

T091 remains the only authorized implementation task. T092–T100 remain dependency-blocked until T091 closes canonically.

## Temporary Workflow Authority

During T091 development only, a branch-local temporary workflow may be added at:

```text
.github/workflows/t091-lock-generation.yml
```

The temporary workflow is authorized only to:

1. run for the T091 development branch/PR candidate;
2. use `permissions: contents: read` and checkout with `persist-credentials: false`;
3. verify the exact checked-out candidate SHA before generating evidence;
4. install the repository-pinned Rust `1.97.1` toolchain using the same pinned toolchain action already accepted by repository workflows;
5. run `cargo generate-lockfile` from the repository root after `Cargo.toml` contains only the already-authorized new direct dependency `ratatui-textarea = "=0.9.2"`;
6. run read-only Cargo metadata/tree commands needed to expose the exact resolved dependency graph and enabled feature graph;
7. print deterministic SHA-256 and package/checksum evidence for the generated `Cargo.lock`, or upload only that generated lock/evidence as a GitHub Actions artifact using an already repository-qualified pinned action;
8. inspect the resolved `ratatui-textarea` package metadata needed to establish exact source, checksum, license, MSRV, direct/transitive dependencies, and feature activation;
9. perform no Git commit, push, PR mutation, tag, release, merge, rebase, cherry-pick, branch mutation, credential operation, product/runtime execution, shell-editor execution, or terminal-child execution;
10. use network access only as required for the explicitly authorized checkout, retrieval of repository-qualified pinned GitHub Actions, installation of the pinned Rust `1.97.1` toolchain, and Cargo registry/index/package resolution for the already-authorized dependency graph. All other workflow-initiated network access remains prohibited.

The workflow MUST NOT contain a write-capable repository token or `contents: write` permission.

## Mandatory Removal Before Final T091 Candidate

The temporary workflow is a development instrument only. It MUST be deleted from the T091 branch after its generated lock/evidence has been incorporated and before final candidate qualification begins.

The final T091 base-to-head changed-file set MUST NOT contain `.github/workflows/t091-lock-generation.yml`.

No CI, author review, Ponytail review, independent review, platform evidence, or merge-ready evidence produced before removal may qualify the final T091 candidate. After removal, all candidate-bound T091 gates start again on the exact final HEAD/TREE.

Historical T091 lock-generation workflow runs remain historical provenance only.

## Dependency and Feature Boundary

This amendment does not change T091's exact direct dependency authority:

```text
ratatui-textarea = "=0.9.2"
```

No other new direct dependency is authorized.

The generated graph must still prove:

- the direct version resolves exactly to `0.9.2` from the expected crates.io source with a Cargo-recorded checksum;
- enabled features are the minimum justified for the T091 editor adapter and do not activate regex/search or unrelated capability merely because available;
- the dependency remains compatible with the already accepted Ratatui `0.30.2` / Crossterm `0.29.0` graph and does not introduce a second terminal backend/runtime;
- no Tokio/async executor, network/provider/browser client, clipboard automation dependency, daemon/service framework, LSP/editor framework, or unrelated UI framework enters the resolved T091 graph;
- crate MSRVs are compatible with Winds Rust `1.97.1`;
- direct and transitive licenses are compatible with Winds `MIT OR Apache-2.0` distribution;
- exact crates.io source identities and checksums are recorded from the Cargo-generated lockfile.

If the generated graph violates any existing T091 boundary, T091 remains blocked until repaired within existing authority or a separately accepted canonical amendment changes scope.

## Explicit Non-Authorization

This amendment does not authorize:

- a permanent lock-generation workflow;
- weakening, deleting, or bypassing an existing workflow/check;
- GitHub Actions write permissions;
- direct dependency versions other than `ratatui-textarea = "=0.9.2"`;
- regex/search features solely for editor convenience;
- Tokio, async runtimes, `tui-term`, another PTY/terminal runtime, browser/provider/model libraries, clipboard automation, daemon/IPC, plugins, semantic/embedding search, LSP/editor frameworks, or benchmark crates;
- a terminal child, persistent event loop, navigation/search UI, verification projection, host-open behavior, database migration, Git mutation, provider/model behavior, or any T092+ implementation;
- treating the temporary workflow run as final T091 qualification;
- bypassing exact-head CI, focused T091 test execution, author correctness/safety/evidence-integrity review, Ponytail/YAGNI review, independent review, zero-thread reconciliation, guarded merge, or post-merge verification.

## Acceptance of This Amendment

This amendment is not canonical merely because this file exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review bound to the exact candidate, zero unresolved material findings/threads, exact one-file scope reconciliation, guarded expected-head landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing may the temporary T091 lock-generation workflow be created.