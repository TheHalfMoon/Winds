# Spec 011 T151 owner process shell provenance

## Scope

T151 adds only the same-binary internal owner shell, per-home/principal singleton arbitration, generation/store reconciliation ordering, secure endpoint readiness, and mandatory 300-second zero-activity exit state.

No persistent child process, PTY/ConPTY ownership migration, provider launch, renderer integration, service manager installation, login/autostart behavior, PID authority, or remote transport is introduced.

## Same-binary internal mode

The shipped `winds` binary recognizes the intentionally undocumented internal command `__winds-internal-owner-v1 --home ABS_PATH`. The command is absent from public usage text and is not a general daemon interface.

## Singleton boundary

- POSIX: canonical Winds home -> private `persistent-runtime/` directory (`0700`) -> regular `owner.lock` (`0600`, `O_NOFOLLOW`, inode/path identity checked) -> nonblocking kernel `flock(LOCK_EX|LOCK_NB)`. File persistence is not authority; the kernel-held lock is authority and is released on descriptor close/process exit.
- native Windows: canonical Winds home -> current-user protected `persistent-owner.lock` with a protected one-user file-object DACL -> `CreateFileW` share mode zero. The open handle is authority; a stale file without a live handle is reusable. Effective owner/DACL are read back using the already-qualified T150 SID/DACL seam.

No PID is written or consulted for singleton authority.

## Startup order

The implementation order is fixed:

1. canonicalize/create Winds home and acquire singleton;
2. create an owner-generation ID through the qualified T149/T150 OS entropy seam;
3. open/migrate Store, record the generation, and reconcile prior live rows;
4. bind the already-qualified private local endpoint;
5. construct the ready owner state.

No ready owner value exists if Store reconciliation or endpoint binding fails.

## Idle exit

Production `OWNER_IDLE_GRACE_MS` is exactly `300_000`. The owner may exit only when both connected-client count and live-runtime count are zero for the full grace period. Any connected client or live runtime suppresses idle exit immediately. No protocol message or wire command is imported as an idle-shutdown authority.

T151 does not yet accept protocol work into an authority loop and does not own any child process. T152 remains responsible for moving/reusing the accepted PTY/ConPTY primitive under the owner.

## Review policy

Alibaba OpenCodeReview delegation mode is the independent review mechanism. CodeRabbit is excluded by explicit user direction.
