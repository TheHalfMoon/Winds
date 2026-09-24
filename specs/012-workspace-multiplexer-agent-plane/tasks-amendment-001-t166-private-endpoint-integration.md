# Spec 012 Tasks Amendment 001 — T166 Existing Private-Endpoint Integration

Status: CANDIDATE UNTIL GUARDED LANDING

Authority basis: Winds Constitution 1.1.0, canonical Spec 012 Plan decisions AD-012-08 through AD-012-10, and canonical T166 purpose/acceptance requirements.

## Material contradiction

Canonical T166 requires the private protocol to expose topology authority through the already-qualified private transport, requires the exact v2 handshake on both owner and shared Rust client, and requires every existing Spec 011 runtime-control client operation to round-trip under v2. The accepted T165 closeout already authorizes T166 and records the successful post-merge qualification; this amendment does not alter that predecessor decision.

The accepted implementation-path list named `src/persistent_runtime/owner.rs` only for typed dispatch and omitted the existing `src/persistent_runtime/transport/unix.rs` and `src/persistent_runtime/transport/windows.rs` session-lifecycle adapters. Those adapters own platform endpoint/session lifecycle and OS-level I/O, including authenticated accept, byte reads/writes, Windows peer proof, and next-client readiness; the existing `protocol.rs` remains the sole owner of frame encoding, decoding, validation, and size ceilings. A production owner service loop cannot be implemented safely on Linux, macOS, and native Windows by changing `owner.rs` alone.

This is a path-authority omission, not authority to add a second server, endpoint, transport, dependency, or public RPC surface.

## Narrow correction

T166 additionally permits narrowly additive authenticated-session lifecycle support in the existing Unix and Windows private transports, solely so `owner.rs` can service the endpoint it already binds:

- bounded/nonblocking same-user accept without stopping runtime polling;
- exact-byte OS session reads/writes and response/event writes on the accepted session;
- explicit disconnect cleanup;
- Windows disconnect and re-arm for a next client;
- preservation of existing effective-user/endpoint owner/mode or SID/DACL checks;
- no endpoint rename, new listener type, TCP/public socket, remote transport, dependency, or alternate owner.

The correction does not authorize detector execution, attention aggregation, Git/worktree mutation, T167 client projection/cache, T168+ UI, provider execution, or a second process-ownership authority.

## Required qualification

The corrected T166 candidate must prove separately through each directly exercised native Linux, macOS, and native-Windows platform:

1. same-user v2 HELLO over the real private endpoint;
2. exact Spec 011 runtime-control operations round-trip through the real owner dispatcher;
3. topology list/read/write-state dispatch remains separate from Runtime Controller authority;
4. malformed, stale, duplicate, and unsupported messages fail closed;
5. disconnect revokes only that connection's controller/observer state;
6. a next client can connect after an ordinary disconnect;
7. runtime polling and bounded replay/gap behavior continue while the endpoint is serviced;
8. all applicable exact-head Linux, macOS, and native-Windows checks pass.

Candidate movement invalidates stale exact-candidate evidence. This governance amendment is canonical only after its exact two-file candidate passes repository quality, author correctness/safety/governance/evidence-integrity review, Ponytail review, fresh independent exact-head review with zero material findings/threads, immediate pre-landing identity/scope/mergeability reconciliation, guarded expected-head merge, and successful post-merge verification. The eight corrected-T166 proof obligations above remain T166 closeout conditions and do not block canonicalization of this amendment.

## Authority state

Before guarded landing:

```text
SPEC_012_TASKS_AMENDMENT_001=IN_QUALIFICATION
T166_TRANSPORT_PATH_CORRECTION=NOT_CANONICAL
T166_IMPLEMENTATION=BLOCKED_UNTIL_AMENDMENT_AND_T165_CLOSEOUT
T167..T185=BLOCKED_BY_PREDECESSOR
```

After guarded landing and successful post-merge verification:

```text
SPEC_012_TASKS_AMENDMENT_001=CLOSED_CANONICAL
T166_TRANSPORT_PATH_CORRECTION=ACCEPTED
T165=CLOSED_CANONICAL
T166=AUTHORIZED
T167..T185=BLOCKED_BY_PREDECESSOR
```
