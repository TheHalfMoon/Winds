# Spec 007 Tasks Amendment 003 — Exact T091 Cargo.lock Landing

## Status

Candidate governance amendment. It grants no authority unless and until it lands canonically on `main` and satisfies the acceptance gate below.

## Purpose

T091 requires a Cargo-generated `Cargo.lock` plus exact dependency evidence for the already authorized direct dependency `ratatui-textarea = "=0.9.2"`.

Canonical Amendment 002 authorizes a temporary read-only workflow to generate and artifact that lock and evidence, but deliberately forbids Git mutation. The connected repository-write surface available to the active execution environment accepts repository file content as reconstructed text rather than an existing artifact/file reference. A byte-for-byte transfer attempt was rejected by its pre-mutation Git-blob identity check because the reconstructed blob did not match the Cargo-generated artifact identity. Hand-editing, accepting a mismatched blob, or claiming equivalence would weaken evidence integrity.

This amendment authorizes the smallest one-time write mechanism needed to land the exact Cargo-generated lock bytes on the already authorized T091 implementation branch. It does not broaden T091 product scope, dependency authority, runtime behavior, or successor-task authority.

## Mandatory Predecessor Gate

This amendment is inert unless live canonical repository truth proves all of the following:

1. `T090=CLOSED_CANONICAL` remains true;
2. Spec 007 Amendment 002 is `CLOSED_CANONICAL` on current `main`;
3. the active branch is the dedicated T091 implementation branch descended from that canonical main;
4. the branch's `Cargo.toml` adds exactly the T091-authorized direct dependency and no other new direct dependency;
5. a fresh Amendment-002-compliant read-only generation run on the exact dependency candidate has succeeded and exposed Cargo-generated `Cargo.lock`, SHA-256, resolved feature tree, and metadata evidence;
6. that evidence proves the T091 dependency boundary, including exact `ratatui-textarea 0.9.2`, crates.io identity/checksum, minimal enabled features without search/regex, compatible license/MSRV, and no prohibited runtime/backend expansion.

Absent, stale, ambiguous, or moved-candidate proof grants no write authority.

## Temporary One-Time Write Authority

Only during the authorized T091 development slice, `.github/workflows/t091-lock-generation.yml` may be minimally changed to perform one exact Cargo-lock landing after regenerating and revalidating the dependency evidence.

For this bounded landing only, the workflow may use:

```yaml
permissions:
  contents: write
```

and the repository token only for the final guarded push described below.

No other write permission is authorized.

The workflow must:

1. check out the exact triggering T091 branch HEAD and full predecessor history;
2. fail unless the canonical Amendment-002 predecessor and T091 branch ancestry remain proven;
3. install only pinned Rust `1.97.1` through the already accepted pinned toolchain action;
4. run `cargo generate-lockfile`;
5. reproduce the lock SHA-256, resolved feature tree, and metadata evidence;
6. re-run the Amendment 002 prohibited dependency/feature checks before any Git mutation;
7. prove before staging that the working-tree mutation produced by lock generation is exactly `Cargo.lock` and no other path;
8. stage exactly `Cargo.lock` with an explicit pathspec;
9. prove the staged diff contains exactly `Cargo.lock` and that `.github/workflows/t091-lock-generation.yml`, `Cargo.toml`, source, tests, specs, migrations, and every other path are unstaged/unmodified by the landing step;
10. create one commit with the exact message `chore(007): land Cargo-generated T091 lock`;
11. immediately before push, prove the remote T091 branch still equals the workflow's triggering candidate SHA; if it moved, fail closed without push;
12. push only the newly created commit to `refs/heads/impl/007-t091-workbench-input`, with no force push and no other ref mutation;
13. print the generated lock SHA-256, generated Git blob identity, created commit SHA, and parent SHA as historical evidence.

The workflow must not amend, rebase, merge, force-push, tag, create/delete branches, alter `main`, alter pull requests, write releases, change repository settings, or commit any file other than the generated `Cargo.lock`.

## Credential Boundary

Repository credentials exist only for the bounded final push. The workflow must not print tokens, persist them into repository files/artifacts, expose them to product/runtime commands, or pass them to dependency/build/test scripts.

The lock-generation and dependency-inspection commands remain non-product dependency tooling. No Winds binary, terminal child, provider/model/browser command, editor workflow, or untrusted repository-supplied executable may receive write credentials.

## Revocation Before T091 Qualification

This write authority is temporary development infrastructure, not product behavior and not acceptance evidence.

After the exact lock commit lands on the T091 branch:

1. `.github/workflows/t091-lock-generation.yml` must be deleted from the branch;
2. no `contents: write` T091 workflow may remain;
3. all T091 implementation, focused tests, and final deterministic gates must run from the resulting workflow-free exact HEAD/TREE;
4. the lock-landing run and its review/CI are historical provenance only and cannot substitute for final T091 qualification;
5. any later movement of `Cargo.toml` or `Cargo.lock` invalidates the earlier dependency evidence and requires fresh authority/evidence as applicable before qualification.

## Scope Boundaries

This amendment does **not** authorize:

- any direct dependency beyond T091's existing `ratatui-textarea = "=0.9.2"` authority;
- hand-edited or reconstructed `Cargo.lock` content;
- permanent write-enabled workflow infrastructure;
- any product/source/test implementation in this governance PR;
- T092 or later task implementation;
- Tokio/async runtime, second terminal backend/runtime, regex/search convenience features, provider/model/browser/clipboard/network/daemon/IPC/LSP behavior;
- workflow-driven source generation or source commits;
- branch-protection/ruleset/repository-setting changes;
- weakening, skipping, or reusing stale final acceptance gates.

## Amendment Acceptance

This amendment is not canonical merely because this file exists.

The exact governance candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review bound to the exact candidate, zero unresolved material findings/threads, exact one-file scope reconciliation, guarded expected-head landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing, and only while the mandatory predecessor gate remains satisfied by live repository truth, may the one-time T091 lock landing authority be exercised.