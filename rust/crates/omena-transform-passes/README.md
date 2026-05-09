# omena-transform-passes

`omena-transform-passes` owns the P01-P40 transform pass registry and DAG
planner for the post-v5 omena-css track. It consumes
`omena-transform-cst` contracts instead of redefining pass metadata. Concrete
mutation engines will land behind this registry so transform execution cannot
drift from the semantic/cascade proof obligations.

The first execution runtime surface is intentionally conservative: it executes
lexer-backed safe commodity mutations for P01, P02, P03, P05, P06, and P07, and
observes the P40 emission boundary. Context-sensitive passes such as P04 unit
normalization remain `plannedOnly` until property/value semantics can prove the
rewrite is legal.
