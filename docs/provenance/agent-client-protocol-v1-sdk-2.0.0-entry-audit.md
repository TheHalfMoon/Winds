# Agent Client Protocol entry audit for formal Spec 006

**Status:** governance/specification input only — no ACP dependency is landed by this document.

**Observed:** 2026-08-21

**Winds entry base:** `6eb6390b0f7cb33ac5215a5990589c8018ed05d6`

## Decision

For the first formal Agentic slice, Winds freezes the following ACP compatibility target for specification and dependency planning:

- **ACP stable wire protocol:** `1`
- **ACP stable v1 JSON Schema release:** `schema-v1.20.0`
- **Schema release commit:** `5e89c71497fe07dd4ae633c181a17224f4a8956d`
- **Official Rust SDK crate target:** `agent-client-protocol = 2.0.0`
- **Rust SDK release commit:** `ce023279824149008659dd8f4b8b70266a7e8210`

This is a protocol/design pin, not permission to add the crate yet. The first implementation task that actually lands ACP MUST exact-pin the selected crate version, commit the Winds-resolved `Cargo.lock`, compile/test the exact dependency graph under the repository-pinned Rust toolchain, and record the exact transitive/license audit before acceptance.

## Primary-source observations

### Protocol/schema repository

Official repository: `agentclientprotocol/agent-client-protocol`.

At release commit `5e89c71497fe07dd4ae633c181a17224f4a8956d`:

- GitHub verification reports `verified=true` / `reason=valid`;
- the release updates stable schema v1 to `1.20.0`;
- the schema changelog records `schema-v1.20.0` on 2026-07-21;
- the repository's versioning contract states that ACP wire compatibility is determined separately by `protocolVersion` during `initialize`;
- the current stable ACP wire protocol version is `1`.

The same release also carries a v2 alpha schema. Winds does **not** select that draft/alpha surface for the first formal slice.

### Official Rust SDK

Official repository: `agentclientprotocol/rust-sdk`.

At release commit `ce023279824149008659dd8f4b8b70266a7e8210`:

- GitHub verification reports `verified=true` / `reason=valid`;
- `agent-client-protocol` is released as `2.0.0`;
- the 2.0 release explicitly keeps the stable ACP v1 wire schema unchanged while making breaking Rust API/transport-boundary changes;
- draft protocol v2 remains separately feature-gated as `unstable_protocol_v2`;
- native MCP-over-ACP remains an opt-in unstable feature rather than an implied stable requirement.

Therefore Winds can use the maintained Rust 2.x SDK surface while still targeting stable ACP wire protocol v1.

## First-slice feature policy

The formal Spec 006 plan MUST NOT rely on draft ACP v2 behavior.

For the first implementation slice:

```text
ACP_WIRE_PROTOCOL=1
ACP_SCHEMA=schema-v1.20.0
ACP_RUST_SDK=2.0.0
UNSTABLE_PROTOCOL_V2=DISABLED
UNSTABLE_MCP_OVER_ACP=DISABLED
REMOTE_HTTP_WEBSOCKET_CONTROL=NOT_AUTHORIZED
MCP_RUNTIME=NOT_AUTHORIZED_BY_THIS_PIN
```

Any other unstable ACP feature requires a separate task-level justification and exact-head review; it is not authorized merely because the SDK exposes it.

## Transport boundary

The first connected-session proof should use the narrowest local structured transport supported by the chosen adapter/runtime. This audit does not authorize a public Winds protocol, a network listener, HTTP/SSE/WebSocket control, remote execution, or a persistent daemon/session owner.

A later persistent-owner/IPC design must have its own explicit threat model, authenticated local-control contract, versioned lifecycle/ownership semantics, and recovery behavior before code is written.

## Authority and security interpretation

ACP is a structured control protocol, not an authority boundary by itself.

Winds MUST preserve these distinctions:

- ACP-declared workspace roots are scope declarations, not OS sandbox proof;
- an ACP permission/elicitation request is a protocol event, not self-granted Winds authority;
- runtime/vendor capability declaration is not equivalent to local Winds observation;
- native session resume is not equivalent to canonical Winds task continuity;
- agent messages and tool reports remain source-labelled and do not become `WINDS_OBSERVED` evidence by transport alone;
- a third-party runtime with direct host access that Winds cannot mediate must be reported with downgraded enforcement quality rather than represented as Winds-enforced.

## MCP disposition

MCP is deliberately **out of the first entry pin**. No MCP specification or SDK is selected by this audit because the first formal Agentic walking skeleton does not need MCP to prove named sessions, continuity, one Codex/Claude structured integration path, and one bounded delegation contract.

If a later Spec 006 task introduces MCP, that task MUST first pin the exact then-current MCP specification and SDK revision, document its transport/tool authority semantics, and prove that MCP cannot bypass the Winds authority ceiling.

## Dependency/provenance landing gate

Before `agent-client-protocol` enters `Cargo.toml`, the implementing task MUST independently verify at least:

1. exact crate version and source checksum resolved by Cargo;
2. exact enabled feature set, with unstable protocol v2 disabled unless separately authorized;
3. complete direct/transitive dependency graph and license obligations;
4. Rust 1.97.1 compatibility under Winds CI or the then-canonical pinned toolchain if that changed through a separate accepted task;
5. platform compile/test behavior for every platform claimed by the slice;
6. bounded framing/output/error handling appropriate to the chosen local transport;
7. no implicit credential duplication or auto-install behavior;
8. fail-closed mapping between native runtime/session identity and Winds canonical workspace/session/task identity.

Failure of any landing gate reopens the dependency decision rather than permitting an unreviewed workaround.

## Why this pin is sufficient for formal specification

The entry criterion requires an exact ACP protocol/SDK decision before architecture. The combination above provides:

- a stable wire contract (`protocolVersion = 1`);
- an exact stable schema artifact (`schema-v1.20.0`);
- an exact maintained official Rust SDK (`2.0.0`);
- explicit exclusion of draft v2 and unstable MCP-over-ACP;
- a fail-closed dependency landing gate before any runtime code.

This document authorizes **specification work only**. It does not add ACP code, start an agent process, send prompts, create a daemon, or broaden Winds execution authority.
