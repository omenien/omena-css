# omena-transform-passes

`omena-transform-passes` owns the P01-P40 transform pass registry and DAG
planner for the post-v5 omena-css track. It consumes
`omena-transform-cst` contracts instead of redefining pass metadata. Concrete
mutation engines will land behind this registry so transform execution cannot
drift from the semantic/cascade proof obligations.

The first execution runtime surface is intentionally narrow: it executes the
safe P02 comment-strip mutation and observes the P40 emission boundary, while
returning explicit `plannedOnly` outcomes for registered passes whose mutation
engines have not landed yet.
