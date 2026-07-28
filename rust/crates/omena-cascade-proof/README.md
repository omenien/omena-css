# omena-cascade-proof

Product-owned cascade proof contracts for Omena CSS.

The crate defines backend-neutral proof inputs, verdicts, and telemetry used by
transform admission. Its default stub backend remains deterministic, while the
optional Z3 backend can discharge the same typed contracts without changing
their product-facing representation.
