# omena-ensemble

`omena-ensemble` hosts the M4-beta replica-overlap lane. It computes
cross-file pairwise overlap, workspace overlap distributions, SBM
detectability thresholds, and the shared cascade-outcome projection policy.

The crate is intentionally a leaf consumer of `omena-cascade` and
`omena-query`. It does not modify cascade or query substrate contracts.
