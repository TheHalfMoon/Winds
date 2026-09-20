# Spec 012 T164 Multiplexer Persistence Provenance

## Canonical base

```text
TASK=T164
BASE=2a27aec85b3d0719c2c231a374505f7f29f3c363
BASE_TREE=c014281c083fb97884066a70991f593701aa7b01
T163=CLOSED_CANONICAL
T164=AUTHORIZED
T165..T185=BLOCKED_BY_PREDECESSOR
```

T164 is a persistence-only slice. It introduces no owner/service behavior, protocol-v2 wire behavior, PTY/ConPTY mutation, TUI/Desktop product behavior, agent detection/execution, Git worktree mutation, dependency, public/remote/plugin surface, or automatic Git landing.

## Frozen predecessor migration identities

The canonical T164 base exposes these exact predecessor migration blobs:

```text
MIGRATION_0011_BLOB=2ba3e3eb1e21e49780966829dc433d7b94577284
MIGRATION_0012_BLOB=c4b38a229273f1617e1a2cff8f96d1b899e39faa
MIGRATION_0013_BLOB=58ec274234849712fa89f1e423cbc12db9be8379
```

T164 must not modify those paths. Migration `0014_workspace_multiplexer.sql` is the only authorized migration addition.

## Persistence boundary

T164 persists only bounded presentation/recovery metadata:

- one versioned topology snapshot per `MultiplexerWorkspaceId`;
- authority-free versioned layout templates;
- explicit worktree membership independent from discovery-row existence;
- repository trust metadata keyed by canonical repository identity and monotonic revision.

Persisted topology JSON contains no process handle, PID, controller lease, provider-native session identity, credential, bearer token, transcript, verification/acceptance result, candidate identity, or evidence identity.

A persisted row is never live owner/process/controller proof.

## Failure semantics

- partial or foreign multiplexer schema fails closed instead of silently reinstalling;
- unsupported/corrupt/noncanonical JSON fails closed;
- topology generation, trust revision, update time, and membership confirmation cannot regress;
- the complete nonempty workspace snapshot set is persisted in one immediate transaction;
- every workspace in one accepted snapshot set must carry the same global TopologyGeneration and exactly one workspace must be focused;
- a failed mixed-generation/multi-focus/SQL transaction leaves the previously accepted snapshot set unchanged;
- changed topology cannot reuse an accepted generation;
- JSON and user-visible identifiers are byte-bounded;
- no authoritative JSON is silently truncated;
- deleting multiplexer metadata may cascade its own membership metadata but cannot delete canonical Git workspace rows or filesystem content.

## Herdr research boundary

```text
HERDR_T164_RESEARCH_HEAD=f090ce95d05d2e7811869be93f9a23bf4ac2c3af
HERDR_T164_RESEARCH_TREE=d5a1a447fd6043056e5ea3cb1a9bd0419957e57b
HERDR_T164_RESEARCH_MESSAGE=fix: confirm closing the last tab in the tui (#4409)
```

This upstream state was already reconciled by T163. It changes no T164 persistence requirement and admits no Herdr source or test reuse.

## Nonclaims

```text
MULTIPLEXER_OWNER_SERVICE_AUTHORIZED=NO
PROTOCOL_V2_IMPLEMENTATION_AUTHORIZED=NO
LIVE_RUNTIME_REATTACHMENT_PROVEN=NO
AGENT_START_AUTHORIZED=NO
WORKTREE_MUTATION_AUTHORIZED=NO
REMOTE_EXECUTION_AUTHORIZED=NO
PLUGIN_RUNTIME_AUTHORIZED=NO
AUTOMATIC_LANDING_AUTHORIZED=NO
HERDR_SOURCE_REUSE=NO
```

Canonical T164 acceptance authorizes T165 only.
