# Spec 011 T146 Domain and Entropy Contract

## Canonical base

TASK=T146
BASE=a90575133bfccd8e2879f45930d50497e50aa504
BASE_TREE=5966ea06f0ac138d4b35be1f27e785eade305149

T146 is a pure Rust domain slice. It introduces no persistent owner process, IPC endpoint, migration, dependency, service manager, terminal behavior, provider execution path, renderer authority, Git authority, verification authority, or remote/public control surface.

## Identity contract

RuntimeNamespaceId and OwnerGenerationId are independent 128-bit identities. Their canonical serialized representation is exactly 32 lowercase hexadecimal characters.

The domain rejects:
- all-zero entropy;
- wrong-length serialized values;
- uppercase/non-hex serialized values;
- aliases, timestamps, counters, PIDs, filesystem paths, or database row IDs as identity substitutes.

A mutable runtime alias is a presentation/search label only and cannot establish identity equivalence.

## Entropy implementation seams frozen for later tasks

T146 freezes the platform contract but intentionally does not call an OS CSPRNG.

Future platform-owning tasks must supply exactly 16 bytes from:
- Linux: getrandom(2) through the accepted libc/platform boundary;
- macOS: getentropy(3) through the accepted libc/platform boundary;
- native Windows: BCryptGenRandom through the exact Win32 dependency seam qualified by T150.

Short, failed, or all-zero entropy fails closed.

No UUID/random dependency is admitted by T146. windows-sys remains transitive-only and is not directly admitted by this slice.

## Authority and lifecycle contract

The domain keeps these dimensions structurally separate:
- Winds live ownership;
- process liveness;
- endpoint availability;
- continuity class;
- observer/controller authority;
- lifecycle proof/source class.

OWNERSHIP_LOST therefore cannot be upgraded by a running PID, remembered row, endpoint path, alias, or presentation state.

Lifecycle events carry only exact runtime/generation identity, sequence, typed event kind, proof class, optional controller/client identity, and optional observed timestamp. The schema has no arbitrary prose/body field and no credential/environment/verification/acceptance/Git authority field.

## Provenance and nonclaims

No Herdr source is copied or adapted in T146. Herdr remains research/test evidence only.

Historical provider/runtime nonclaims remain unchanged:

T079_LIVE_PASS=NO
T080_LIVE_PASS=NO
T082_WORKER_LIVE_PASS=NO
REAL_CLAUDE_EXECUTION=NO
REAL_CODEX_WORKER_EXECUTION=NO

T146 does not authorize T147 or later behavior until its own exact candidate is canonically accepted with post-merge proof.
