# T100 — Spec 007 Final Closeout Addendum

Status: `IN_QUALIFICATION`

Canonical predecessor at addendum creation:

```text
BASE=8e65ff864958c9db3543ccf1cf018f36f6ae32e1
BASE_TREE=480b3ae2ac2418932a2df05abb079a0621234393
T099=CLOSED_CANONICAL
POST_T100_REPAIR_CHAIN=CLOSED_CANONICAL
T100=AUTHORIZED_FOR_FINAL_DOCUMENTATION_CLOSEOUT
SPEC_008_IMPLEMENTATION_AUTHORIZED=NO
```

This addendum is the final documentation/evidence-only reconciliation permitted by canonical T100 and the post-T100 repair amendments. It changes no production source, runtime behavior, dependency, lockfile, workflow, migration, performance threshold, terminal ownership rule, verification authority, Git authority, platform authority, provider/browser surface, daemon/IPC surface, or successor-specification authority.

The immutable SHA of this addendum candidate is intentionally not embedded here. Exact PR metadata, candidate-bound CI and reviews, guarded expected-head landing, canonical merge identity, and post-merge push checks bind final T100 closure without requiring a self-referential follow-up commit.

## 1. Relationship to the original T100 reconciliation

`specs/007-native-agentic-terminal-ux-foundation/t100-final-reconciliation.md` remains the canonical requirement-level reconciliation artifact. It classifies FR-001 through FR-066, reconciles SC-001 through SC-018, records dependency provenance, frozen FR-045 through FR-053 performance evidence, platform claim boundaries, Spec 006 live-runtime nonclaims, negative scope, and the exact-candidate closure discipline.

This addendum does not replace or weaken that reconciliation. It records the material post-landing evidence discovered after the original T100 closeout candidate landed and establishes the new final-closeout predecessor from which T100 may be requalified.

The original T100 checkbox in `tasks.md` therefore remains candidate-intent history rather than proof that T100 had already closed. Canonical closure still requires this final documentation/evidence-only candidate to pass the T100 Standard Acceptance Gate, land through an expected-head guard, and pass applicable post-merge push checks on canonical `main`.

## 2. Original T100 landing did not close T100

The original T100 closeout candidate was:

```text
PR=125
HEAD=ebb908c72c6da09fce7dbab5540ab3632c676516
TREE=522ea718cce2f3472a8eacd0c67342b25d91ac25
MERGE=6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7
```

The guarded merge itself was mechanically correct, but T100 did not become `CLOSED_CANONICAL` because its required first post-merge push `quality` run failed:

```text
RUN=34284403842
EVENT=push
HEAD=6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7
CONCLUSION=FAILURE
FAILED_DOMAIN=ubuntu
FAILED_TEST=git::t060_fault_tests::interrupt_then_close_escalates_only_while_session_is_still_owned
OBSERVED_STATUS=EXITED
OVER_STRONG_EXPECTATION=INTERRUPTED_ONLY
```

The failed run remains material first-attempt evidence. It was not rerun into acceptance and was not reclassified as a flake.

## 3. Amendment 006 and T060 truthful-close repair

Canonical Amendment 006:

```text
PR=126
HEAD=e8f395182a258559b5c151eb43d0085a2980dd4d
TREE=0430bf44722ad60add76e9b078972818f8d40075
MERGE=6c69a64560e37e0411bf5403b36e21a60e285cf3
POSTMERGE_QUALITY=34285524118 SUCCESS
```

Amendment 006 authorized only the existing T060 fixture `interrupt_then_close_escalates_only_while_session_is_still_owned` to distinguish the production lifecycle truths already implemented: natural `EXITED/PROCESS_EXITED`, controlled `INTERRUPTED/CLOSED_BY_WINDS`, and bounded fail-closed `OWNERSHIP_LOST/OWNERSHIP_LOST_PROCESS_STATE_UNKNOWN`. The canonical 500 ms production terminal cleanup window and all production lifecycle mappings remained unchanged.

The authorized repair landed as:

```text
PR=127
HEAD=70de319a55e022ca233d6ce8c2555d0f296854d0
TREE=f9e113cdfb24d3aa3fd68f2fb8a13b720639b720
MERGE=6fb8782bea3831e7b8f94902082783631bcaf8ad
POSTMERGE_QUALITY=34289087688 SUCCESS
POSTMERGE_WINDOWS_TERMINAL=34289087658 FAILURE
```

The separate first-attempt `windows-terminal` failure remained material. Ubuntu terminal, native Windows, and real Windows plus Ubuntu WSL2 passed; only macOS failed at T096 native workbench qualification with the existing bounded-cleanup error. No rerun-to-green was used.

## 4. Amendment 007 and truthful T096 native close repair

Canonical Amendment 007:

```text
PR=128
HEAD=84248f77153341f2c84c648af70c3a192593865b
TREE=0b0ff858a771c0e4916f2c883b0f2a93d5f8b028
MERGE=50e941e76950f71aa49ca00202d956bd93243351
POSTMERGE_QUALITY=34290856420 SUCCESS
```

Amendment 007 authorized only the native T096 fixture to accept either proven `Exited` or bounded-unproven `OwnershipLost` after ownership revocation. It did not alter the 500 ms production cleanup bound, cleanup ordering, ownership revocation, WSL2 fixture, workflow, dependency, schema, performance threshold, or runtime/Git authority.

The authorized repair landed as:

```text
PR=129
HEAD=21d23211395b2c8d06b8a533078f4c823002287d
TREE=36eced35139ad4d7bb3433add1fcc0b73b68d09d
MERGE=1ba8ebf6202648476fccf0ebfb713fe69b3dca11
POSTMERGE_QUALITY=34291818680 SUCCESS
POSTMERGE_WINDOWS_TERMINAL=34291818718 FAILURE
```

The post-merge `windows-terminal` failure was again preserved as first-attempt evidence. Ubuntu terminal, macOS terminal, real Windows plus Ubuntu WSL2, and the repaired T096 native-Windows fixture passed. The sole failure was the native-Windows full Spec 003 touched-surface suite at `git::process_scope::tests::surviving_descendant_is_detected_and_terminated_as_owned_scope`, where the direct PowerShell bootstrap was not observed exited inside the fixture-local five-second observation budget.

## 5. Amendment 008 and Windows process-scope observation repair

Canonical Amendment 008:

```text
PR=130
HEAD=e1305dbe3aacff7884fe4429cad4f0c13eb05fbf
TREE=c7f91e05467abd58989a070447247c79d2e950b4
MERGE=39a31416847b3ab4b68071a0a8cb4a84d39d5170
POSTMERGE_QUALITY=34293420504 SUCCESS
```

Amendment 008 authorized only the existing Windows test fixture's direct-child observation budget to move from five seconds to fifteen seconds. Linux remained five seconds. The Windows 30-iteration `ping.exe` descendant, Linux `sleep 30 &` descendant, both 100 ms scope observations, the two-second `terminate_and_prove` deadline, Job Object containment, ownership accounting, and all production cleanup semantics remained unchanged.

The exact repair candidate was:

```text
PR=131
HEAD=c4435ecd93a62a98d705996236be06d1266ba2f0
TREE=480b3ae2ac2418932a2df05abb079a0621234393
QUALITY=34293993870 SUCCESS
WINDOWS_TERMINAL=34293993876 SUCCESS
RELEASE_CANDIDATE=34293993899 SUCCESS
```

The exact-head `windows-terminal` run passed Ubuntu, macOS, native Windows, and real Windows plus Ubuntu WSL2. The native-Windows `Full Spec 003 touched-surface tests` step passed, directly exercising the repaired process-scope fixture. Author correctness/safety/governance/evidence-integrity and Ponytail/YAGNI reviews passed, fresh independent substantive review reported no material source finding, and zero review threads remained.

The guarded landing was:

```text
MERGE=8e65ff864958c9db3543ccf1cf018f36f6ae32e1
TREE=480b3ae2ac2418932a2df05abb079a0621234393
PARENT_1=39a31416847b3ab4b68071a0a8cb4a84d39d5170
PARENT_2=c4435ecd93a62a98d705996236be06d1266ba2f0
GITHUB_SIGNATURE=VALID
POSTMERGE_QUALITY=34295540574 SUCCESS
POSTMERGE_WINDOWS_TERMINAL=34295540423 SUCCESS
```

The final post-repair canonical `windows-terminal` push passed Ubuntu terminal integration, macOS terminal integration, native-Windows full Spec 003 touched-surface tests, native Windows ConPTY lifecycle, T096 native Windows qualification, and real Windows plus Ubuntu WSL2 T062/T096 qualification.

## 6. Historical failure discipline

These failures remain part of the canonical evidence chain:

```text
34284403842=FAILURE
34289087658=FAILURE
34291818718=FAILURE
```

None is erased, rerun into acceptance, or relabeled as a flake. Each failure caused the program to remain open, produced an explicit bounded governance amendment, and was followed by a forward-only repair on a new exact candidate.

Local macOS diagnostic failures observed while preparing the Amendment-008 repair were explicitly non-qualifying. A separate untouched-canonical-main control on the same host also failed under ambient external `git-ai 1.5.2` instrumentation that wrote `remote.git/ai/...` into a Git fixture. Those local diagnostics did not waive, replace, or weaken clean exact-head repository CI.

## 7. Requirement and success-criterion reconciliation remains intact

The classifications in `t100-final-reconciliation.md` remain applicable because Amendments 006 through 008 and their repairs changed only test expectations or test-local observation budget within existing production truth. They introduced no new product behavior and relaxed no Spec 007 FR/SC requirement.

In particular:

- FR-045 through FR-053 performance thresholds remain frozen and unchanged;
- final implementation-head performance provenance remains T099 `t097-performance` run `34274620724` and artifact `10075321553` on `0875333b16c9c2c1838051c1bcafd6f6f56f8f1b`;
- the later test-only repair chain does not replace that performance provenance and introduces no performance implementation change;
- native Windows, WSL2, Linux, and macOS claims remain limited to directly exercised domains;
- Spec 006 live-runtime nonclaims remain separate and unchanged;
- no daemon/IPC, remote execution, browser/provider runtime, ACP/MCP runtime, generic plugin system, learning subsystem, vector/RAG memory, custom PTY, automatic winner, or automatic landing path was introduced.

SC-016 and SC-017 are not predeclared satisfied by this file. They become satisfied only if the exact final documentation/evidence-only candidate passes its own required CI/review stack and then lands and passes applicable post-merge push checks.

## 8. Final T100 candidate gate

This addendum is documentation/evidence-only. Focused implementation tests are N/A because the candidate changes no implementation surface. Repository/applicable workflows remain mandatory according to actual path triggering and canonical T100 governance.

Before T100 may become `CLOSED_CANONICAL`, the exact final candidate must prove all of the following:

- the candidate starts from canonical `8e65ff864958c9db3543ccf1cf018f36f6ae32e1` / tree `480b3ae2ac2418932a2df05abb079a0621234393` or is reconciled forward-only if `main` moves;
- changed scope remains only T100-authorized documentation/evidence surfaces;
- no production source, Cargo/lockfile, workflow, migration, runtime, dependency, protocol, provider/browser, daemon/IPC, remote, learning, plugin, ACP/MCP, performance threshold, or automatic-landing behavior changes;
- exact-head repository `quality` succeeds;
- every other applicable workflow actually triggered for the candidate succeeds;
- author correctness/safety/governance/evidence-integrity review passes on the exact final head;
- Ponytail/YAGNI review passes on the exact final head;
- fresh independent substantive review reaches the exact final head with zero unresolved material findings;
- zero unresolved material review threads remain;
- final main/base/head/tree/scope/ruleset/mergeability race reconciliation succeeds;
- landing uses an explicit expected-head guard and merge method `merge`;
- canonical merge main/tree/parents are verified;
- all applicable post-merge push checks actually triggered by the closeout landing succeed.

Only after those external candidate-bound gates are proven may repository truth state:

```text
T087..T100=CLOSED_CANONICAL
SPEC_007_SPEC=CLOSED_CANONICAL
SPEC_007_PLAN=CLOSED_CANONICAL
SPEC_007_TASKS=CLOSED_CANONICAL
SPEC_007_FIRST_IMPLEMENTATION_PROGRAM=CLOSED_CANONICAL
```

## 9. Successor authority

Closing T100 authorizes no successor implementation task.

No `specs/008-*` implementation program is canonically authorized by this closeout. Research RFC #91 explicitly identifies its proposed future sequence as non-canonical and non-authorizing. Research-only roadmap or draft surfaces may inform a later governance decision but cannot be treated as implementation authority.

Accordingly, successful T100 closeout means the Spec 007 first implementation program is closed. It does not mean every future Winds roadmap idea has been implemented, and it does not permit starting Spec 008 without a separate canonical entry/specification decision.
