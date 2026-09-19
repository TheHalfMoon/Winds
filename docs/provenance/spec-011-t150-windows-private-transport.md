# Spec 011 T150 native-Windows private transport provenance and security record

## Scope

T150 introduces only the native-Windows named-pipe transport/security seam required by `specs/011-persistent-agent-runtime-private-local-control/tasks.md`.

No persistent owner process, PTY/ConPTY ownership migration, renderer transport, TCP/HTTP/WebSocket listener, remote transport, service installation, workflow scheduler, or provider execution is introduced.

## Dependency qualification

Direct dependency: `windows-sys = 0.61.2` under `cfg(windows)` only.

- source: crates.io / Microsoft `windows-rs`;
- lockfile checksum: `ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc`;
- license: MIT OR Apache-2.0;
- crate-declared MSRV: Rust 1.71, below Winds Rust 1.97.1;
- package graph impact: the package already existed transitively; direct admission adds only the Winds root edge and retains its existing `windows-link` dependency;
- direct dependency uses `default-features = false`;
- exact direct features: `Win32_Security_Authorization`, `Win32_Security_Cryptography`, `Win32_Storage_FileSystem`, `Win32_System_IO`, `Win32_System_Pipes`, `Win32_System_Threading`;
- generated parent features (`Win32`, `Win32_Foundation`, `Win32_Security`, `Win32_Storage`, `Win32_System`) are implied by those child features; T150 does not name redundant parent/umbrella features directly;
- pre-existing `serial2 -> portable-pty` feature unification already activates `Win32_Devices_Communication`, `Win32_System_Registry`, and `Win32_System_WindowsProgramming`; T150 does not introduce those features and does not use them;
- no second Windows abstraction crate;
- the existing Tauri host lockfile (`desktop/src-tauri/Cargo.lock`) records the root path package dependency edge and therefore receives exactly one derived `windows-sys 0.61.2` dependency reference; no desktop package entry, version, checksum, or feature package is added or changed;
- removal path: remove the target-specific dependency and T150 Windows modules if the named-pipe seam is replaced by a separately accepted implementation.

The feature set is derived from compiled calls only: named-pipe create/connect/disconnect, synchronous file I/O, current-process/thread token SID inspection, SDDL conversion/effective DACL inspection, anonymous/named-pipe impersonation, and `BCryptGenRandom`.

## Endpoint and principal boundary

The endpoint is a native local named pipe only. The deterministic name binds canonical raw current-user SID bytes (lowercase hex, bounded by `SECURITY_MAX_SID_SIZE`) and the expected owner-generation ID. This avoids unbounded textual SID expansion while preserving exact principal identity. The pipe is created with `FILE_FLAG_FIRST_PIPE_INSTANCE` and `PIPE_REJECT_REMOTE_CLIENTS`.

The security descriptor is explicit SDDL with a protected DACL containing exactly one file-object `FILE_ALL_ACCESS` allow ACE for the current accepted user SID. No Everyone, Authenticated Users, Administrators, or default permissive DACL is admitted. The created kernel object's effective owner/DACL are read back and validated before use.

On server accept, `ImpersonateNamedPipeClient` plus the thread token's `TokenUser` SID proves the connected client is the same accepted user before the connection is admitted. `RevertToSelf` failure is a hard error.

Official-Windows tests also impersonate the anonymous principal and prove `CreateFileW` is denied by the actual pipe DACL. This is direct denial evidence for a non-accepted principal without claiming arbitrary cross-account administration capabilities on the hosted runner.

Same-user malicious-code isolation remains explicitly unclaimed.

## Collision and generation semantics

- same exact SID + generation pipe name collision fails closed through `FILE_FLAG_FIRST_PIPE_INSTANCE`;
- a client connects only to the exact expected generation-derived name;
- a missing/wrong generation is unavailable rather than silently falling back to another pipe;
- named-pipe kernel lifetime removes stale names when the final handle closes; T150 does not add a PID/file recovery mechanism;
- T151 remains responsible for owner-process singleton arbitration and lifecycle.

## Entropy boundary

Native-Windows runtime namespace and owner-generation IDs use exactly one 16-byte `BCryptGenRandom` call with `BCRYPT_USE_SYSTEM_PREFERRED_RNG`. API failure, short injected result, or all-zero bytes fail closed before identity creation.

## Evidence policy

T150 requires official native-Windows compile/tests. macOS/Linux local checks can prove unchanged non-Windows behavior and formatting only; they are not evidence for the Windows DACL, named-pipe, SID, or BCrypt claims.

CodeRabbit is excluded by explicit user direction. Review uses Alibaba OpenCodeReview delegation mode plus exact-head GitHub CI.
