# Workspace Graph Contracts

This document records the current workspace snapshot and typed summary-plane
contracts for omena-css runtime consumers.

## Workspace Snapshot Id Contract

`OmenaWorkspaceSnapshotIdV0` is a typed re-key of `IncrementalRevisionV0`.
Equal revisions produce equal snapshot ids, different revisions produce
different snapshot ids, and the id can round-trip back to the revision it
represents. It does not introduce a new counter or identity space.

Snapshot-consuming result surfaces must carry the typed id. The LSP layer only
reports ids received from query snapshots; it does not mint or authorize them.

## Typed Graph Summary Plane Contract

Cross-file summary views are derived from `OmenaQueryCrossFileSummaryV0`.
The raw `edge_kind` histogram is recomputed from `summary.edges`, byte-matched
against the stored raw-keyed field, and every raw key is checked against the
typed vocabulary before it is accepted.

Structural graph deltas record typed added and removed cross-file summary
edges. They do not replace the reachability delta set in the streaming IFDS
engine and do not move committed graph storage.

## Residual Ledger

- Full semantic node identity remains future work. Current graph typing covers
  edge vocabulary and node roles; node identity remains the existing string
  surface.
- Summary-field migration remains future work. The current contract is an
  oracle-before-migration view over the existing field.
- Transform artifacts still need a workspace snapshot id and closed-world
  bundle hash once transform result surfaces enter the workspace snapshot
  plane.
- A full workspace snapshot struct carrying virtual-file, source, style,
  semantic, analysis, interface, and evidence views remains future work. The
  current contract only lands the typed snapshot id and summary-plane seam.
