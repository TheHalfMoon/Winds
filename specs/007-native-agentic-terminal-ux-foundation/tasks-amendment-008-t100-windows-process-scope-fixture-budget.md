# Spec 007 Tasks Amendment 008 — T100 Windows Process-Scope Fixture Observation Budget

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0 governance deviation/amendment process, canonical Spec 003 T068 process-scope containment truth, canonical Spec 007 T100 post-merge closure requirements, canonical Amendments 006 and 007, and the Founder directive in the active project session to continue the authorized Winds program without fabricating, suppressing, or rerunning away material evidence.

## Purpose

The Amendment-007-authorized T096 repair landed canonically as merge commit `1ba8ebf6202648476fccf0ebfb713fe69b3dca11` from exact reviewed head `21d23211395b2c8d06b8a533078f4c823002287d` / tree `36eced35139ad4d7bb3433add1fcc0b73b68d09d` against canonical base `50e941e76950f71aa49ca00202d956bd93243351`.

Post-merge `quality` run `34291818680` succeeded. The separately triggered post-merge `windows-terminal` run `34291818718` failed on the same exact canonical merge and therefore remains material first-attempt evidence. It MUST NOT be rerun or reclassified as a flake to obtain closure.

Exact post-merge result:

```text
RUN=34291818718
EVENT=push
HEAD=1ba8ebf6202648476fccf0ebfb713fe69b3dca11
TREE=36eced35139ad4d7bb3433add1fcc0b73b68d09d
CONCLUSION=FAILURE
UBUNTU_TERMINAL=SUCCESS
MACOS_TERMINAL=SUCCESS
REAL_WINDOWS_WSL2=SUCCESS
NATIVE_WINDOWS_TERMINAL=FAILURE
FAILED_JOB=102279796752
FAILED_STEP=Full Spec 003 touched-surface tests
```
The failing native-Windows full-suite result was:

```text
running 384 tests
378 passed; 1 failed; 5 ignored

FAILED:
git::process_scope::tests::surviving_descendant_is_detected_and_terminated_as_owned_scope

assertion failed:
wait_for_direct_exit(&mut process, Instant::now() + Duration::from_secs(5))
```

The repaired T096 native Windows fixture passed in that same failed job. Format, compile, and Clippy also passed before the full-suite failure. Ubuntu, macOS, and real Windows + Ubuntu WSL2 jobs passed on the same canonical merge.

The same source tree had already passed exact-head `windows-terminal` run `34291125022` and exact-head `release-candidate` run `34291124806` before guarded landing. Those earlier successes do not erase the failed post-merge attempt.

## Diagnosis

The failing assertion is a test-only observation deadline, not a production cleanup or containment deadline.

On Windows, the fixture starts `powershell.exe`, which launches a `ping.exe` descendant configured for 30 iterations and then exits. `spawn_owned_process(...)` creates a Windows Job Object, starts the direct child suspended, assigns it to the Job Object, and only then resumes it. Production cleanup and ownership proof use the existing Job Object accounting and termination path.

The fixture first waits for the direct PowerShell process to exit, then proves that the descendant keeps the owned scope live, then calls `terminate_and_prove(...)` and proves quiescence. The observed failure occurred before the cleanup proof: the direct PowerShell bootstrap did not complete inside the fixture's five-second observation window.
Canonical Spec 003 and Spec 007 do not define five seconds as a required direct-child startup/exit threshold for this fixture. The five-second value was introduced with the T068 regression itself. Increasing the Windows-only observation window therefore does not weaken the production ownership proof, terminate-and-prove deadline, or platform claim.

## Mandatory predecessor and live-truth gate

This amendment is inert unless live repository truth continues to prove all of the following:

- canonical `main` is `1ba8ebf6202648476fccf0ebfb713fe69b3dca11` or a governance-only forward descendant;
- post-merge `quality` run `34291818680` remains SUCCESS on that exact merge;
- post-merge `windows-terminal` run `34291818718` remains preserved as FAILURE and is not replaced by rerun-to-green evidence;
- its only failed job is native-Windows job `102279796752`, with the failure above inside the full Spec 003 touched-surface suite;
- the repaired T096 native-Windows fixture passed in that same failed job;
- production Windows Job Object containment and cleanup semantics remain unchanged;
- no later canonical repair has already removed the need for this amendment.

No Spec 008 or later-roadmap implementation is authorized while T100 remains open.

## Exact additional repair authority

Only after this amendment is canonically landed may the T100 post-merge repair modify:

```text
src/process_scope.rs
```

and only inside:

```rust
#[cfg(any(target_os = "linux", windows))]
#[test]
fn surviving_descendant_is_detected_and_terminated_as_owned_scope()
```
The authorized repair is limited to making the direct-child observation budget platform-explicit:

- Windows: change only this fixture's direct-child exit observation from 5 seconds to 15 seconds;
- Linux: retain the existing 5-second observation budget;
- retain the Windows 30-iteration `ping.exe` descendant command unchanged;
- retain the Linux `sleep 30 &` descendant command unchanged;
- retain the 100 ms non-quiescence observation unchanged;
- retain the 2-second `terminate_and_prove(...)` cleanup deadline unchanged;
- retain the final 100 ms quiescence proof unchanged.

A valid implementation may introduce only the smallest local test variable or `cfg` expression required to select 15 seconds on Windows and 5 seconds on Linux. It MUST NOT alter helper semantics or production code.

## Values and behavior that MUST remain unchanged

This amendment does **not** authorize changing:

- `MAX_CLEANUP_RESERVE` or `POLL_INTERVAL`;
- `OwnedProcess`, `WindowsJob`, Job Object assignment, accounting, termination, or `KILL_ON_JOB_CLOSE` behavior;
- `spawn_owned_process(...)`, suspended-child assignment, or primary-thread resume semantics;
- `wait_for_scope_quiescence(...)` or `terminate_and_prove(...)` production semantics;
- the direct-child wait helper implementation;
- any other process-scope fixture;
- any T096 fixture, including the Amendment-007-repaired fixture;
- workflows, runner images, retry policy, dependencies, lockfiles, schema, migrations, performance thresholds, platform authority, runtime authority, or Git authority;
- any Spec 003 or Spec 007 FR/SC requirement.

No retry loop, rerun-based acceptance, ignored-test marker, platform skip, production timeout widening, assertion deletion, or ownership-proof weakening is authorized.
## Required evidence after amendment landing

The governance amendment itself must satisfy the repository Standard Acceptance Gate before use.

After the amendment is canonical, the repair must start from the then-current exact `main` and must be requalified from scratch on its exact final head. Required evidence includes:

- repository `quality` SUCCESS on Ubuntu and macOS;
- exact repaired process-scope fixture execution and PASS on native Windows;
- full existing native-Windows touched-surface suite green;
- `windows-terminal` SUCCESS across Ubuntu, macOS, native Windows, and real Windows + Ubuntu WSL2;
- `release-candidate` SUCCESS if triggered or required by canonical gate;
- no production or performance implementation change;
- author correctness/safety/governance/evidence-integrity review PASS;
- Ponytail/YAGNI review PASS;
- fresh independent substantive review on the exact repair candidate, or a reaching review stack permitted by the canonical Standard Acceptance Gate;
- zero unresolved material findings and review threads;
- exact changed-path/function reconciliation proving only the authorized fixture timing expression changed;
- `behind_by=0`, ruleset/mergeability/main-race revalidation;
- guarded expected-head landing;
- successful applicable post-merge `quality` and `windows-terminal` checks on repaired canonical main.

Historical failed runs `34284403842`, `34289087658`, and `34291818718` remain material evidence and MUST NOT be erased, rerun into acceptance, or reclassified as flakes.

## T100 closure after repair

Landing this amendment does not close T100. Landing the fixture-budget repair does not by itself close T100.
Only after the repaired canonical main passes all applicable post-merge checks may repository truth perform the final T100 documentation/evidence reconciliation. That reconciliation must preserve:

- guarded T100 merge `6a4476e3ea33a82ff8dfc0df11fcc6d1493122c7`;
- failed original T100 post-merge `quality` run `34284403842`;
- canonical Amendments 006, 007, and 008;
- failed post-repair `windows-terminal` runs `34289087658` and `34291818718`;
- exact repair candidates and guarded merge evidence;
- successful post-repair canonical push evidence;
- no successor implementation authority beyond then-canonical governance.

A final closure record may modify or add only Spec 007 documentation/evidence surfaces necessary to state this complete chronology truthfully. It may not alter production behavior or silently authorize Spec 008.

## Explicit non-authorization

This amendment does not authorize rerunning failed run `34291818718` as a substitute for repair, flake classification, production timeout widening, changing process ownership semantics, weakening cleanup proof, retries, ignored tests, platform skips, workflow weakening, dependency/schema/performance changes, daemon/IPC, provider/model/browser/network runtime, remote execution, automatic Git mutation/selection/landing, Spec 008, or any later roadmap implementation.

## Acceptance of this amendment

This file is governance-only and is not canonical merely because it exists.

The exact amendment candidate must satisfy the repository Standard Acceptance Gate applicable to governance-only changes: repository `quality` SUCCESS, correctness/safety/governance/evidence-integrity author review, Ponytail/YAGNI review, fresh independent substantive review or permitted reaching review stack, zero unresolved material findings/threads, exact one-file scope reconciliation, live main/ruleset/mergeability race reconciliation, guarded `expected_head_sha` landing, and post-merge canonical main/tree plus applicable push-CI verification.

Only after successful canonical landing may the exact Windows-only test-observation repair authority above be used.
