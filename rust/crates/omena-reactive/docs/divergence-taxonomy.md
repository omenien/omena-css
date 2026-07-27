# Reactive Observation Divergence Taxonomy

The read-only reactive observer may record a reviewed timing difference while a
wave is still stabilizing. It must converge with the authoritative execution
path at flush. A mismatch that survives flush, or a mismatch without one of the
IDs below, is a blocker.

## Class: `flushConeClosureTiming`

A value in the observer cone can change at a different intermediate step from
the authoritative scheduler while both paths are still processing the same
event. This class is admissible only before flush and only when all compared
values converge at flush.

## Class: `midWaveReadTiming`

A projection read between bounded stabilization steps can observe an earlier
settled value than a direct execution-time read. This class is admissible only
before flush, may not cause an external effect, and must converge at flush.

## Registration Rules

- New classes require an explicit reviewed source change; runtime observations
  never extend this list.
- An unknown class is a blocker.
- Every target-set, snapshot, generation, tier digest, cutoff, and pending-work
  mismatch is a blocker at flush.

## Authority Boundary

Tide remains the sole scheduling and publishing authority. The reactive graph
only records observations and receipts. Any future authority handoff is a
separately gated change that must re-establish the 19 Tide tests without edits,
prove flush-time cone closure, and measure both interface-changing and
interface-preserving edits on a pinned product corpus. The synthetic engine-step
envelope is a bounded-work smoke test, not product-path performance evidence.
