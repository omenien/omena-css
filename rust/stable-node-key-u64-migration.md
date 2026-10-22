# Stable node key migration

The expired string compatibility arm is retired. Rust consumers now use
`StableNodeKeyU64V0`; `StableNodeKeyV0`, `additive_node_key()` and the expiry build
script are removed. Construct deterministic keys with
`StableNodeKeyU64V0::from_semantic_key_and_ordinal`, or read CST keys with
`additive_node_key_u64()`. Internal equality and collection lookup use all 64 bits.
The existing FNV framing and source-order ordinal remain unchanged.

The affected payloads now explicitly emit `schemaVersion: "1"`:

- `omena-transform-cst.stable-ir` emits only `nodeKeyU64`; the old semantic-text
  `nodeKey` field is removed.
- `omena-transform-passes.execution` mutation spans retain the `nodeKey` field,
  whose value is now the decimal encoding of the u64 identity.
- SCSS control-flow, edge-IR, call/return, fixed-point and oracle report payloads
  emit the new key representation and `nodeKeyType: "StableNodeKeyU64V0"` where
  applicable.

Every serialized `StableNodeKeyU64V0` is a **decimal JSON string**, including
`nodeKeyU64`, so JavaScript preserves values above `Number.MAX_SAFE_INTEGER`.
Compare strings directly or explicitly convert to `BigInt`; never coerce keys to
`Number`. The Rust representation stays `u64`; serialization does not restore the
retired semantic string key storage or compatibility API.

Consumers must reject or migrate the affected version-0 payloads and rebuild
persisted node-key indexes, references and reports from source. Do not mix old
`semantic-key#ordinal` strings, old floating-point numeric `nodeKeyU64` values, and
new decimal keys in one cache. Include each nested payload's `schemaVersion` in
its cache discriminator. The stable-IR identity policy is now
`schema-v1-u64-node-key-preferred-node-id-fallback`; positional node IDs remain the
fallback for missing keys or unknown schema versions. Outer envelopes and
unrelated cache identity domains are unchanged.

This is a breaking Rust/source and diagnostic-wire migration, requiring release
notes and an appropriate semver release before publication. An unchanged outer
SDK/NAPI envelope version does not imply that its nested execution report is
version 0: consumers must inspect the nested version. No backwards-compatibility
window is renewed by this change.
