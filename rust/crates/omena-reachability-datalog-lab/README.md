# omena-reachability-datalog-lab

Independent Datalog witness crate for Omena cross-file hypergraph reachability.

This crate is intentionally kept out of product dependency closures. Product
code owns the shipped reachability path; this crate independently re-derives the
same reachable-node set for equivalence checks.
