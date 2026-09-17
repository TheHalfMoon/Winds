# Spec 010 Tasks Amendment 007 — T143 Post-Merge macOS SQLite I/O Qualification Isolation

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 010 T143 qualification authority, canonical Amendment 006 selection-feedback repair authority, canonical no-rerun-to-green discipline, existing infrastructure-only exact-head retry precedents that do not apply to a failure occurring inside repository tests, and the active Founder directive to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Material evidence

Canonical production repair merge `3dbd5828120095eff3b9db4c358357918fd701b7` has tree `fcfc0473793c09daa4ab7162e9739849cb56403e`, exactly matching the accepted PR #219 head `d217ce6bacb7ff167ac08616b448a697fea85e84` tree.

The exact PR head passed all eleven triggered workflows, including macOS `quality`. On the identical post-merge tree, ten of eleven workflows passed, while post-merge `quality` run `35173120818` failed only in macOS job `105048846406`; the Ubuntu job passed.

The macOS job passed checkout identity, format, and Clippy, then failed during the full `cargo test --locked --all-targets --all-features` suite. The run completed 616 tests successfully before reporting 37 failures and 4 ignored tests. The failures span unrelated T094/T098/T109/T131-T138 surfaces and share the same SQLite error class: `SqliteFailure(Error { code: SystemIoFailure, extended_code: 1034 }, Some("disk I/O error"))` while opening independent temporary fixture stores. No failed assertion identifies the selection-feedback repair, and the accepted PR tree and merge tree are byte-identical.

The failed post-merge run remains material evidence and MUST NOT be rerun on the same head to obtain green status.

## Authorized repair scope

This amendment authorizes one forward-only CI determinism repair:

- `.github/workflows/quality.yml`: on macOS only, execute the unchanged full Rust test suite with one Rust test thread so SQLite/WAL fixture activity is serialized on the hosted temporary volume;
- an existing deterministic workflow/source assertion test may be extended to prove the macOS-only serialization command and to prove Ubuntu retains ordinary parallel execution;
- `docs/provenance/010-t143-performance.md` may record the post-merge failure and repair identity.

The authoritative command remains `cargo test --locked --all-targets --all-features`; only the macOS test harness concurrency setting may change to `-- --test-threads=1` or an equivalent exact one-thread form.

## Non-negotiable boundaries

The repair MUST NOT:

- rerun post-merge `quality` run `35173120818` or job `105048846406` as acceptance evidence;
- skip, ignore, filter, tolerate, or delete any test or assertion;
- change production Rust, desktop product behavior, SQLite journal mode, persistence semantics, runtime authority, terminal authority, verification authority, or approval authority;
- weaken T143 thresholds, sample floors, committed-selection predicates, security gates, accessibility gates, or platform gates;
- alter Ubuntu test concurrency unless a new material failure separately justifies new authority;
- add retries, `continue-on-error`, timeout inflation, or runner-result relabeling.

## Required proof

The repair successor must prove on its new exact head:

1. workflow/source regression coverage proves macOS uses the unchanged full suite with exactly one test thread and Ubuntu remains the unchanged full suite;
2. `quality` succeeds on both Ubuntu and macOS without skipped or filtered acceptance tests beyond the repository's pre-existing ignored tests;
3. every other workflow triggered by the repair PR remains green;
4. after guarded merge, the same applicable workflows succeed on the new canonical merge;
5. T143 is forward-integrated onto that canonical merge and begins performance qualification from a new exact head;
6. all historical failed heads and runs remain preserved as failed evidence.

## Closure discipline

This amendment becomes canonical only after expected-head guarded merge and successful post-merge qualification on its own documentation-only candidate. The subsequent workflow repair must land separately under this authority. T143 remains blocked until the repair is canonical and forward-integrated; this amendment does not close T143, authorize T144, or substitute for later Founder visual acceptance.
