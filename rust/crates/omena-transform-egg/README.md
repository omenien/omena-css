# omena-transform-egg

`omena-transform-egg` owns the optional e-graph rewrite boundary for selector
and computed-value rewrites. It runs the optional `egg` equality-saturation
engine for accepted rewrite candidates and reports source witnesses for the
selector and calc rewrites that the transform DAG applies.

The crate keeps proof obligations explicit:

- selector rewrites must preserve specificity and matching semantics;
- calc rewrites must preserve computed value;
- accepted rewrites must preserve provenance and carry a cascade-safe witness.

M6 positioning also exposes `summarize_contextual_eqsat_scaffold_v0`. That
summary is a contextual equality-saturation scaffold for the #66/#73 execution
view only. It deliberately reports `egglogBindingReady = false`,
`threeViewFusionReady = false`, and `theoremClaimed = false`; the implemented
engine remains the current optional `egg` boundary.
