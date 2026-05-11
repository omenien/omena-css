# omena-transform-passes

`omena-transform-passes` owns the P01-P40 transform pass registry and DAG
planner for the post-v5 omena-css track. It consumes
`omena-transform-cst` contracts instead of redefining pass metadata. Concrete
mutation engines will land behind this registry so transform execution cannot
drift from the semantic/cascade proof obligations.

The first execution runtime surface is intentionally conservative: it executes
lexer-backed safe commodity mutations for P01 through P13, and
observes the P40 emission boundary. P04 unit normalization is limited to zero
length dimensions inside declaration properties that accept unitless zero;
broader unit/value rewrites remain planned until property/value semantics can
prove them legal. P08 selector compression is limited to specificity-preserving
`:is()` unwrapping and duplicate argument removal for `:is()`/`:where()`. P09
shorthand combining consumes the `omena-cascade` box-shorthand proof and only
combines adjacent, non-important margin/padding longhand quartets. P10 rule
deduplication is limited to adjacent exact duplicate ordinary rules. P11 rule
merging is limited to adjacent same-selector ordinary rules and preserves
declaration order. P12 selector merging is limited to adjacent ordinary rules
with identical declaration blocks. P13 empty rule removal is limited to
top-level ordinary rules whose blocks contain only whitespace.
