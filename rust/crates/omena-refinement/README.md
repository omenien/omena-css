# omena-refinement

`omena-refinement` hosts refinement type contracts and strict-superset abstract
property value wrappers.

The crate also exposes `summarize_cascade_dimensional_refinement_bridge_v0`.
That bridge evaluates the existing `CascadeValueFamilyV0` substrate through
existing `RefinementPropertyPredicateV0` predicates and reports context
verdicts plus witness/provenance counts. It is a research-staged dimensional
refinement substrate only: it does not fork a unit system, complete
Liquid-Haskell-style inference, complete SMT refinement, or claim a theorem.
