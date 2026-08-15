# Winds 0.1 Public Repository Hygiene Audit

**Task**: Spec 002 / T037  
**Final automated audit head**: `9f4cff591d8f8af284d035914cbbc665f6652d29`  
**Final probe run**: `31900819095`  
**Audit date**: 2026-08-15  
**Status**: PASS; the temporary probe was removed after the final successful content scan

## Scope

This audit covers tracked repository content intended to become public. It does not rewrite or sanitize Git history automatically. If a real historical secret is later identified, publication must stop and the event must be handled as a separate credential-rotation/history-remediation incident.

The temporary `public-hygiene-probe` workflow excluded only its own file from the scan because that file embedded the detection regex literals. The probe was not part of the intended accepted tree and was removed immediately after the final successful scan. Removing that excluded scanner implementation did not add unscanned public content.

## Automated tracked-content scan

The final read-only GitHub Actions probe checked out exact head `9f4cff591d8f8af284d035914cbbc665f6652d29` with persisted credentials disabled and scanned `git ls-files` content other than the probe itself.

Observed markers:

```text
AUDITED_TRACKED_FILE_COUNT=39
TEXT_FILE_COUNT=39
BINARY_OR_NON_UTF8_FILE_COUNT=0
HIGH_CONFIDENCE_FINDINGS=0
PUBLIC_SURFACE_REVIEW_MATCHES=61
SOURCE_PROVENANCE_MARKERS=0
TRACKED_PUBLIC_HYGIENE_HIGH_CONFIDENCE=PASS
```

High-confidence blocker patterns covered:

- GitHub personal/fine-grained token shapes;
- OpenAI-style secret-key shapes;
- AWS access-key IDs;
- Slack token shapes;
- PEM/OpenSSH private-key headers;
- signed/tokenized HTTP(S) URLs;
- platform-specific absolute user-home paths on macOS, Linux, and Windows.

The scanner printed only rule/path/line metadata when a blocker was found; it did not echo matched secret values.

## Public-surface semantic triage

The review-only keyword matches were manually reconciled rather than promoted to automatic failures.

### `sandbox` / `native Windows`

Current release-facing README, SECURITY, CHANGELOG, CONTRIBUTING, AGENTS, and constitution occurrences are explicit **non-claims** or scope boundaries. They state that worktrees are not a security sandbox, that OS/network/secret isolation is not provided, and that native Windows execution semantics are not part of the 0.1 support claim.

No tracked public-facing document claims OS/network/secret sandboxing or native Windows support.

### `pre-alpha`, `internal-only`, `unpublished`, and `0.0.0`

Remaining occurrences are in historical/as-built specification and planning records describing the earlier repository baseline, deferred scope, or the release-readiness transition from the internal placeholder package version. They do not represent the current package metadata or current public README contract.

Current release-facing truth is:

- package version `0.1.0`;
- `publish = false` for crates.io/package publication;
- GitHub release status remains unreleased until founder-controlled T041;
- public-facing README describes the 0.1 contract rather than calling the current product pre-alpha.

Historical specifications are retained as provenance/evidence records rather than rewritten to hide project history.

## Source/provenance review

The automated scan found zero third-party copyright/SPDX/license markers under `src/`, `tests/`, `scripts/`, or `migrations/` that would contradict the current Winds-authored implementation provenance claim.

The intentionally vendored third-party material under `third-party/licenses/` is license/notice text only and has explicit package/version provenance. In particular, `rsqlite-vfs 0.1.1` carries a pinned upstream MIT license override because its crates.io archive declares MIT metadata but omits a physical license file; no runtime source from that upstream is copied into Winds.

`docs/provenance/donors.md` remains the canonical material-influence ledger and currently records no copied donor runtime code.

## Private URLs and machine details

No high-confidence signed/tokenized URL or local user-home path was found in the intended tracked corpus. Repository URLs, upstream provenance URLs, and GitHub API/repository URLs intentionally documented as public project/upstream references are not treated as leaks.

No personal machine username/path is required by the public build, test, verification, release-candidate, or security-reporting instructions.

## Release-language review

The current public project surface consistently distinguishes:

- `0.1.0` release-candidate package metadata from an actual GitHub release;
- GitHub distribution intent from disabled crates.io publication;
- verified candidate selection from automatic downstream Git integration;
- checkout/index isolation from security sandboxing;
- supported Linux/macOS/WSL2-Linux-domain usage from unsupported native Windows semantics.

No stale public-facing `0.0.0` package claim or automatic publication promise was identified.

## Publication boundary

This audit does not authorize publication. At the time of the release-candidate dry run, the repository remained private, Git tags and GitHub Releases were empty, and package publication remained disabled. T041 still requires a separate founder authorization naming the exact release commit before visibility/tag/release/package mutation.

## T037 verdict

`PASS — NO TRACKED-CONTENT PUBLICATION BLOCKER IDENTIFIED.`

The final probe successfully scanned the complete intended public corpus at `9f4cff591d8f8af284d035914cbbc665f6652d29`, excluding only its own temporary workflow file, and was then removed. The accepted tree therefore contains no additional unscanned implementation content from that probe.
