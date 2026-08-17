# Workspace Terminal Trust Boundary

This document records the Spec 003 trust boundary for workspace and terminal execution.

## Core rule

Winds workspace terminals and explicit `winds run` commands execute as the user who launched Winds. They are developer-workspace activity, not a security sandbox and not an authoritative verification environment.

A command launched through a Winds workspace terminal may therefore do anything the launching user could do through an ordinary terminal, subject only to operating-system, account, container, network, and other controls that already exist outside Winds.

That may include:

- reading, creating, modifying, or deleting files that the launching user can access;
- mutating the primary checkout, including tracked and untracked files;
- invoking Git and other installed executables;
- accessing network destinations available to the launching user;
- reading environment variables, credentials, tokens, SSH material, cloud credentials, package-manager credentials, or other secrets available to the launched process;
- starting child processes or tools that themselves have the launching user's permissions.

Winds does not claim that PTY/ConPTY ownership, workspace identity, WSL execution-domain selection, command history controls, or execution-ledger persistence provides OS, filesystem, network, credential, or hostile-code isolation.

## Workspace activity may mutate the primary checkout

Interactive terminal activity and explicit commands are user-directed workspace behavior. They may intentionally mutate the primary checkout.

Winds may record before/after Git observations around supported command boundaries, but those observations describe workspace history only. A mutation does not become verified merely because Winds observed it, persisted it, or can show it in the execution ledger.

In particular, terminal or command activity MUST NOT become `ELIGIBLE` verification evidence merely because it appears in local execution history.

## Execution records are not verification evidence

The execution ledger records typed local facts about supported activity, including exact workspace identity, execution domain, lifecycle state, timing, profile identity, command metadata where supported, and Git observations where available.

Source labels remain meaningful:

- `WINDS_OBSERVED` means Winds directly observed that specific process, lifecycle, timing, or Git fact through an accepted observation path;
- caller-entered commands, profile choices, clone destinations, and similar requested values remain caller intent;
- shell-reported lifecycle or command telemetry remains shell-reported unless Winds independently proves the same fact;
- output bytes may be observed by Winds, but claims contained in terminal output are still claims made by the producing process.

Persistence does not elevate any of these records into authenticated-human authority, candidate eligibility, promotion authority, or verification evidence.

## `winds verify` remains the authoritative verification path

`winds verify` is deliberately separate from workspace-terminal execution.

The existing verification path binds evidence to the exact verification candidate/base under its accepted candidate-worktree and evidence rules. Workspace terminal history and execution-ledger records do not bypass or replace those rules.

Here, verification isolation means separation of the candidate worktree and evidence path from ordinary workspace activity. It does not mean that `winds verify` is an OS, filesystem, network, or credential sandbox.

Therefore:

- a successful workspace command is not a successful verification run;
- a zero exit code recorded in workspace history is not candidate eligibility evidence;
- terminal output saying that tests passed is not authoritative proof that required checks passed;
- an observed Git mutation in the primary checkout is not a verified candidate snapshot;
- a workspace execution cannot make a candidate `ELIGIBLE` or promotable merely by being recorded;
- `winds verify`, `winds promote`, and `winds recover` retain their existing evidence semantics unless a later specification explicitly changes them.

## Repository configuration is trust-sensitive

Opening a workspace may inventory selected project/environment manifests, but Winds does not automatically execute repository configuration merely to discover the workspace.

Once the user launches a shell, command, package manager, build system, agent CLI, or other executable, that program may itself read or execute repository configuration according to its own behavior. This is normal user-driven terminal execution and remains inside the launching user's trust boundary.

Likewise, system Git clone may use user-configured Git behavior such as credential helpers or filters. Winds does not claim hostile-clone sandboxing because the clone was initiated through Winds.

## Secrets and history

Winds applies the accepted Spec 003 local history and metadata controls, including bounded retention, explicit transcript policy, conservative sanitization, and the ability to disable supported command/transcript history.

These controls reduce unnecessary persistence; they are not a guarantee that a command cannot access or disclose a secret. A launched process may access any secret available to that process, and no secret detector can prove that arbitrary command text or output is secret-free.

Users should disable history when the supported local-history policy is inappropriate for a sensitive session and should rely on external OS/container/credential controls when stronger isolation is required.

## PTY ownership is lifecycle ownership, not security isolation

When Winds owns a PTY/ConPTY session, that ownership allows Winds to perform only the accepted lifecycle operations it can prove for that session, such as input/output handling, resize, observed exit, and bounded terminate/close behavior.

It does not mean Winds owns or confines every descendant process, every filesystem effect, every network connection, or every credential that the terminal process may reach.

After a Winds restart, a persisted session whose continuing process ownership cannot be proven is reconciled conservatively as ownership-lost/unknown. A stored PID is not sufficient identity and is not a basis for blind signaling or destructive cleanup.

## WSL and execution domains

Native Windows and WSL distributions are distinct execution domains. Mapping a workspace path into WSL is not itself proof of identical repository identity; Winds only claims equivalence where the accepted mapping and Git-identity checks prove it.

Execution-domain selection does not add sandboxing. A WSL process has the permissions and reachable resources provided by that WSL environment and its host integration.

## What Winds does and does not prove

| Surface | Winds may prove | Winds does not thereby prove |
| --- | --- | --- |
| Workspace identity | Canonical repository/worktree identity and accepted Git observations | That workspace code is safe or verified |
| PTY/ConPTY lifecycle | Accepted directly observed lifecycle facts for the session Winds owns | OS/network/secret isolation or complete descendant ownership |
| Explicit command execution | Requested command plus accepted lifecycle/exit/Git observations | That the command's claims or produced code are correct |
| Local history | Bounded retained history and its metadata under the selected policy | That retained content is secret-free or verification evidence |
| `winds verify` | Evidence produced under the accepted verification path for the exact candidate/base | Authorization to weaken candidate, evidence, or promotion rules |

## Scope boundary

This document describes existing Spec 003 behavior. It does not authorize a daemon, persistent detached terminal owner, public IPC/runtime protocol, remote execution service, plugin/provider runtime, MCP/ACP/A2A, Agent Fleet, broad sandbox framework, or Herdr/Pi transplant.

Any future runtime that introduces stronger isolation or long-lived agent/session ownership must specify and prove that boundary independently rather than inheriting a sandbox claim from the current workspace terminal.
