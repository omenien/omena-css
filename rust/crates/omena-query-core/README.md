# omena-query-core

Internal Rust crate for the low-level query runtime that sits below the public
`omena-query` facade. It owns producer fragment summaries, expression-domain
runtime state, and abstract-value projection glue so the facade can stay focused
on product routing.
