# Spec 011 T149 POSIX private local transport provenance and security record

## Scope

T149 introduces only the Linux/macOS Unix-domain transport and POSIX peer/entropy seams required by `specs/011-persistent-agent-runtime-private-local-control/tasks.md`.

No Windows named pipe, persistent owner process, PTY/ConPTY ownership migration, renderer transport, TCP/HTTP/WebSocket listener, provider execution, dependency, lockfile, migration, or remote transport is introduced.

## Endpoint model

- transport: filesystem Unix-domain stream socket only;
- deterministic endpoint filename: `owner.sock`;
- preferred user runtime directory: platform user-runtime base plus `winds-runtime-v1`;
- conservative portable socket-path ceiling: 100 pathname bytes;
- deterministic overlength fallback: `/tmp/winds-runtime-<effective-uid>`;
- resolver canonicalizes the selected existing base before appending the runtime directory; explicit bind/connect paths reject ancestor path aliases;
- runtime directory must be an absolute, non-symlink directory owned by the effective UID with exact mode `0700`;
- endpoint must be a non-symlink socket owned by the effective UID with exact mode `0600`;
- regular-file and symlink collisions fail closed and are preserved;
- a connectable existing socket is treated as live and is never replaced;
- an existing socket is removed as stale only after `connect(2)` returns `ECONNREFUSED`, its socket/owner/mode facts are revalidated, and its device/inode identity still matches the pre-probe identity;
- listener cleanup removes only the same device/inode socket it created, so a replacement path is not deleted by drop cleanup.

This protects against cross-user pathname substitution under the accepted local-principal threat model. Spec 011 explicitly does not claim isolation from arbitrary malicious code already executing as the same effective OS user.

## Principal proof

Linux uses `getsockopt(SOL_SOCKET, SO_PEERCRED)` through the already accepted direct `libc = 0.2.189` dependency. macOS uses `getpeereid(3)` through the same libc boundary. Both results are compared to the current process effective UID before a stream is admitted.

Linux evidence and macOS evidence remain separate; a result on one platform is not presented as proof for the other.

## Entropy proof boundary

The T146 contract is implemented without a random/UUID dependency:

- Linux: one `getrandom(2)` request for exactly 16 bytes;
- macOS: one `getentropy(3)` request for exactly 16 bytes;
- syscall failure, short result, or all-zero result fails closed;
- accepted bytes feed the existing `RuntimeNamespaceId::from_entropy_bytes` / `OwnerGenerationId::from_entropy_bytes` constructors.

Native-Windows entropy remains T150 scope and is not implemented or inferred here.

## Evidence policy

Focused tests exercise same-user kernel peer proof, synthetic wrong-UID rejection, secure directory creation, mode/owner validation, live socket preservation, stale socket replacement after refused connect, file/symlink collision preservation, deterministic path fallback, entropy failure semantics, and absence of network/Windows/process-owner surfaces.

Real cross-UID peer rejection is recorded only where the CI/runtime environment permits creation of a genuinely different effective UID. A synthetic UID mismatch test does not get relabelled as real cross-user kernel evidence.
