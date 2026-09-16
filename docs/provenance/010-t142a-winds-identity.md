# Spec 010 T142A — Winds IDE Identity Qualification

Status: CANDIDATE

## Authority

- Canonical predecessor: T142 merge `f89dd4f65173b298f36b5da85e2520d6df1dddda`.
- Amendment 002 guarded merge: `54880f256551659be5fb4b4456754c7c9f69d9d9`.
- Amendment 002 post-merge `quality` run `35036384884`: `success`.
- PR #209 post-merge closure: `AMENDMENT_002=CLOSED_CANONICAL`, `T142A=AUTHORIZED`.
- T143 remains blocked until T142A is `CLOSED_CANONICAL`.

## Authorized implementation boundary

T142A is presentation and renderer composition only. This candidate may change `PRODUCT.md`, `DESIGN.md`, `desktop/src/**`, `desktop/index.html`, `desktop/tests/**`, the T142A workflow, and this provenance document.

It does not authorize root Rust, Tauri Rust command-surface changes, Git/filesystem/runtime/provider/credential/persistence/network authority, migrations, package dependencies, or lockfile changes.

## Candidate intent

- establish a narrow Winds activity rail with Winds-authored Current Mark geometry;
- make Chat the default left tool window and Projects its sibling;
- bind left Chat presentation to one exact selected canonical Session;
- expose composer availability truthfully without inventing dispatch;
- make the center a one/two-Session operational Workbench with Terminal primary;
- retain the contextual Inspector and compact status rail;
- evolve Quiet Current into the independent Current Spectrum identity;
- preserve accessibility, exact-target selection, dual-Session identity, layout persistence, terminal ownership, Inspector binding, Needs You, and safe command navigation.

## Local qualification before publication

The candidate must pass from a clean worktree:

```text
npm run format:check
npm run typecheck
npm run lint
npm test
npm run frontend:build
```

The focused T142A test must prove default Chat, sibling Projects, exact-target Chat binding, truthful unavailable composer behavior, Workbench composition, Current Spectrum primitives, deterministic dark/light Chat/Projects fixtures, and retained contextual binding.

No local result substitutes for exact-head CI, independent review, guarded merge, merge verification, or actually-triggered post-merge workflows.

## Required external qualification

Before T142A can close canonically:

1. publish one bounded implementation PR from the exact candidate;
2. run all applicable exact-head CI, including `t142a-winds-identity`;
3. apply Alibaba OpenCodeReview to the complete candidate when available;
4. complete author correctness/safety/design and Ponytail/YAGNI reviews;
5. obtain a fresh independent exact-head substantive review with zero material findings;
6. reconcile changed files, dependencies, Tauri command surface, review threads, base/head/tree, and mergeability;
7. guarded-merge only the reviewed expected head;
8. verify merge parentage, tree identity, and GitHub signature;
9. require success for every workflow actually triggered by the merge commit.

T142A does not substitute for T144. Human Founder visual acceptance remains mandatory after T143 performance qualification.

## Auxiliary repository-wide Rust baseline comparison

A local macOS auxiliary `cargo test --locked --all-targets --all-features` was run once on the T142A candidate even though T142A changes no Rust/Tauri source and this command is not a T142A renderer gate.

Candidate result:

```text
647 passed; 6 failed; 4 ignored
```

Every failure was an existing T090 terminal cleanup bounded-exit assertion. The identical command was then run once, without retry-to-green, in a detached clean worktree at canonical base `54880f256551659be5fb4b4456754c7c9f69d9d9` on the same host.

Canonical-base result:

```text
646 passed; 7 failed; 4 ignored
```

The base failures were the same T090 bounded terminal child-exit/cleanup class, with one additional test in that class. No Rust or Tauri source is changed by T142A, so this macOS-local baseline behavior is recorded rather than repaired outside T142A authority. Exact-head GitHub CI remains authoritative for candidate qualification; no local rerun is substituted for the failed auxiliary result.
