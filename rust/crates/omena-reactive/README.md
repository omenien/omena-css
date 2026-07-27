# omena-reactive

`omena-reactive` provides a small, deterministic reactive graph for observing
Omena control-plane decisions without owning scheduling or external effects.

The graph is deliberately static. Inputs are deposited at event boundaries,
derived nodes stabilize in height order, and effect boundaries produce typed
receipts that a caller may compare with an authoritative execution path. The
crate does not publish diagnostics or recompute facts owned by another engine.
