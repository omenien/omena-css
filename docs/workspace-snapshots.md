# Workspace snapshot bindings

A bound workspace request carries the complete `OmenaWorkspaceSnapshotBindingV0`:
the workspace root, `OmenaWorkspaceSnapshotIdV0` revision, and an input commitment.
The numeric revision remains available to existing SDK requests. It identifies a
revision within its owner; it is not sufficient to compare two hosts.

The commitment is a derived integrity digest. It covers the ordered style and
source inputs, source language and admitted provider evidence, actual source
syntax indexes, manifests, resolver inputs, settings, external SIF facts and
their admitted trust records, and source-corpus completeness. Length-framed,
domain-separated serialization preserves meaningful list order. The digest does
not issue module identities or authenticate a remote publisher.

Configuration here means the admitted effective inputs: diagnostic severity and
deep-analysis mode, the five LSP feature flags, package manifest text, resolver
path mappings and disk identities, and the external SIF cache fingerprint. It
does not promise the raw bytes of every configuration file or scheduler/cache
settings that the query does not read. Utility configuration is independently
loaded during source admission and its resulting source facts are committed.
The optional `configContentDigest` is an additional caller input; the LSP omits
it. It is not evidence that a receiver read or authenticated a configuration file.

## Reading an exported snapshot

The version `1` LSP `omena/sdkWorkflow` envelope can export a binding and the
inputs needed to admit it. The CLI SDK transport and daemon bound handshake
carry that same binding. Each receiver reconstructs source facts from actual
text and language, resolves imports, admits provider results against parsed
expression IDs and exact spans, and recomputes the commitment. Serialized
`SourceSyntaxIndex` values and transported SIF trust assertions are not admitted.
Local external SIFs are independently regenerated through the existing bridge
and trust path; a lock claim is not equivalent to that admission.

External SIF inputs are scoped to the root's reachable dependency closure,
including forwarded interfaces and dependency entries. Owner exports select
already admitted resolved targets without reading current disk contents. Each
receiver independently admits its own local bridge/trust results and applies
the same deterministic projection. Unrelated workspaces' SIFs and trust records
do not enter the root's commitment. Equal relative import spellings retain their
document context; ambiguous convenience aliases yield to the consumer's existing
contextual resolver while all resolved target facts remain present.

The commitment also includes independently admitted contextual resolution edges:
each editor document and import spelling identifies the actual bridge backing
URL, canonical SIF URL, and full artifact digest. A disk SIF origin includes its
backing URL and artifact digest, keeping its transitive imports distinct from an
unsaved editor buffer at the same URI. Package transitive resolution uses that
physical backing URL. Exports project these existing records without disk reads;
receivers produce their own records through admission. Swapping two importers'
targets changes the commitment even when the unordered SIF/trust set is identical.
The existing SIF input carrier retains this admission metadata for query
consumers. A resolved SIF reference carries its selected importer edge through
forward and dependency traversal; an admitted corpus cannot fall back to a
legacy global alias when that context is missing or mismatched. Bound reads
verify that attached consumer metadata equals the committed edge set. Existing
Salsa input equality includes the metadata, and the disk diagnostics environment
fingerprint commits it explicitly, including mappings to corpus members.

These records provide integrity provenance. The legacy serialized admission
result omits the Rust metadata; Rust callers supply the actual admitted context
to preserve consumer resolution and input commitment consistency.

The resulting SDK view is immutable for a read request. Read clones retain a
reader handle and cannot publish another revision. The destination session's
existing owner performs supported mutations against an expected current binding,
then publishes reconstructed source and provider facts with the changed styles.

The scope of freshness is explicit:

- LSP bound requests must match the current LSP owner.
- Daemon bound requests must match that daemon session's current owner. Reconnects
  cannot replace an existing different binding by importing another claim.
- A CLI invocation admits an immutable input set. It cannot discover a remote
  owner's subsequent mutations from the commitment alone.

Consequently, equal complete inputs and revision can have equal portable
bindings. A coherent claim admitted by a fresh receiver is not proof that a
different process still considers it current. Cross-process current-write
authority is not conveyed by this envelope.

## Protecting a workspace edit

A snapshot-backed transaction derives expected file digests from its admitted
read view. It retains the same actual destination owner's reader handle and
requires its current native write guard across staging, precondition checks,
journal validation, renames, and rollback. An equal portable binding from a
different owner does not satisfy this provenance check. A metadata-only owner
advance makes an older transaction stale even when the file bytes are unchanged.

Imported origin is permanent for destination writes. A local in-memory mutation
and republication of an imported snapshot does not promote it to native file
write authority. Native disk-only CLI writes remain a separate unbound route
with their existing transaction safety conditions. Bound daemon format and lint
requests currently refuse the unadapted disk route; they require integration with
the actual resident document owner before they can succeed.

## Wire compatibility and verification

Version `1` wraps existing workflow payloads in an explicit bound envelope.
Legacy requests retain their version `0` payloads. A binding without the explicit
bound version is refused instead of silently becoming an unbound request. A
bound request's inner numeric revision must agree with its full envelope.

The snapshot contract verification probe takes already-built LSP, daemon,
and CLI binaries. It captures the actual LSP export, independently admits it
through TCP and CLI, compares complete diagnostics and explain payload bytes,
and checks input and stale-owner refusals. This contract probe does not establish
resident parse reuse, worker cancellation, or memory and latency budgets.

Contextual SIF admission retains the verdict actually returned by the existing
bridge for that full artifact. Identical package bytes can be admitted under
different root verdict directories. The selected snapshot trust frame therefore
uses its admitted importer, specifier, backing URL, canonical URL and artifact
digest as a context key, paired with the actual bridge verdict. The input
validator checks that this frame agrees with the attached consumer edges;
a global canonical-URL map cannot replace a contextual verdict.

Forward and dependency edges retain the initiating document as well as the disk
SIF origin. Repeated targets in one admission retain the actual bridge result.
A supplied legacy SIF without a current-context verdict must be admitted before
it can supply contextual trust; generic non-context seed APIs retain their
existing unbound behavior. Export performs no filesystem reconstruction, and
untrusted transfers cannot supply this admission metadata.

These Rust fields are omitted from legacy SIF JSON and transport schemas. They
intentionally participate in commitments, existing memo input equality and the
disk-cache context frame. Rust callers constructing admission edges must provide
both their actual verdict and initiating document context. A change to either
alters the bound input commitment and the diagnostics cache environment key.
The LSP build continues to refuse recorded
verdicts when attestation verification is unavailable; metadata does not change
verification policy or grant an imported snapshot native write authority.

Local bound imports and subsequent in-memory mutations select recorded verdicts
from the importing workspace's `.cache/omena` verdict directory. A shared package
file does not cause another workspace's verdict directory to be used. The LSP
keeps its configured regenerable-cache location; SDK imports use the process
cache policy for their explicit local root. Recorded verdicts remain scoped to
that root even when no regenerable disk cache is available. Cache memory hits
still verify the actual root's verdict under the existing verification policy.

`OmenaBridgeExternalSifStorageV0::workspace_cache_root()` returns `Option<&Path>`
to represent a disabled regenerable disk cache. Existing constructors that take
a cache root return `Some`; the optional-cache constructor accepts a separate
recorded-verdict directory. The process-root factory accepts absolute native
paths and file URIs using the bridge's existing URI conversion.

## Rust caller migration

This snapshot slice contributes to the declared pre-1.0 minor breaking release.
`OmenaQueryExternalSifInputV0.admitted_resolution_edges` and
`OmenaQueryBridgeExternalSifTrustedResolutionV1.resolution_edges` are new public
fields. Rust struct literals must supply them, and exhaustive patterns must name
them or use `..`. An empty vector is appropriate for a genuinely legacy, unbound
input. A caller retaining admitted inputs must carry the actual bridge-produced
edges, artifact digests and verdicts; replacing that provenance with an empty
vector changes the input's meaning.

Both fields use `serde(skip)`. The input carrier implements both Serialize and
Deserialize: legacy JSON omits its edges, deserialization defaults them to empty,
and even a supplied `admittedResolutionEdges` property is ignored. A nonempty
admitted input therefore loses provenance and equality on a JSON roundtrip.
`Eq` includes the edge vector even though the legacy JSON does not. The trusted
resolution result implements Serialize, Default and Eq, but not Deserialize;
its equal legacy JSON likewise does not imply equal Rust values. These omitted
fields do not add wire or IDL properties.

The loss can also change selection. Once any corpus member carries admission
metadata, lookup uses the importer and specifier, validates the complete artifact
and conflicting contextual records, and refuses a missing or mismatched edge
without borrowing a legacy alias. If a JSON roundtrip removes all those edges,
the remaining inputs again use the legacy unbound lookup. A serialized legacy SIF
is not a portable substitute for the admitted snapshot transfer and re-admission
path.

`OmenaSdkWorkspaceV0::replace_style_resolution_inputs` now returns
`Result<OmenaSdkSnapshotResponseV0, OmenaError>`. A caller that previously read
`workspace.replace_style_resolution_inputs(inputs).snapshot_id` must handle the
result first, for example `workspace.replace_style_resolution_inputs(inputs)?.snapshot_id`.
`replace_style_sources` already returned Result. Both methods now check revision
addition instead of saturating: changed unbound inputs at `u64::MAX` return
`workspace.snapshot-revision-exhausted` before storing inputs. Unchanged no-ops
remain successful at that revision and return the same complete snapshot.

Bound resolver changes require replacement of the complete admitted source-fact
view and return `workspace.snapshot-full-replacement-required`, including at the
maximum revision. Supported bound style changes require a current owner and its
admitted utility inputs, reconstruct and admit the next view, then publish it.
A current request clone cannot publish; a stale style request fails the live
binding check even for a no-op. Resolver no-ops retain the existing behavior of
returning that workspace clone's snapshot without a new publication. Callers
must still validate freshness for a subsequent bound read. Refusals preserve the
stored inputs, revision and binding.

The NAPI Workspace and CachedWorkspace and the WASM Workspace retain their
existing JS replacement methods. Success and no-op responses preserve the full
snapshot payload. Native errors carry the serialized typed envelope, and WASM
throws that typed envelope. Maximum-revision and bound-owner error paths are
validated through private adapter fixtures using existing Rust constructors;
production JS constructors do not expose revision or owner seeding.

At the existing JS number boundary, WASM's JSON-compatible serializer cannot
encode a revision above `Number.MAX_SAFE_INTEGER`. A Rust no-op at `u64::MAX`
still succeeds internally, but its WASM response returns the typed
`sdk.response-serialization` error. A changed input at that revision reaches the
revision-exhaustion error first. This slice preserves that serializer behavior;
normal-revision adapter no-ops return the complete successful snapshot.
