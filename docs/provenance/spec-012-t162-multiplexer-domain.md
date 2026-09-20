# Spec 012 T162 Multiplexer Domain Contract

## Canonical base

TASK=T162
BASE=0faeb964ee07bcea7a12fda599555892fc973f25
BASE_TREE=2c63f224f173f3f088f21ab5515b0259e1f06bd3

SPEC_012_ENTRY=CLOSED_CANONICAL
SPEC_012_SPEC=CLOSED_CANONICAL
SPEC_012_PLAN=CLOSED_CANONICAL
SPEC_012_TASKS=CLOSED_CANONICAL
T162=AUTHORIZED
T163..T185=BLOCKED_BY_PREDECESSOR

HERDR_T162_RECHECK=f10df75c8a5f1b5e5f90689c3a8ceb6758366e05
HERDR_T162_RECHECK_TREE=3fa0fa2a9f48cf1cf1f0820cfab94abda63b8549

T162 is a pure Rust domain/contract slice. It introduces no database migration, owner/service integration, local-control protocol-v2 behavior, endpoint, terminal/process mutation, TUI/Desktop behavior, detector execution, worktree mutation, dependency, lockfile change, workflow-semantic change, remote/plugin/marketplace/updater behavior, provider execution, or Git landing authority.

## Identity contract

T162 freezes five independent opaque 128-bit domains:

- MultiplexerWorkspaceId
- TabId
- PaneId
- LayoutTemplateId
- AgentObservationId

Their canonical serialized representation is exactly 32 lowercase hexadecimal characters. All-zero entropy, wrong length, uppercase, non-hex, and the existing Git workspace string format are rejected.

These identities remain distinct from:

- the existing Git Workspace ID;
- Project / Session / Workstream;
- RuntimeNamespaceId and OwnerGenerationId;
- terminal ID;
- provider-native session identity;
- candidate/evidence identity;
- OS PID;
- path, alias, label, ordinal, focus, and geometry.

Pane replacement must use a different PaneId. T162 freezes that invariant with an explicit domain validator; T163 owns lifecycle/state-machine enforcement.

TopologyGeneration is a separate nonzero monotonic u64 domain. It is stale-ordering proof only and is never pane/workspace/runtime identity.

## Entropy seam

T162 intentionally does not call an OS CSPRNG.

When later authorized creation logic needs a fresh multiplexer identity, it must reuse the already-qualified Spec 011 128-bit entropy seams:

- Linux: one exact 16-byte getrandom(2) result through the accepted libc/platform boundary;
- macOS: one exact 16-byte getentropy(3) result through the accepted libc/platform boundary;
- native Windows: BCryptGenRandom through the accepted windows-sys seam.

Short, failed, or all-zero entropy fails closed.

No UUID/random dependency is admitted by T162.

## Authority contract

MultiplexerAuthority has only:

- OBSERVER
- MULTIPLEXER_WRITE

MultiplexerWrite is topology authority only. It does not imply the existing persistent-runtime CONTROLLER authority, and Runtime Controller does not imply MultiplexerWrite.

T162 implements no lease acquisition, owner connection state, topology mutation, process mutation, or controller mechanics.

## Client capability contract

ClientSurfaceCapability distinguishes:

- NON_INTERACTIVE_OBSERVER;
- CONTROLLING_TERMINAL;
- TRUSTED_DESKTOP_TERMINAL_SURFACE.

This freezes the Plan rule that TUI/CLI terminal requirements cannot be generalized into a Desktop controlling-TTY requirement. Capability preflight behavior itself remains T165 scope.

## Error contract

The closed T162 vocabulary includes explicit stale topology, unknown/closed target, identity reuse, duplicate/replayed mutation, snapshot limit, capability unavailable, write/controller requirement, unsupported operation/platform, ownership loss, outcome unknown, and generation exhaustion states.

No free-form error string becomes authority.

## Agent-observation naming boundary

AgentObservationTopologyBinding uses the exact field name multiplexer_workspace_id and contains no generic workspace_id field.

The final T173 observation schema may add separately typed optional Git workspace context. T162 does not invent or conflate that later context.

## Provenance and nonclaims

Herdr remains research/test-design evidence only. No Herdr source or test is copied or adapted in T162.

T162 does not authorize T163 until this exact candidate lands canonically and its post-merge push workflows succeed.
