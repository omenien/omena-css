# Two-Tier Identity Contract

This contract classifies identity keys that are allowed to cross revision
boundaries. It is the shared identity seam for Rust subsystems that persist,
memoize, or compare style and source facts across a session.

## Clause T1: Session-Stable Keys

Session-stable keys may be compared across revisions inside one language-server
or command session. Persistent maps, memos, and evidence stores that outlive one
snapshot must key on this tier.

Current session-stable members include:

- `ModuleIdV0`, a path newtype for linked style modules.
- `ModuleInstanceKeyV0`, a module plus configuration key.
- `LspFileId`, a `u32` assigned by the LSP file identity interner.
- `StableNodeKeyV0`, a content-derived transform node key.
- The closed-world `closure_hash`, a content hash over a sorted module closure.
- Source symbol identifiers from the Rust source frontend.

Raw spans, arena positions, positional transform node ids, and temporary
analysis indices are not session-stable keys.

## Clause T2: Snapshot-Local Keys

Snapshot-local keys may be used inside one computation or one immutable
snapshot. They must not key persistent stores. Examples include byte spans,
arena indices, positional transform node ids such as `ir:{index}`, and
temporary fact keys created by a single demand analysis.

Snapshot-local values may be serialized as evidence payloads when a separate
oracle proves the payload is only observational. That does not make them legal
persistent keys.

## Clause T3: Re-Key Migration

Replacing a synthetic string key with a compact key must derive the compact key
from the same underlying identity inputs. The compact arm lands additively, with
a dual-arm equivalence check, before the old arm can expire.

Interning a snapshot-local value for performance is allowed only when the
interner is rebuilt per snapshot. Such an interned value must not become a
cross-snapshot key.

## Clause T4: Mechanized Audit

The tier conformance check scans designated persistent-store modules and
reconstructs the collection inventory. Every scanned persistent keyed structure
must appear in `rust/omena-two-tier-identity-inventory.json` with:

- source path and owner;
- collection name and collection type;
- key type;
- explicit identity tier;
- whether the key is a persistent identity key;
- a short justification.

The check fails when a scanned entry is missing, when the inventory drifts from
the scan, or when a persistent identity key is classified as snapshot-local.

## Decision: Fact-Key Interning

Fact-key interning is a performance concern for the demand engine. A future
implementation may intern fact keys inside a snapshot-local table, but this
contract does not introduce or persist a fact-key id space.

## Decision: Cache Generation Clocks

`STYLE_IDENTITY_CACHE_VERSION` and `CANONICALIZE_PATH_CACHE_VERSION` are
session-scoped cache-coherence generation clocks. They are not identity keys and
stay in their resolver and LSP protocol planes. Re-enter this decision only if a
persistent identity key starts reading a generation counter, or if a future
session-wide invalidation plane requires a shared generation clock.
