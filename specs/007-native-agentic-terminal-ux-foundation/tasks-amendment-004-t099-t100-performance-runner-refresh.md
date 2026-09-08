# Spec 007 Tasks Amendment 004 — T099/T100 Pinned Performance Runner Refresh

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 007 Tasks, canonical Spec 007 Amendment 003, canonical T097 frozen performance requirements FR-045..FR-053, and the Founder directive in the active project session to continue the authorized Winds program through its dependency-ordered completion.

## Purpose

Canonical Amendment 003 intentionally makes the exact GitHub-hosted `ubuntu-24.04` runner image identity part of the T097 measurement method and requires fail-closed rejection before qualifying samples when that identity drifts. During exact-head T099 qualification on candidate `159663e2d57f286c5b21f5e19879f2fda451322a`, the `t097-performance` workflow did exactly that: the committed expected image version was `20260831.293.1`, while GitHub supplied `20260907.300.1`. The job failed before installing the toolchain, building the candidate, or collecting any qualifying performance sample.

This is external reference-environment drift, not evidence of a product performance regression. The prior pinned GitHub-hosted image cannot be selected by exact image version through the accepted `ubuntu-24.04` hosted-runner label, and Amendment 003 explicitly prohibits dynamically accepting the newly observed image or treating a failed identity check as qualifying evidence.

The simpler paths are therefore insufficient:

- reusing older T097 evidence would violate exact-candidate evidence discipline for T099/T100;
- rerunning the unchanged workflow would continue to fail closed while GitHub serves the new image;
- accepting `20260907.300.1` post hoc without a committed candidate change is explicitly prohibited by Amendment 003;
- relaxing FR-045..FR-053 or removing the image identity guard is prohibited and unnecessary.

This amendment authorizes only the smallest reversible governance response: a single forward update of the committed expected runner image version, followed by complete exact-candidate requalification under the unchanged frozen method and thresholds.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live canonical repository truth proves all of the following at use time:

- canonical `main` contains T098 `CLOSED_CANONICAL` at `df0d48992f37d430fa73271fee7936e118be612a` or a forward descendant;
- T097 remains canonically closed at `b7cb7c23701b46c278d142e51321c76af4ca8ec4` and Amendment 003 remains canonical;
- T099 is the active dependency-authorized implementation task and T100 remains blocked by T099;
- exact-head `t097-performance` evidence demonstrates a pre-sampling image identity mismatch from expected `20260831.293.1` to observed `20260907.300.1` on the explicit `ubuntu-24.04` runner;
- no qualifying FR-045..FR-053 samples from that mismatched run are represented as valid evidence.

At amendment creation, the observed mismatch is workflow run `34256729177`, job `102166423591`, where checkout identity matched T099 candidate `159663e2d57f286c5b21f5e19879f2fda451322a` and the job stopped at `Verify pinned reference environment before sampling` with:

```text
expected ImageOS=ubuntu24
expected ImageVersion=20260831.293.1
observed ImageOS=ubuntu24
observed ImageVersion=20260907.300.1
runner label=ubuntu-24.04
runner OS=Linux
runner arch=X64
```

These are historical observations supporting the governance decision only. They do not qualify a later candidate and must be reverified by the workflow after the committed refresh.

## Exact additional authority

After this amendment lands canonically and its post-merge verification succeeds, the active T099 candidate may make exactly one additional changed-path update:

```text
.github/workflows/t097-performance.yml
```

The only authorized semantic change in that workflow is:

```text
EXPECTED_IMAGE_VERSION: 20260831.293.1
```

to:

```text
EXPECTED_IMAGE_VERSION: 20260907.300.1
```

No other workflow field, runner label, permission, action pin, toolchain pin, command, fixture, threshold, sample count, artifact behavior, or acceptance rule gains authority from this amendment.

The refreshed T099 candidate MUST then rerun the complete `t097-performance` workflow on its exact final HEAD/TREE. Earlier T097/T098/T099 benchmark evidence remains historical and cannot substitute for the refreshed exact-candidate campaign.

If GitHub serves any image identity other than the newly predeclared `20260907.300.1`, the workflow MUST continue to fail closed before samples. This amendment does not authorize another automatic or dynamic refresh. A further image change requires another explicit governance decision before qualifying evidence can proceed.

T100 may reconcile the canonical T099 performance evidence produced under this refreshed pinned identity, but T100 gains no independent authority to modify production/runtime behavior. If T100 itself requires a current benchmark rerun and the pinned image has drifted again, the same fail-closed and governance discipline applies.

## Frozen requirements remain unchanged

This amendment does not modify or relax any Spec 007 requirement. In particular FR-045..FR-053 remain exactly frozen:

- cold/input-ready startup <=1500 ms p95 over >=20 runs;
- workbench-only input-to-dispatch overhead <=16 ms p95 over >=1000 iterations;
- 50 inert-pane topology operation p95 <=16 ms over >=1000 operations;
- >=100,000-line retained-history search p95 <=100 ms over >=200 representative searches;
- >=100,000 lines and >=10 MiB output navigation p95 <=100 ms for the specified measurement;
- per-pane retained transcript <=100,000 logical lines and <=32 MiB payload or tighter with visible eviction;
- ten idle live fixture panes over 60 seconds <=2% of one logical CPU core and <=256 MiB workbench RSS overhead excluding child memory;
- 1000 resize requests across ten fixture panes in ten seconds preserve final-size correctness;
- exact candidate/tree/environment/method identity and staleness semantics remain required.

Measured failures still require repair and requalification, never threshold relaxation.

## Authority and security boundary

This amendment changes no product capability and grants no execution authority beyond the already accepted read-only T097 measurement workflow. It introduces no dependency, schema, daemon/service, socket/RPC/IPC, provider/model/browser runtime, network benchmark fixture, remote execution, Git mutation authority, automatic winner, automatic acceptance, or automatic landing.

The existing workflow must retain:

- `permissions: contents: read`;
- `persist-credentials: false`;
- explicit `ubuntu-24.04` runner selection;
- explicit `ImageOS` and `ImageVersion` checks before sampling;
- repository-pinned Rust `1.97.1`;
- existing pinned actions;
- release-like `--locked` execution;
- local deterministic fixtures only;
- machine-readable exact-candidate evidence and artifact upload;
- fail-closed behavior for identity or threshold failure.

The decision is reversible in the sense that no product state or runtime authority changes: the expected image value can only move again through a future explicit governance decision if external runner identity drifts.

## Explicit non-authorization

This amendment does NOT authorize:

- `ubuntu-latest` or any unpinned runner identity;
- dynamic discovery followed by automatic acceptance of `ImageVersion`;
- wildcard/range matching of runner images;
- removing or weakening the pre-sampling image identity check;
- changing runner OS/architecture/label;
- changing Rust/toolchain/action pins;
- changing benchmark commands, fixtures, sample counts, thresholds, evidence schema, or performance claims;
- adding dependencies or benchmark frameworks;
- any production/runtime behavior change;
- any T100 production change;
- any Spec 008 or later roadmap work;
- treating historical workflow success or failure as current exact-candidate acceptance.

## Acceptance gate for this amendment

This governance-only amendment is not canonical merely because this file exists. The exact amendment candidate must satisfy:

- canonical base/race reverified immediately before landing;
- changed scope exactly this one amendment file;
- repository `quality` SUCCESS on the exact candidate;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review bound to the exact candidate;
- zero unresolved material findings and zero unresolved material review threads;
- no threshold, workflow, product, dependency, runtime, protocol, or authority change in the amendment PR itself;
- expected-head guarded merge;
- landed main/tree verification and applicable post-merge/push CI SUCCESS before the authority is used.

Only after those conditions are proven may T099 incorporate the exact workflow version refresh described above. T099 remains open until its own full exact-head Standard Acceptance Gate passes. T100 remains dependency-blocked until T099 closes canonically.
